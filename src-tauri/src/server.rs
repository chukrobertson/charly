use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::Query,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

use crate::db::Database;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub pairing_code: Arc<Mutex<String>>,
    pub tx: broadcast::Sender<String>,
}

pub async fn generate_pairing_code(state: &AppState) -> String {
    use rand::Rng;
    let code = format!("{:06}", rand::thread_rng().gen_range(0..1_000_000));
    let mut pc = state.pairing_code.lock().await;
    *pc = code.clone();
    code
}

pub async fn start_server(
    db: Arc<Database>,
    port: u16,
) -> Result<(String, tokio::task::JoinHandle<()>), Box<dyn std::error::Error + Send + Sync>> {
    let (tx, _rx) = broadcast::channel::<String>(100);
    let state = AppState {
        db,
        pairing_code: Arc::new(Mutex::new(String::new())),
        tx: tx.clone(),
    };

    let pairing_code = generate_pairing_code(&state).await;
    let state = Arc::new(state);

    let app = Router::new()
        .route("/", get(serve_mobile_ui))
        .route("/api/health", get(|| async { "ok" }))
        .route("/ws", get(ws_handler))
        .route("/api/notes", get(get_notes_handler))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    let local_addr = listener.local_addr()?.to_string();

    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });

    Ok((pairing_code, handle))
}

async fn serve_mobile_ui() -> Html<&'static str> {
    Html(MOBILE_HTML)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<HashMap<String, String>>,
    state: axum::extract::State<Arc<AppState>>,
) -> impl IntoResponse {
    let code = params.get("code").cloned().unwrap_or_default();
    let stored = state.pairing_code.lock().await.clone();

    if code != stored {
        return Err((
            axum::http::StatusCode::UNAUTHORIZED,
            "Invalid pairing code",
        ));
    }

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state.0)))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.tx.subscribe();

    loop {
        tokio::select! {
            msg = rx.recv() => {
                if let Ok(text) = msg {
                    if socket.send(Message::Text(text.into())).await.is_err() {
                        break;
                    }
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        // Broadcast to all clients
                        let _ = state.tx.send(text.to_string());
                    }
                    Some(Ok(Message::Close(_))) => break,
                    _ => break,
                }
            }
        }
    }
}

async fn get_notes_handler(
    state: axum::extract::State<Arc<AppState>>,
) -> axum::Json<serde_json::Value> {
    let notes = state.db.get_notes().unwrap_or_default();
    axum::Json(serde_json::to_value(notes).unwrap_or_default())
}

