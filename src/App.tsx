import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import QRCode from "qrcode";
import StickyNote from "./components/StickyNote";
import type { Note, ClipboardEvent, Connection } from "./types";
import { CONNECTION_COLORS } from "./types";
import "./App.css";

type Theme = "light" | "dark" | "contrast";

interface OtgStatus {
  running: boolean;
  url: string | null;
  pairing_code: string | null;
}

interface PeerInfo {
  ip: string;
  port: number;
}

export default function App() {
  const [notes, setNotes] = useState<Note[]>([]);
  const [connections, setConnections] = useState<Connection[]>([]);
  const [viewX, setViewX] = useState(0);
  const [viewY, setViewY] = useState(0);
  const [scale, setScale] = useState(1);
  const [panning, setPanning] = useState(false);
  const [theme, setTheme] = useState<Theme>(() =>
    (localStorage.getItem("charly-theme") as Theme) || "dark"
  );
  const canvasRef = useRef<HTMLDivElement>(null);
  const panRef = useRef<{ mx: number; my: number; vx: number; vy: number } | null>(null);
  const [pinnedNoteId, setPinnedNoteId] = useState<string | null>(null);
  const [connColor, setConnColor] = useState(CONNECTION_COLORS[0]);
  const [connColorOpen, setConnColorOpen] = useState(false);
  const [otgStatus, setOtgStatus] = useState<OtgStatus>({ running: false, url: null, pairing_code: null });
  const [otgPanelOpen, setOtgPanelOpen] = useState(false);
  const [qrDataUrl, setQrDataUrl] = useState<string | null>(null);
  const [peers, setPeers] = useState<PeerInfo[]>([]);
  const [peersLoading, setPeersLoading] = useState(false);

  useEffect(() => {
    invoke<Note[]>("get_notes")
      .then(setNotes)
      .catch(() => setNotes([]));
    invoke<Connection[]>("get_connections")
      .then(setConnections)
      .catch(() => setConnections([]));

    const unlisten = listen<ClipboardEvent>("clipboard-capture", (event) => {
      const ce = event.payload;
      const newNote: Note = {
        id: ce.id,
        x: ce.x,
        y: ce.y,
        width: ce.width,
        height: ce.height,
        color: ce.color,
        content_type: ce.content_type as Note["content_type"],
        content_ref: ce.content_type === "text" ? ce.content : ce.content_ref,
        file_name: ce.file_name,
        created_at: ce.created_at,
        z_index: ce.z_index,
      };
      setNotes((prev) => [...prev, newNote]);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    localStorage.setItem("charly-theme", theme);
    document.documentElement.setAttribute("data-theme", theme);
  }, [theme]);

  const cycleTheme = useCallback(() => {
    setTheme((t) => (t === "light" ? "dark" : t === "dark" ? "contrast" : "light"));
  }, []);

  const updateNote = useCallback(
    (id: string, x: number, y: number, width: number, height: number, zIndex: number) => {
      setNotes((prev) =>
        prev.map((n) => (n.id === id ? { ...n, x, y, width, height, z_index: zIndex } : n))
      );
      invoke("update_note", { id, x, y, width, height, zIndex }).catch(() => {});
    },
    []
  );

  const changeColor = useCallback((id: string, color: string) => {
    setNotes((prev) => prev.map((n) => (n.id === id ? { ...n, color } : n)));
    invoke("update_note_color", { id, color }).catch(() => {});
  }, []);

  const deleteNote = useCallback((id: string) => {
    setNotes((prev) => prev.filter((n) => n.id !== id));
    setConnections((prev) => prev.filter((c) => c.from_note_id !== id && c.to_note_id !== id));
    invoke("delete_note", { id }).catch(() => {});
  }, []);

  const copyText = useCallback(async (text: string) => {
    await invoke("copy_to_clipboard", { text }).catch(() => {
      navigator.clipboard.writeText(text).catch(() => {});
    });
  }, []);

  const openFileManager = useCallback(async (path: string) => {
    await invoke("open_in_file_manager", { path }).catch(console.error);
  }, []);

  const bringToFront = useCallback(
    (id: string, zIndex: number) => {
      setNotes((prev) =>
        prev.map((n) => (n.id === id ? { ...n, z_index: zIndex } : n))
      );
    },
    []
  );

  const handlePin = useCallback(
    (noteId: string) => {
      if (pinnedNoteId === null) {
        setPinnedNoteId(noteId);
      } else if (pinnedNoteId === noteId) {
        setPinnedNoteId(null);
      } else {
        const existing = connections.find(
          (c) =>
            (c.from_note_id === pinnedNoteId && c.to_note_id === noteId) ||
            (c.from_note_id === noteId && c.to_note_id === pinnedNoteId)
        );
        if (existing) {
          setConnections((prev) => prev.filter((c) => c.id !== existing.id));
          invoke("delete_connection", { id: existing.id }).catch(() => {});
        } else {
          const id = crypto.randomUUID();
          const newConn: Connection = {
            id,
            from_note_id: pinnedNoteId,
            to_note_id: noteId,
            color: connColor,
          };
          setConnections((prev) => [...prev, newConn]);
          invoke("add_connection", { conn: newConn }).catch(() => {});
        }
        setPinnedNoteId(null);
      }
    },
    [pinnedNoteId, connections, connColor]
  );

  const deleteConnection = useCallback((id: string) => {
    setConnections((prev) => prev.filter((c) => c.id !== id));
    invoke("delete_connection", { id }).catch(() => {});
  }, []);

  const handleCanvasMouseDown = useCallback(
    (e: React.MouseEvent) => {
      if (e.target !== canvasRef.current && !(e.target as HTMLElement).classList.contains("canvas-bg")) return;
      if (e.button === 1 || e.button === 2 || (e.button === 0 && e.altKey)) {
        setPanning(true);
        panRef.current = { mx: e.clientX, my: e.clientY, vx: viewX, vy: viewY };
      }
      if (e.button === 0 && pinnedNoteId) {
        setPinnedNoteId(null);
      }
    },
    [viewX, viewY, pinnedNoteId]
  );

  useEffect(() => {
    if (!panning) return;
    const handleMouseMove = (e: MouseEvent) => {
      if (panRef.current) {
        const dx = e.clientX - panRef.current.mx;
        const dy = e.clientY - panRef.current.my;
        setViewX(panRef.current.vx + dx);
        setViewY(panRef.current.vy + dy);
      }
    };
    const handleMouseUp = () => { setPanning(false); panRef.current = null; };
    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, [panning]);

  const handleWheel = useCallback(
    (e: React.WheelEvent) => {
      if (e.ctrlKey) {
        e.preventDefault();
        const delta = e.deltaY > 0 ? 0.9 : 1.1;
        setScale((s) => Math.max(0.15, Math.min(3, s * delta)));
      }
    },
    []
  );

  const addManualNote = useCallback(() => {
    const id = crypto.randomUUID();
    const now = new Date().toISOString();
    const colors = ["#FEF08A", "#FECACA", "#BFDBFE", "#BBF7D0", "#DDD6FE", "#FED7AA", "#FBCFE8", "#A5F3FC"];
    const note: Note = {
      id,
      x: -viewX / scale + 200 + Math.random() * 100,
      y: -viewY / scale + 200 + Math.random() * 100,
      width: 220,
      height: 180,
      color: colors[Math.floor(Math.random() * colors.length)],
      content_type: "text",
      content_ref: "",
      file_name: null,
      created_at: now,
      z_index: Date.now(),
    };
    setNotes((prev) => [...prev, note]);
    invoke("add_note", { note }).catch(() => {});
  }, [viewX, viewY, scale]);

  const toggleOtg = useCallback(async () => {
    if (otgStatus.running) {
      await invoke("stop_otg");
      setOtgStatus({ running: false, url: null, pairing_code: null });
      setQrDataUrl(null);
      setOtgPanelOpen(false);
    } else {
      const status = await invoke<OtgStatus>("start_otg");
      setOtgStatus(status);
      setOtgPanelOpen(true);
      if (status.url) {
        const dataUrl = await QRCode.toDataURL(status.url, { width: 200, margin: 1 });
        setQrDataUrl(dataUrl);
      }
    }
  }, [otgStatus.running]);

  const scanForPeers = useCallback(async () => {
    setPeersLoading(true);
    try {
      const result = await invoke<PeerInfo[]>("scan_for_peers");
      setPeers(result);
    } catch {
      setPeers([]);
    }
    setPeersLoading(false);
  }, []);

  const noteMap = new Map(notes.map((n) => [n.id, n]));

  const connectionLines = connections
    .map((conn) => {
      const from = noteMap.get(conn.from_note_id);
      const to = noteMap.get(conn.to_note_id);
      if (!from || !to) return null;
      return {
        ...conn,
        x1: from.x + from.width / 2,
        y1: from.y + from.height / 2,
        x2: to.x + to.width / 2,
        y2: to.y + to.height / 2,
      };
    })
    .filter(Boolean) as (Connection & { x1: number; y1: number; x2: number; y2: number })[];

  return (
    <div
      className="canvas-container"
      ref={canvasRef}
      onMouseDown={handleCanvasMouseDown}
      onWheel={handleWheel}
      onContextMenu={(e) => e.preventDefault()}
    >
      <div
        className="canvas-bg"
        style={{
          transform: `translate(${viewX}px, ${viewY}px) scale(${scale})`,
          transformOrigin: "0 0",
        }}
      />

      <div
        className="canvas-layer"
        style={{
          transform: `translate(${viewX}px, ${viewY}px) scale(${scale})`,
          transformOrigin: "0 0",
        }}
      >
        <svg className="connections-layer">
          {connectionLines.map((line) => (
            <g key={line.id} style={{ pointerEvents: "auto", cursor: "pointer" }}>
              <title>Click to remove connection</title>
              <line
                x1={line.x1} y1={line.y1} x2={line.x2} y2={line.y2}
                stroke={line.color} strokeWidth={3} strokeLinecap="round" opacity={0.7}
              />
              <circle cx={line.x1} cy={line.y1} r={5} fill={line.color} stroke="var(--canvas-bg)" strokeWidth={1.5} />
              <circle cx={line.x2} cy={line.y2} r={5} fill={line.color} stroke="var(--canvas-bg)" strokeWidth={1.5} />
              <line
                x1={line.x1} y1={line.y1} x2={line.x2} y2={line.y2}
                stroke="transparent" strokeWidth={12} style={{ cursor: "pointer" }}
                onClick={(e) => { e.stopPropagation(); deleteConnection(line.id); }}
              />
            </g>
          ))}
        </svg>

        {notes.map((note) => (
          <StickyNote
            key={note.id}
            note={note}
            scale={scale}
            isPinned={pinnedNoteId === note.id}
            onUpdate={updateNote}
            onColorChange={changeColor}
            onDelete={deleteNote}
            onCopyText={copyText}
            onBringToFront={bringToFront}
            onPin={handlePin}
            onOpenFile={openFileManager}
          />
        ))}
      </div>

      <div className="toolbar">
        <button
          className={`toolbar-btn ${otgStatus.running ? "otg-active" : ""}`}
          onClick={toggleOtg}
          title={otgStatus.running ? "Stop mobile sharing" : "Share on mobile"}
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <rect x="5" y="2" width="14" height="20" rx="2" />
            <line x1="12" y1="18" x2="12.01" y2="18" />
          </svg>
        </button>
        <button className="toolbar-btn" onClick={cycleTheme} title={`Theme: ${theme}`}>
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            {theme === "light" && (
              <circle cx="12" cy="12" r="5" />
            )}
            {theme === "dark" && (
              <path d="M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z" />
            )}
            {theme === "contrast" && (
              <>
                <circle cx="12" cy="12" r="5" />
                <line x1="12" y1="1" x2="12" y2="3" />
                <line x1="12" y1="21" x2="12" y2="23" />
                <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
                <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
                <line x1="1" y1="12" x2="3" y2="12" />
                <line x1="21" y1="12" x2="23" y2="12" />
                <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" />
                <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
              </>
            )}
          </svg>
        </button>
        <div className="conn-color-picker" style={{ position: "relative" }}>
          <button
            className="toolbar-btn conn-color-btn"
            onClick={() => setConnColorOpen(!connColorOpen)}
            title="Connection color"
            style={{ background: connColor, border: "2px solid var(--canvas-bg)", boxShadow: "0 2px 8px rgba(0,0,0,0.15)" }}
          />
          {connColorOpen && (
            <div className="color-picker-popup">
              {CONNECTION_COLORS.map((c) => (
                <button
                  key={c}
                  className="color-swatch-btn"
                  style={{
                    background: c,
                    border: connColor === c ? "2px solid var(--note-text)" : "2px solid transparent",
                    boxShadow: connColor === c ? "0 0 0 2px var(--canvas-bg)" : "none",
                  }}
                  onClick={() => { setConnColor(c); setConnColorOpen(false); }}
                />
              ))}
            </div>
          )}
        </div>
        <button className="toolbar-btn" onClick={addManualNote} title="Add sticky note">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <line x1="12" y1="5" x2="12" y2="19" />
            <line x1="5" y1="12" x2="19" y2="12" />
          </svg>
        </button>
        <div className="zoom-label">{Math.round(scale * 100)}%</div>
      </div>

      <div className="hint">
        Ctrl+Scroll to zoom  |  Middle-click or Alt+drag to pan  |  Double-click note for new color
        <br />
        {pinnedNoteId
          ? "Now click another note to connect them  |  Click canvas to cancel"
          : "Click the star pin to connect notes  |  Clipboard is watched — copy anything"}
      </div>

      {otgPanelOpen && otgStatus.running && (
        <div className="otg-panel">
          <button className="otg-close" onClick={() => setOtgPanelOpen(false)}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
          <h3>Mobile Access</h3>
          {qrDataUrl && <img src={qrDataUrl} alt="QR Code" className="otg-qr" />}
          <div className="otg-code">Pairing code: <strong>{otgStatus.pairing_code}</strong></div>
          <div className="otg-url">{otgStatus.url}</div>
          <p style={{ fontSize: 11, color: "#888", margin: "8px 0" }}>
            Open this URL on your phone and enter the pairing code.
            <br />Add to Home Screen for a full-screen app experience.
          </p>
          <button className="otg-scan-btn" onClick={scanForPeers} disabled={peersLoading}>
            {peersLoading ? "Scanning..." : "Scan for other OTG servers"}
          </button>
          {peers.length > 0 && (
            <div className="otg-peers">
              <p style={{ fontSize: 11, marginBottom: 4 }}>Peers found:</p>
              {peers.map((p, i) => (
                <div key={i} className="otg-peer">
                  <span className="otg-peer-ip">{p.ip}:{p.port}</span>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
