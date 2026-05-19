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

