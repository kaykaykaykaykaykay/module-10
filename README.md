## Tutorial 3.1
![alt text](<Screenshot 2026-05-19 101616.png>)

## Tutorial 3.2
### History
![alt text](<Screenshot 2026-05-19 113539.png>)

User "late" joins later in the conversation

![alt text](<Screenshot 2026-05-19 113644.png>)

Chats from previous users comes up even after joining late

Server stores messages as they happen:

let messageHistory: string[] = [];  // grows over time
Every time someone sends a chat message, the server pushes the already-serialized JSON string into messageHistory before broadcasting it (app.ts:49).

New user gets the history replayed after registering:

case 'register':
    users.push({ ws, nick: parsed_data.data, isAlive: true });
    broadcast(...users list...); // 1. tell everyone the new user joined
    messageHistory.forEach((msg) => ws.send(msg)); // 2. send past messages to new user only
Users list goes first so the Yew client populates self.users, then history arrives and the client can find each message's sender without panicking. On the Yew client side, the replayed messages go through exactly the same HandleMsg -> MsgTypes::Message path as live messages. The client has no idea they're historical, it just appends them to self.messages and re-renders. The history is in-memory only, so it resets every time the server restarts. There's also no cap on messageHistory, so it grows forever while the server is running.

### Shower Thoughts
![alt text](<Screenshot 2026-05-19 115758.png>)
You type in the share input and click "Share" → Msg::SubmitThought fires → sends { messageType: "showerthought", data: "your text" } over WebSocket → server finds your username from the users array, overwrites currentThought with { from, text, likes: 0 }, and broadcasts thoughtupdate to every connected client → each client's Thoughtupdate handler updates self.current_thought and re-renders the card. Click 👍 → Msg::LikeThought fires → sends { messageType: "likethought" } → server increments currentThought.likes and broadcasts thoughtupdate again → all clients re-render with the new count simultaneously. After they send register, the server first broadcasts the users list, then sends them the thoughtupdate for currentThought directly (not broadcast, just to that socket). So they immediately see whatever thought and like count is currently active. currentThought lives in the server's memory, so it survives users disconnecting and reconnecting, but resets when the server process restarts, same as message history.

## Bonus
![alt text](<Screenshot 2026-05-19 121456.png>)

The original Tutorial 2 server forwarded raw text with addr: message. YewChat speaks a JSON envelope over that same single WebSocket text frame, the JSON is just serialized into one string before sending, exactly like plain text, only structured. So the transport layer didn't change at all; only what goes inside the string changed.

The structural change was the broadcast architecture. Tutorial 2 used a single Tokio broadcast::channel where every connection subscribed to it and received every message identically. That works for identical payloads, but here we need to send different messages to different clients history replay go only to the newly registered user, not everyone. So we replaced it with Arc<Mutex<State>> shared across tasks, where each connection has its own mpsc::unbounded_channel. Broadcasts iterate the users list and push into each connection's private sender. Private messages go directly to one sender. The Mutex is never held across an .await, so there's no deadlock risk.