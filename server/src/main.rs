use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::{thread, time};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:1234")?;
    let mut player_1 = PlayerPos::new_player_1();
    let mut player_2 = PlayerPos::new_player_2();
    println!("Game server started on local port 1234.");

    println!("Waiting for connection from player...");

    for s1 in listener.incoming() {
        let mut s1 = s1.expect("Error accepting connection");
        println!("Connection from {}", s1.peer_addr().unwrap());
        player_1
            .write_pos(&mut s1)
            .expect("Send player 1 initial position failed");
        println!("Sent player 1 initial position");

        // Wait for connection from player 2
        let (mut s2, addr) = listener.accept().expect("Error accepting connection");
        println!("Connection from {}", addr);

        player_2
            .write_pos(&mut s2)
            .expect("Send player 2 initial position failed");
        println!("Sent player 2 initial position");

        // Send enemy position
        // This will make the client go into countdown state
        player_2
            .write_pos(&mut s1)
            .expect("Send enemy position to connection 1 failed");
        player_1
            .write_pos(&mut s2)
            .expect("Send enemy position to connection 2 failed");

        // Countdown
        s1.write(b"3").expect("Sending time failed");
        s2.write(b"3").expect("Sending time failed");
        thread::sleep(time::Duration::from_secs(1));
        s1.write(b"2").expect("Sending time failed");
        s2.write(b"2").expect("Sending time failed");
        thread::sleep(time::Duration::from_secs(1));
        s1.write(b"1").expect("Sending time failed");
        s2.write(b"1").expect("Sending time failed");
        thread::sleep(time::Duration::from_secs(1));

        // Send 0 to signal start of game
        s1.write(b"0").expect("Sending time failed");
        s2.write(b"0").expect("Sending time failed");

        // Send players current position and receive their next position
        loop {
            // Send enemies
            player_1
                .write_pos(&mut s2)
                .expect("Sending enemy to s2 failed");
            player_2
                .write_pos(&mut s1)
                .expect("Sending enemy to s1 failed");

            // Receive players
            player_1
                .read_pos(&mut s1)
                .expect("Reading player_1 pos failed");
            player_2
                .read_pos(&mut s2)
                .expect("Reading player_2 pos failed");
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
