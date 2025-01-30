# Shooter-rs

This is a rewrite of my [C++ game](https://github.com/Otsutez/shooter) in Rust.
The game now supports 2 players. Rust's raylib binding is used to create graphics
while serde and bincode are used for networking. 

To play this game first start the server.
```
cargo run --bin server
```

Then start the client.
```
cargo run --bin client
```

By default the server will listen on port 1234. Put in the socket address of the 
server in the lobby of the game. For example `127.0.0.1:1234`.

The game will wait for another player to connect and then begin the game.
