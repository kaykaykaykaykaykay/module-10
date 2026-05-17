## Tutorial 2.1
![alt text](<Screenshot 2026-05-18 015326.png>)![alt text](<Screenshot 2026-05-18 014704.png>) ![alt text](<Screenshot 2026-05-18 014749.png>) ![alt text](<Screenshot 2026-05-18 014736.png>)

When the server starts, it listens on port 2000 for incoming WebSocket connections. Each time a client connects, the server prints the client's address and sends a welcome message to that client.
When a client types a message and presses enter, the message is sent to the server via the WebSocket connection. The server then broadcasts that message to all connected clients, so every client sees what every other client typed.
This works asynchronously, the server handles multiple clients at the same time without blocking. This is done using tokio::select! which listens for incoming messages from the client and messages from the broadcast channel. When either arrives, it handles it immediately without waiting for the other.
The broadcast channel (bcast_tx and bcast_rx) connects all the clients, when any client sends a message, the server puts it into the broadcast channel, and every connected client's handler receives it and forwards it to their respective client.

## Tutorial 2.2
server.rs and client.rs. In server.rs, the bind address was changed from 127.0.0.1:2000 to 127.0.0.1:8080. In client.rs, the connection URI was changed from ws://127.0.0.1:2000 to ws://127.0.0.1:8080.  both sides server and client must use the same port. The server defines which port it listens on, and the client must connect to that exact same port. The ws:// prefix is the WebSocket protocol, similar to how http:// or https:// are protocols for web browsing.

## Tutorial 2.3
![alt text](<Screenshot 2026-05-18 025649.png>) ![alt text](<Screenshot 2026-05-18 025627.png>)![alt text](<Screenshot 2026-05-18 025701.png>)
In server.rs, the broadcast message was modified to include the sender's IP address and port number. Previously the server just forwarded the raw text message. Now it formats the message as {addr}: {text} before broadcasting it to all clients. When a client connects, the server's TcpListener::accept() returns both the socket and the SocketAddr of the connecting client. This addr is then passed into the handle_connection function, where it is used to format the broadcast message.