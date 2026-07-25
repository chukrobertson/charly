# Roadmap

## Phase 1 — Local (done)

- [x] Infinite corkboard canvas (pan, zoom)
- [x] Multicolored sticky notes (drag, resize, 8 color presets)
- [x] System clipboard watcher (text, images, file URIs)
- [x] File notes — click to open in file manager, copy path
- [x] String connections between notes with colored pins
- [x] System tray — hide on close, always-on clipboard watch
- [x] Light / dark / high-contrast themes
- [x] SQLite persistence (notes + connections survive restarts)
- [x] Cross-platform via Tauri (Linux, macOS, Windows)

## Phase 2 — Network clipboard sync

Goal: multiple machines on the same LAN or VPN share a single workspace. Anything copied on machine A appears on machine B and vice versa.

- [ ] mDNS / Bonjour discovery — peers auto-find each other on the local network
- [ ] Peer-to-peer encrypted transport (libp2p or QUIC)
- [ ] Real-time canvas sync — notes, moves, resizes, color changes propagate
- [ ] Shared clipboard bridge — clip on one machine, paste on another
- [ ] Conflict-free sync via CRDT (automerge or y.js)
- [ ] Optional VPN mode — manually specify peer IPs for connections over WireGuard / Tailscale
- [ ] Peer presence indicator — show which machines are connected on the canvas
- [ ] Note authorship — see which machine added which note

### Architecture sketch

```
┌──────────┐  mDNS discover  ┌──────────┐
│ Machine A │◄──────────────►│ Machine B │
│ (desktop) │  QUIC encrypted │ (laptop)  │
│           │◄──────────────►│           │
└─────┬─────┘  CRDT sync     └─────┬─────┘
      │                            │
      └──────────┬─────────────────┘
                 │
          ┌──────┴──────┐
          │  Machine C   │
          │  (remote via │
          │   VPN IP)    │
          └─────────────┘
```

Each peer runs a lightweight embedded server. Clipboard changes and note mutations are broadcast as CRDT operations. The canvas is eventually consistent — last-write-wins for simple fields, CRDT merge for text content.

## Phase 3 — Mobile companion

Goal: one-click OTG server that serves the workspace as a mobile-friendly web view. Your phone becomes a read/write window into the shared board.

- [ ] Embedded HTTP server in the Tauri app (actix-web or axum)
- [ ] One-click "Share on mobile" button in the toolbar
- [ ] Generates a QR code for instant phone connection
- [ ] Mobile-optimized web UI (touch gestures, responsive layout)
- [ ] Mobile clipboard bridge — copy on phone, appears on desktop board and vice versa
- [ ] Works over LAN (same Wi-Fi) or via VPN when remote
- [ ] No app install required on mobile — works in any browser
- [ ] Optional PWA manifest for "Add to Home Screen" experience
- [ ] Authentication via a one-time pairing code shown on desktop

### Architecture sketch

```
┌──────────┐                    ┌──────────┐
│ Desktop   │  HTTP + WebSocket │  Mobile   │
│ (Tauri)   │◄────────────────►│  Browser  │
│           │   on LAN :8080   │           │
└──────────┘                    └──────────┘

Desktop runs embedded server.
Mobile connects via QR code (URL with token).
Canvas state synced over WebSocket.
Clipboard bridge: copy on mobile → desktop clipboard → all peers.
```

## Phase 4 — Local LLM assistant

Goal: a local language model (running fully offline) that understands your workspace — making meaning from notes, connections, and clipboard history. Acts as a background intelligence layer, not a chatbot.

- [ ] Ollama integration — connect to a locally running instance (llama.cpp compatible)
- [ ] Configurable model selection — choose from any Ollama-pulled model
- [ ] **Meaning from connections** — given a cluster of string-connected notes, generate a one-paragraph summary of the idea they represent together. Display as a floating "insight card" on the connection midpoint
- [ ] **Clipboard history analysis** — periodically scan recent clipboard captures and surface patterns: recurring topics, copied URLs from the same domains, repeated phrases
- [ ] **Auto-tagging** — generate 2–4 short tags per note based on content. Display tags as small chips on each note
- [ ] **Semantic search** — natural language query across all notes ("find notes about database design"). Uses local embeddings via the same model, no cloud dependency
- [ ] **Workspace cleanup** — suggest merging near-duplicate notes, flag stale/isolated notes for archival, propose reorganizing clusters into logical groups
- [ ] **Daily digest** — optional end-of-day summary of what you captured, grouped by inferred topic
- [ ] **OCR via LLM** — multimodal models can read text from clipped screenshots and populate the note with the extracted text
- [ ] **Ambient, not intrusive** — suggestions appear as subtle decorators on the canvas. You approve, dismiss, or ignore. Nothing auto-modifies your board
- [ ] Privacy-first: everything runs locally. No API keys, no telemetry, no network calls for inference

### Architecture sketch

```
┌─────────────┐     localhost:11434     ┌──────────────┐
│   Charly    │◄──────────────────────►│   Ollama     │
│  (Tauri)    │    HTTP + JSON          │  (llama.cpp) │
│             │                         │              │
│  ┌────────┐ │   POST /api/generate    │  llama3.2    │
│  │ LLM    │ │   POST /api/embeddings  │  mistral     │
│  │ plugin │ │                         │  gemma       │
│  └────────┘ │                         │  ...         │
└─────────────┘                         └──────────────┘

Charly's LLM plugin runs optionally.
User controls: model, frequency, which features are active.
All prompts run as background tasks — non-blocking.
Results rendered as suggestions, not mutations.
```

### Key prompts

| Task | Prompt shape |
|---|---|
| Cluster summary | "You are analyzing a mind map. These {n} notes are connected. Each note says: ... Summarize what this cluster of ideas represents in 2–3 sentences." |
| Auto-tag | "Generate 2–4 single-word tags for this note: {content}. Return as comma-separated list." |
| Semantic search | Embed query + all notes → cosine similarity → top-k results ranked |
| Dedup detection | "Are these two notes essentially the same idea? Note A: ... Note B: ... Reply YES or NO with brief reason." |
| Daily digest | "Here are {n} notes captured today across {m} clusters. Group them by topic and provide a 2-sentence summary per group." |

## Future ideas (post-Phase 4)

- [ ] Web clipper browser extension — grab snippets, screenshots, bookmarks
- [ ] Custom note shapes (circles, arrows, freehand drawing)
- [ ] Calendar / timeline mode for time-based note arrangement
- [ ] Plugin system — user-contributed note types and integrations
- [ ] Export board as SVG / PDF / Markdown
- [ ] Multi-user collaboration with presence cursors
