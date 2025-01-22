use serde::{Deserialize, Serialize};
use serde_json;
use std::net::TcpListener;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:1234")?;
    let player_1 = PlayerPacket { x: 0.0, y: 18.0 };
    println!("Game server started on local port 1234");

    println!("Waiting for connection from player...");

    for stream in listener.incoming() {
        println!("Accepted new connection!");
        match stream {
            Ok(s) => match serde_json::to_writer(s, &player_1) {
                Ok(_) => println!("Sent first player location"),
                Err(_) => println!("Failed to send first player location"),
            },
            Err(_) => todo!(),
        }
    }
    // let (socket, addr) = listener.accept()?;
    // println!("First player connected!");
    // println!("First player's address: {addr}");

    // match serde_json::to_writer(socket, &player_1) {
    //     Ok(_) => println!("Sent first player location"),
    //     Err(_) => println!("Failed to send first player location"),
    // }
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct PlayerPacket {
    x: f32,
    y: f32,
}