const MOBILE_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=no">
<meta name="apple-mobile-web-app-capable" content="yes">
<meta name="apple-mobile-web-app-status-bar-style" content="black-translucent">
<meta name="apple-mobile-web-app-title" content="Charly">
<link rel="manifest" href="/manifest.json">
<link rel="apple-touch-icon" href="/icon-192.png">
<title>Charly</title>
<style>
*{margin:0;padding:0;box-sizing:border-box}
html,body{width:100%;height:100%;overflow:hidden;font-family:-apple-system,BlinkMacSystemFont,sans-serif;background:#2a2a2e;color:#fff;touch-action:none}
#canvas{width:100%;height:100%;position:relative;background:#2a2a2e;background-image:radial-gradient(circle,#3a3a3e 1px,transparent 1px);background-size:20px 20px}
.note{position:absolute;border-radius:4px;padding:12px;min-width:120px;min-height:60px;font-size:14px;line-height:1.4;box-shadow:0 2px 8px rgba(0,0,0,0.3);overflow-wrap:break-word}
#auth{position:fixed;top:0;left:0;width:100%;height:100%;display:flex;align-items:center;justify-content:center;background:rgba(0,0,0,0.9);z-index:9999}
#auth input{font-size:32px;text-align:center;width:200px;padding:12px;border:2px solid #555;border-radius:12px;background:#333;color:#fff;letter-spacing:8px}
#auth button{margin-top:12px;padding:12px 32px;font-size:18px;border:none;border-radius:12px;background:#4ECDC4;color:#000;cursor:pointer}
#toolbar{position:fixed;bottom:16px;right:16px;display:flex;gap:8px;z-index:100}
#toolbar button{width:44px;height:44px;border-radius:12px;border:none;background:rgba(255,255,255,0.1);color:#fff;font-size:20px;cursor:pointer}
#status{position:fixed;top:0;left:0;right:0;padding:8px 16px;font-size:12px;color:#888;text-align:center;background:rgba(0,0,0,0.5)}
</style>
</head>
<body>
<div id="status"></div>
<div id="auth">
<div style="text-align:center">
<h2 style="margin-bottom:16px">Enter pairing code</h2>
<input type="text" id="code" maxlength="6" inputmode="numeric" pattern="[0-9]*" autocomplete="off">
<br><button onclick="connect()">Connect</button>
</div>
</div>
<div id="canvas"></div>
<div id="toolbar">
<button onclick="addNote()" title="Add note">+</button>
</div>
<script>
let ws=null,notes=[],scale=1,panX=0,panY=0,code='';
const colors=['#FEF08A','#FECACA','#BFDBFE','#BBF7D0','#DDD6FE','#FED7AA','#FBCFE8','#A5F3FC'];

function connect(){
code=document.getElementById('code').value;
ws=new WebSocket(`ws://${location.host}/ws?code=${code}`);
ws.onopen=()=>{
document.getElementById('auth').style.display='none';
document.getElementById('status').textContent='Connected';
loadNotes();
};
ws.onmessage=e=>{
const data=JSON.parse(e.data);
if(data.type==='notes')renderNotes(data.notes);
if(data.type==='note_added'){notes.push(data.note);renderAll();}
if(data.type==='note_moved')updateNote(data);
};
ws.onclose=()=>{document.getElementById('status').textContent='Disconnected'};
}

function loadNotes(){ws.send(JSON.stringify({type:'get_notes'}));}

function addNote(){
const id=crypto.randomUUID(),x=100+Math.random()*200,y=100+Math.random()*300;
const note={id,x,y,width:180,height:120,color:colors[Math.floor(Math.random()*colors.length)],content:'Tap to type...'};
ws.send(JSON.stringify({type:'note_added',note}));
}

function renderNotes(n){notes=n;renderAll();}

let dragNote=null,dragStart=null;
document.getElementById('canvas').addEventListener('pointerdown',e=>{
if(e.target.closest('button'))return;
const el=e.target.closest('.note');
if(el){
e.preventDefault();
dragNote={id:el.dataset.id,el,x:parseFloat(el.style.left),y:parseFloat(el.style.top)};
dragStart={x:e.clientX,y:e.clientY};
el.style.zIndex=Date.now();
}
});

document.addEventListener('pointermove',e=>{
if(!dragNote)return;
const dx=(e.clientX-dragStart.x)/scale,dy=(e.clientY-dragStart.y)/scale;
dragNote.el.style.left=(dragNote.x+dx)+'px';
dragNote.el.style.top=(dragNote.y+dy)+'px';
});

document.addEventListener('pointerup',()=>{
if(dragNote){
ws.send(JSON.stringify({type:'note_moved',id:dragNote.id,x:parseFloat(dragNote.el.style.left),y:parseFloat(dragNote.el.style.top)}));
dragNote=null;dragStart=null;
}
});

function renderAll(){
document.getElementById('canvas').innerHTML='';
notes.forEach(n=>{
const el=document.createElement('div');
el.className='note';
el.dataset.id=n.id;
el.style.cssText=`left:${n.x}px;top:${n.y}px;width:${n.width}px;height:${n.height}px;background:${n.color};cursor:grab`;
el.innerHTML=`<div style="font-size:10px;margin-bottom:4px;opacity:0.5">${n.content_type}</div><div>${n.content_type==='text'?n.content_ref||'':''}</div>`;
document.getElementById('canvas').appendChild(el);
});
}

function updateNote(d){const n=notes.find(x=>x.id===d.id);if(n){n.x=d.x;n.y=d.y;}}
</script>
</body>
</html>"##;
