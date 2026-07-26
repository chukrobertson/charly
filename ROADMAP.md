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

## Phase 2 — Mobile companion + OTG server

Goal: one machine on your network runs an embedded HTTP server. Your phone (or any browser) connects to see and interact with the board. One-click to start, one-click to stop. Smart about which device should host.

### Core features

- [ ] **Embedded HTTP + WebSocket server** in the Tauri app (axum + tokio-tungstenite)
- [ ] **One-click toggle** in the toolbar — "Share on mobile"
- [ ] **QR code** displayed on screen for instant phone pairing
- [ ] **Mobile-optimized web UI** — touch gestures, responsive layout, same canvas
- [ ] **PWA manifest** — "Add to Home Screen" on iOS/Android, works fullscreen
- [ ] **Mobile clipboard bridge** — copy on phone → appears on host desktop board
- [ ] **Auth via pairing code** — one-time 6-digit code shown on desktop, entered on phone

### Leader election — only one OTG at a time

You install Charly on multiple devices. Only ONE should run the OTG server at any time. When you toggle OTG on a new device:

```
┌─────────────┐                         ┌─────────────┐
│  Machine A   │  "OTG already running" │  Machine B   │
│  (currently  │◄──────────────────────│  (you just   │
│   hosting)   │     mDNS responds      │   toggled    │
│              │     on _charly._tcp    │   OTG on)    │
└─────────────┘                         └─────────────┘
                                               │
                                     ┌─────────┴─────────┐
                                     │  Dialog appears:   │
                                     │                    │
                                     │  "Machine A is      │
                                     │  already hosting.   │
                                     │                     │
                                     │  [Keep on A]  [Move here]" │
                                     └────────────────────┘
```

| Scenario | Behavior |
|---|---|
| No other OTG detected | Start immediately |
| Another OTG detected on LAN | Prompt: keep on remote or move here |
| User chooses "Move here" | Remote OTG shuts down gracefully, this one starts |
| User chooses "Keep on A" | This instance does nothing, shows a link to connect |
| Hosting device goes offline | Any other device can claim OTG after 30s timeout |

### Tailscale / remote access

- [ ] **Bind to Tailscale IP** — server listens on the Tailscale interface (`100.x.x.x`), not just `0.0.0.0`
- [ ] **Tailscale-aware discovery** — check known Tailscale peers for a running OTG, not just mDNS
- [ ] **QR code includes Tailscale URL** — `http://100.x.x.x:8080` so it works from anywhere
- [ ] **Optional: bind to localhost only** — if Tailscale isn't running, fall back to LAN-only
- [ ] **Works over Tailscale MagicDNS** — `http://hostname.tailnet-name.ts.net:8080`

### Architecture sketch

```
                   Tailscale network (100.x.y.z)
┌──────────────┐                              ┌──────────────┐
│  Ubuntu       │  mDNS + Tailscale peer check │  Ubuntu       │
│  ThinkPad     │◄───────────────────────────►│  MacBook      │
│  (OTG host?)  │                              │  (OTG host?)  │
└──────┬───────┘                              └──────┬───────┘
       │                                             │
       │             ┌──────────────┐                │
       │             │  Win11       │                │
       │             │  Desktop     │                │
       │             │  (OTG host?) │                │
       │             └──────┬───────┘                │
       │                    │                        │
       │    Only ONE runs   │                        │
       │    the OTG server  │                        │
       │                    │                        │
       └────────┬───────────┴────────────────────────┘
                │
                │  HTTP :8080 (over Tailscale)
                │
         ┌──────┴──────┐
         │   iPhone     │
         │   Browser    │
         │   (PWA)      │
         └─────────────┘

Leader election protocol:
1. Instance toggles OTG → broadcasts mDNS query (_charly._tcp)
2. Any host responds with its ID, IP, port, uptime
3. If no response → start server immediately
4. If response(s) received → show dialog with options
5. Winner binds to :8080, registers _charly._tcp service
6. Others remain passive, poll mDNS every 5s for liveness
7. Host dies → after 30s silence, any waiting instance can claim
```

## Phase 3 — Network canvas sync

Goal: the full canvas syncs in real-time across machines. Built on the discovery + transport layer from Phase 2. Clip on machine A, it appears on machines B, C, and the mobile view simultaneously.

- [ ] **Reuse Phase 2 transport** — the same axum/WebSocket server handles both mobile clients and peer sync
- [ ] **Real-time canvas sync** — notes, moves, resizes, colors, connections propagate between peers
- [ ] **Shared clipboard bridge** — clip on one machine, appears on all
- [ ] **Conflict-free sync via CRDT** (automerge or y.js) for note content
- [ ] **Peer presence indicator** — show connected machines as icons on the canvas
- [ ] **Note authorship** — see which machine added or last edited each note
- [ ] **Offline resilience** — each machine has its own SQLite. On reconnect, CRDT merges changes

### Architecture sketch

```
                 Tailscale network + LAN
┌──────────┐                            ┌──────────┐
│ Machine A │◄────────────────────────►│ Machine B │
│ (hosting   │  WebSocket CRDT sync     │ (peer)    │
│  OTG +     │                          │           │
│  full      │◄────────────────────────►│           │
│  canvas)   │                          └──────────┘
└─────┬─────┘
      │                          ┌──────────┐
      │◄────────────────────────►│ Machine C │
      │  WebSocket               │ (peer)    │
      │                          └──────────┘
      │
      │  HTTP (mobile UI)
      │
┌─────┴─────┐
│  iPhone    │
│  Browser   │
└───────────┘

The OTG host acts as the sync hub.
All peers connect to it via WebSocket.
Canvas state is a CRDT document — mutations broadcast to all.
Each machine keeps its own SQLite for offline durability.
Mobile browser gets canvas state pushed via the same WebSocket.
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
