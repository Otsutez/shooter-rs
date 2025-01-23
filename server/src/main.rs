use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:1234")?;
    let mut player_1 = PlayerPos::new_player_1();
    println!("Game server started on local port 1234.");

    println!("Waiting for connection from player...");

    for stream in listener.incoming() {
        println!("Accepted new connection!");
        match stream {
            Ok(mut s) => {
                // Send data initially
                player_1.write_pos(&mut s).expect("Send player position");

                // Get player position
                while let Ok((n1, n2)) = player_1.read_pos(&mut s) {
                    if n1 == 0 || n2 == 0 {
                        break;
                    }
                    println!("player_1 x: {}", player_1.x);
                    println!("player_1 y: {}", player_1.y);
                }
            }
            Err(_) => println!("Error getting new connection"),
        }
    }
    Ok(())
}

struct PlayerPos {
    x: f32,
    y: f32,
}

impl PlayerPos {
    fn new_player_1() -> Self {
        PlayerPos { x: 0.0, y: 18.0 }
    }

    fn new_player_2() -> Self {
        PlayerPos { x: 0.0, y: -18.0 }
    }

    fn write_pos(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        stream.write(&self.x.to_be_bytes())?;
        stream.write(&self.y.to_be_bytes())?;
        Ok(())
    }

    fn read_pos(&mut self, stream: &mut TcpStream) -> std::io::Result<(usize, usize)> {
        let mut x: [u8; 4] = [0; 4];
        let mut y: [u8; 4] = [0; 4];
        let n1 = stream.read(&mut x)?;
        let n2 = stream.read(&mut y)?;
        self.x = f32::from_be_bytes(x);
        self.y = f32::from_be_bytes(y);
        Ok((n1, n2))
    }
}
