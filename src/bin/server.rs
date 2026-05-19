use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_websockets::{Message, ServerBuilder, WebSocketStream};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct WsMessage {
    message_type: String,
    data: Option<String>,
    data_array: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ShowerThought {
    from: String,
    text: String,
    likes: u32,
}

#[derive(Serialize)]
struct ChatMessage {
    from: String,
    message: String,
    time: u64,
}


struct UserEntry {
    nick: String,
    tx: mpsc::UnboundedSender<String>,
}

struct State {
    users: Vec<UserEntry>,
    message_history: Vec<String>,
    current_thought: Option<ShowerThought>,
}

impl State {
    fn new() -> Self {
        State {
            users: Vec::new(),
            message_history: Vec::new(),
            current_thought: None,
        }
    }

    fn broadcast(&self, msg: &str) {
        for user in &self.users {
            let _ = user.tx.send(msg.to_string());
        }
    }

    fn make_users_msg(&self) -> String {
        let nicks: Vec<String> = self.users.iter().map(|u| u.nick.clone()).collect();
        serde_json::to_string(&WsMessage {
            message_type: "users".to_string(),
            data: None,
            data_array: Some(nicks),
        })
        .unwrap()
    }

    fn make_thought_update_msg(&self) -> Option<String> {
        self.current_thought.as_ref().map(|t| {
            serde_json::to_string(&WsMessage {
                message_type: "thoughtupdate".to_string(),
                data: Some(serde_json::to_string(t).unwrap()),
                data_array: None,
            })
            .unwrap()
        })
    }
}

type SharedState = Arc<Mutex<State>>;


async fn handle_connection(
    _addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    state: SharedState,
    mut rx: mpsc::UnboundedReceiver<String>,
    tx: mpsc::UnboundedSender<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut nick: Option<String> = None;

    loop {
        tokio::select! {

            msg = rx.recv() => {
                match msg {
                    Some(text) => ws_stream.send(Message::text(text)).await?,
                    None => break,
                }
            }
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(msg)) => {
                        let text = match msg.as_text() {
                            Some(t) => t,
                            None => continue,
                        };
                        let parsed: WsMessage = match serde_json::from_str(text) {
                            Ok(p) => p,
                            Err(_) => continue,
                        };

                        match parsed.message_type.as_str() {
                            "register" => {
                                if let Some(name) = parsed.data {
                                    nick = Some(name.clone());
                                    let mut st = state.lock().unwrap();
                                    st.users.push(UserEntry { nick: name, tx: tx.clone() });

                                    let users_msg = st.make_users_msg();
                                    st.broadcast(&users_msg);

                                    for h in &st.message_history {
                                        let _ = tx.send(h.clone());
                                    }

                                    if let Some(thought_msg) = st.make_thought_update_msg() {
                                        let _ = tx.send(thought_msg);
                                    }
                                }
                            }

                            "message" => {
                                if let (Some(text), Some(sender)) = (parsed.data, nick.as_ref()) {
                                    let ts = SystemTime::now()
                                        .duration_since(UNIX_EPOCH)
                                        .unwrap()
                                        .as_millis() as u64;
                                    let chat = ChatMessage {
                                        from: sender.clone(),
                                        message: text,
                                        time: ts,
                                    };
                                    let broadcast_msg = serde_json::to_string(&WsMessage {
                                        message_type: "message".to_string(),
                                        data: Some(serde_json::to_string(&chat).unwrap()),
                                        data_array: None,
                                    })
                                    .unwrap();
                                    let mut st = state.lock().unwrap();
                                    st.message_history.push(broadcast_msg.clone());
                                    st.broadcast(&broadcast_msg);
                                }
                            }

                            "showerthought" => {
                                if let (Some(text), Some(sender)) = (parsed.data, nick.as_ref()) {
                                    let thought = ShowerThought {
                                        from: sender.clone(),
                                        text,
                                        likes: 0,
                                    };
                                    let mut st = state.lock().unwrap();
                                    st.current_thought = Some(thought);
                                    if let Some(thought_msg) = st.make_thought_update_msg() {
                                        st.broadcast(&thought_msg);
                                    }
                                }
                            }

                            "likethought" => {
                                let mut st = state.lock().unwrap();
                                if let Some(thought) = &mut st.current_thought {
                                    thought.likes += 1;
                                }
                                if let Some(thought_msg) = st.make_thought_update_msg() {
                                    st.broadcast(&thought_msg);
                                }
                            }

                            _ => {}
                        }
                    }
                    Some(Err(err)) => return Err(err.into()),
                    None => break,
                }
            }
        }
    }

    if let Some(name) = nick {
        let mut st = state.lock().unwrap();
        st.users.retain(|u| u.nick != name);
        let users_msg = st.make_users_msg();
        st.broadcast(&users_msg);
    }

    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let state: SharedState = Arc::new(Mutex::new(State::new()));
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Listening on port 8080");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {addr:?}");
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            let (tx, rx) = mpsc::unbounded_channel::<String>();
            let (_req, ws_stream) = ServerBuilder::new().accept(socket).await?;
            handle_connection(addr, ws_stream, state, rx, tx).await
        });
    }
}
