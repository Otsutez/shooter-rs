use game_channel::{Channel, ChannelVector2, Packet};
use std::net::{TcpListener, TcpStream};
use std::{thread, time};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:1234")?;
    let mut player_1 = Player::new_player_1();
    let mut player_2 = Player::new_player_2();
    println!("Game server started on local port 1234.");

    println!("Waiting for connection from player...");

    for s1 in listener.incoming() {
        let s1 = s1.expect("Error accepting connection");
        println!("Connection from {}", s1.peer_addr().unwrap());
        let mut c1 = Channel::with_stream(s1);
        player_1
            .write_pos(&mut c1)
            .expect("Send player 1 initial position failed");
        println!("Sent player 1 initial position");

        // Wait for connection from player 2
        let (s2, addr) = listener.accept().expect("Error accepting connection");
        println!("Connection from {}", addr);
        let mut c2 = Channel::with_stream(s2);

        player_2
            .write_pos(&mut c2)
            .expect("Send player 2 initial position failed");
        println!("Sent player 2 initial position");

        // Send enemy position
        // This will make the client go into countdown state
        player_2
            .write_pos(&mut c1)
            .expect("Send enemy position to connection 1 failed");
        player_1
            .write_pos(&mut c2)
            .expect("Send enemy position to connection 2 failed");

        // Countdown
        c1.send(Packet::Time(3)).expect("Sending time failed");
        c2.send(Packet::Time(3)).expect("Sending time failed");
        thread::sleep(time::Duration::from_secs(1));
        c1.send(Packet::Time(2)).expect("Sending time failed");
        c2.send(Packet::Time(2)).expect("Sending time failed");
        thread::sleep(time::Duration::from_secs(1));
        c1.send(Packet::Time(1)).expect("Sending time failed");
        c2.send(Packet::Time(1)).expect("Sending time failed");
        thread::sleep(time::Duration::from_secs(1));

        // Send 0 to signal start of game
        c1.send(Packet::Time(0)).expect("Sending time failed");
        c2.send(Packet::Time(0)).expect("Sending time failed");

        // Send players current position and receive their next position
        loop {
            // Send enemies
            player_2
                .write_pos(&mut c1)
                .expect("Sending enemy to s1 failed");
            player_1
                .write_pos(&mut c2)
                .expect("Sending enemy to s2 failed");

            // Receive players
            player_2
                .read_pos(&mut c2)
                .expect("Reading player_2 pos failed");
            player_1
                .read_pos(&mut c1)
                .expect("Reading player_1 pos failed");
        }
    }
    Ok(())
}

struct Player {
    pos: ChannelVector2,
    target: ChannelVector2,
}

impl Player {
    fn new_player_1() -> Self {
        Player {
            pos: ChannelVector2 { x: 0.0, z: 18.0 },
            target: ChannelVector2 { x: 0.0, z: -1.0 },
        }
    }

    fn new_player_2() -> Self {
        Player {
            pos: ChannelVector2 { x: 0.0, z: -18.0 },
            target: ChannelVector2 { x: 0.0, z: 1.0 },
        }
    }

    fn write_pos(&self, channel: &mut Channel<TcpStream>) -> Result<(), ()> {
        channel
            .send(Packet::Player {
                pos: self.pos,
                target: self.target,
            })
            .map_err(|_| ())
    }

    fn read_pos(&mut self, channel: &mut Channel<TcpStream>) -> Result<(), ()> {
        if let Ok(Packet::Player { pos, target }) = channel.receive() {
            self.pos = pos;
            self.target = target;
            Ok(())
        } else {
            Err(())
        }
    }
}
