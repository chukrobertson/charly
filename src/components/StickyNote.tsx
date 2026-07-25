import { useState, useRef, useCallback, useEffect, useMemo } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { Note } from "../types";
import { NOTE_COLORS, NOTE_SHADOWS } from "../types";

interface Props {
  note: Note;
  scale: number;
  isPinned: boolean;
  onUpdate: (id: string, x: number, y: number, width: number, height: number, zIndex: number) => void;
  onColorChange: (id: string, color: string) => void;
  onDelete: (id: string) => void;
  onCopyText: (text: string) => void;
  onBringToFront: (id: string, zIndex: number) => void;
  onPin: (id: string) => void;
  onOpenFile: (path: string) => void;
}

export default function StickyNote({
  note,
  scale,
  isPinned,
  onUpdate,
  onColorChange,
  onDelete,
  onCopyText,
  onBringToFront,
  onPin,
  onOpenFile,
}: Props) {
  const [dragging, setDragging] = useState(false);
  const [resizing, setResizing] = useState(false);
  const dragRef = useRef<{ ox: number; oy: number; mx: number; my: number } | null>(null);
  const resizeRef = useRef<{ ow: number; oh: number; mx: number; my: number } | null>(null);
  const [textContent, setTextContent] = useState<string | null>(null);

  useEffect(() => {
    if (note.content_type === "text") {
      setTextContent(note.content_ref ?? "");
    }
  }, [note.content_ref, note.content_type]);

  const imageUrl = useMemo(() => {
    if (note.content_type === "image" && note.content_ref) {
      return convertFileSrc(note.content_ref);
    }
    return null;
  }, [note.content_type, note.content_ref]);

  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      if ((e.target as HTMLElement).closest(".note-resize, .note-color, .note-delete, .note-copy, .note-pin, .note-open")) return;
      e.stopPropagation();
      setDragging(true);
      dragRef.current = { ox: note.x, oy: note.y, mx: e.clientX, my: e.clientY };
      onBringToFront(note.id, Date.now());
    },
    [note.x, note.y, note.id, onBringToFront]
  );

  const handleResizeDown = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      e.preventDefault();
      setResizing(true);
      resizeRef.current = { ow: note.width, oh: note.height, mx: e.clientX, my: e.clientY };
    },
    [note.width, note.height]
  );

  useEffect(() => {
    if (!dragging && !resizing) return;

    const handleMouseMove = (e: MouseEvent) => {
      if (dragging && dragRef.current) {
        const dx = (e.clientX - dragRef.current.mx) / scale;
        const dy = (e.clientY - dragRef.current.my) / scale;
        const nx = dragRef.current.ox + dx;
        const ny = dragRef.current.oy + dy;
        onUpdate(note.id, nx, ny, note.width, note.height, note.z_index);
      }
      if (resizing && resizeRef.current) {
        const dx = (e.clientX - resizeRef.current.mx) / scale;
        const dy = (e.clientY - resizeRef.current.my) / scale;
        const nw = Math.max(120, resizeRef.current.ow + dx);
        const nh = Math.max(100, resizeRef.current.oh + dy);
        onUpdate(note.id, note.x, note.y, nw, nh, note.z_index);
      }
    };

    const handleMouseUp = () => {
      setDragging(false);
      setResizing(false);
      dragRef.current = null;
      resizeRef.current = null;
    };

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, [dragging, resizing, scale, note.id, note.x, note.y, note.width, note.height, note.z_index, onUpdate]);

  const cycleColor = useCallback(() => {
    const idx = NOTE_COLORS.indexOf(note.color);
    const next = NOTE_COLORS[(idx + 1) % NOTE_COLORS.length];
    onColorChange(note.id, next);
  }, [note.id, note.color, onColorChange]);

  const shadowColor = NOTE_SHADOWS[note.color as keyof typeof NOTE_SHADOWS] || "rgba(0,0,0,0.15)";

  return (
    <div
      className={`sticky-note ${isPinned ? "note-pinned" : ""}`}
      style={{
        left: note.x,
        top: note.y,
        width: note.width,
        height: note.height,
        backgroundColor: note.color,
        boxShadow: isPinned
          ? `0 0 0 3px #333, 2px 3px 8px ${shadowColor}`
          : `2px 3px 8px ${shadowColor}`,
        zIndex: note.z_index,
        cursor: dragging ? "grabbing" : "grab",
        outline: isPinned ? "3px solid #333" : undefined,
      }}
      onMouseDown={handleMouseDown}
      onDoubleClick={(e) => {
        e.stopPropagation();
        cycleColor();
      }}
    >
      <div className="note-toolbar">
        <button
          className={`note-pin ${isPinned ? "note-pin-active" : ""}`}
          title={isPinned ? "Selected for connection — click another note" : "Pin to connect to another note"}
          onClick={(e) => { e.stopPropagation(); onPin(note.id); }}
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill={isPinned ? "currentColor" : "none"} stroke="currentColor" strokeWidth="2">
            <path d="M12 2L15.09 8.26L22 9.27L17 14.14L18.18 21.02L12 17.77L5.82 21.02L7 14.14L2 9.27L8.91 8.26L12 2Z" />
          </svg>
        </button>
        <button
          className="note-color"
          title="Change color (double-click also works)"
          onClick={(e) => { e.stopPropagation(); cycleColor(); }}
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <circle cx="12" cy="12" r="10" />
          </svg>
        </button>
        <button
          className="note-copy"
          title="Copy content"
          onClick={(e) => {
            e.stopPropagation();
            if (note.content_type === "text") {
              onCopyText(textContent || "");
            } else if (note.content_ref) {
              onCopyText(note.content_ref);
            }
          }}
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <rect x="9" y="9" width="13" height="13" rx="2" />
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
          </svg>
        </button>
        {note.content_type === "file" && note.content_ref && (
          <button
            className="note-open"
            title="Open in file manager"
            onClick={(e) => { e.stopPropagation(); onOpenFile(note.content_ref!); }}
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
            </svg>
          </button>
        )}
        <button
          className="note-delete"
          title="Delete note"
          onClick={(e) => { e.stopPropagation(); onDelete(note.id); }}
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>

      <div className="note-content">
        {note.content_type === "text" && (
          <textarea
            className="note-textarea"
            value={textContent ?? ""}
            onChange={(e) => setTextContent(e.target.value)}
            placeholder="Type something..."
            onClick={(e) => e.stopPropagation()}
            onMouseDown={(e) => e.stopPropagation()}
            style={{ background: "transparent" }}
          />
        )}
        {note.content_type === "image" && imageUrl && (
          <div className="note-image-wrapper">
            <img
              src={imageUrl}
              alt="clipped"
              className="note-image"
              draggable={false}
              onClick={(e) => e.stopPropagation()}
              onMouseDown={(e) => e.stopPropagation()}
            />
          </div>
        )}
        {note.content_type === "file" && note.file_name && (
          <div
            className="note-file"
            onClick={(e) => {
              e.stopPropagation();
              if (note.content_ref) onOpenFile(note.content_ref);
            }}
            title="Click to open in file manager"
          >
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
              <polyline points="14 2 14 8 20 8" />
            </svg>
            <div className="note-file-info">
              <span className="note-filename">{note.file_name}</span>
              <span className="note-file-path">{note.content_ref}</span>
            </div>
          </div>
        )}
      </div>

      <div
        className="note-resize"
        onMouseDown={handleResizeDown}
      />
    </div>
  );
}
