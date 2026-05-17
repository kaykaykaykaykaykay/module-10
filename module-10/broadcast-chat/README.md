## Tutorial 2.1
![alt text](<Screenshot 2026-05-18 015326.png>)![alt text](<Screenshot 2026-05-18 014704.png>) ![alt text](<Screenshot 2026-05-18 014749.png>) ![alt text](<Screenshot 2026-05-18 014736.png>)

When the server starts, it listens on port 2000 for incoming WebSocket connections. Each time a client connects, the server prints the client's address and sends a welcome message to that client.
When a client types a message and presses enter, the message is sent to the server via the WebSocket connection. The server then broadcasts that message to all connected clients, so every client sees what every other client typed.
This works asynchronously, the server handles multiple clients at the same time without blocking. This is done using tokio::select! which listens for incoming messages from the client and messages from the broadcast channel. When either arrives, it handles it immediately without waiting for the other.
The broadcast channel (bcast_tx and bcast_rx) connects all the clients, when any client sends a message, the server puts it into the broadcast channel, and every connected client's handler receives it and forwards it to their respective client.