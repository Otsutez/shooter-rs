use anyhow::{Context, Result};
use game_channel::{Channel, ChannelVector2, Packet};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::{thread, time};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:1234")?;
    let mut player_1 = Player::new_player_1();
    let mut player_2 = Player::new_player_2();
    eprintln!("Game server started on local port 1234.");

    eprintln!("Waiting for new session...");

    while let Ok(conn) = listener.accept() {
        eprintln!("Session started!");
        if let Err(e) = handle_session(&listener, conn, &mut player_1, &mut player_2) {
            eprintln!("{}", e);
        }
        eprintln!("Session over.");
        player_1.reset_player_1();
        player_2.reset_player_2();
        eprintln!("Waiting for new session...");
    }
    Ok(())
}

fn handle_session(
    listener: &TcpListener,
    conn: (TcpStream, SocketAddr),
    player_1: &mut Player,
    player_2: &mut Player,
) -> Result<()> {
    let (s1, addr) = conn;
    eprintln!("Connection from {}", addr);
    let mut c1 = Channel::with_stream(s1);

    // Send player 1 initial position
    player_1.write_pos(&mut c1)?;
    eprintln!("Sent player 1 initial position");

    // Wait for connection from player 2
    let (s2, addr) = listener.accept()?;
    eprintln!("Connection from {}", addr);
    let mut c2 = Channel::with_stream(s2);

    // Send player 2 initial position
    player_2.write_pos(&mut c2)?;
    eprintln!("Sent player 2 initial position");

    // Send enemy position
    // This will make the client go into countdown state
    player_2.write_pos(&mut c1)?;
    player_1.write_pos(&mut c2)?;

    // Countdown
    c1.send(Packet::Time(3)).context("Sending time failed")?;
    c2.send(Packet::Time(3)).context("Sending time failed")?;
    thread::sleep(time::Duration::from_secs(1));
    c1.send(Packet::Time(2)).context("Sending time failed")?;
    c2.send(Packet::Time(2)).context("Sending time failed")?;
    thread::sleep(time::Duration::from_secs(1));
    c1.send(Packet::Time(1)).context("Sending time failed")?;
    c2.send(Packet::Time(1)).context("Sending time failed")?;
    thread::sleep(time::Duration::from_secs(1));

    // Send 0 to signal start of game
    c1.send(Packet::Time(0)).context("Sending time failed")?;
    c2.send(Packet::Time(0)).context("Sending time failed")?;

    // Send players current position and receive their next position
    loop {
        // Send enemies
        player_2.write_pos(&mut c1)?;
        player_1.write_pos(&mut c2)?;

        // Send players health
        player_1.write_health(&mut c1)?;
        player_2.write_health(&mut c2)?;

        // Receive players
        player_1.read_pos(&mut c1)?;
        player_2.read_pos(&mut c2)?;

        // Receive enemies health
        player_2.read_health(&mut c1)?;
        player_1.read_health(&mut c2)?;

        eprintln!("player_1: {:?}", player_1.pos);
        eprintln!("player_2: {:?}", player_2.pos);
    }
}

struct Player {
    pos: ChannelVector2,
    target: ChannelVector2,
    health: u8,
}

impl Player {
    fn new_player_1() -> Self {
        Player {
            pos: ChannelVector2 { x: 0.0, z: 18.0 },
            target: ChannelVector2 { x: 0.0, z: -1.0 },
            health: 100,
        }
    }

    fn reset_player_1(&mut self) {
        self.pos = ChannelVector2 { x: 0.0, z: 18.0 };
        self.target = ChannelVector2 { x: 0.0, z: -1.0 };
        self.health = 100;
    }

    fn new_player_2() -> Self {
        Player {
            pos: ChannelVector2 { x: 0.0, z: -18.0 },
            target: ChannelVector2 { x: 0.0, z: 1.0 },
            health: 100,
        }
    }

    fn reset_player_2(&mut self) {
        self.pos = ChannelVector2 { x: 0.0, z: -18.0 };
        self.target = ChannelVector2 { x: 0.0, z: 1.0 };
        self.health = 100;
    }

    fn write_pos(&self, channel: &mut Channel<TcpStream>) -> Result<()> {
        channel.send(Packet::Player {
            pos: self.pos,
            target: self.target,
        })?;
        Ok(())
    }

    fn read_pos(&mut self, channel: &mut Channel<TcpStream>) -> Result<()> {
        if let Ok(Packet::Player { pos, target }) = channel.receive() {
            self.pos = pos;
            self.target = target;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to receive player position"))
        }
    }

    fn write_health(&self, channel: &mut Channel<TcpStream>) -> Result<()> {
        channel.send(Packet::Health(self.health))?;
        Ok(())
    }

    fn read_health(&mut self, channel: &mut Channel<TcpStream>) -> Result<()> {
        if let Ok(Packet::Health(health)) = channel.receive() {
            self.health = health;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to read player health"))
        }
    }
}
