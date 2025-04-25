use anyhow::{Context, Result};
use game_channel::error::ChannelError;
use game_channel::{Channel, ChannelVector2, Packet};
use std::io::ErrorKind;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::{thread, time};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("0.0.0.0:1234")?;
    let mut player_1 = Player::new_player_1();
    let mut player_2 = Player::new_player_2();
    eprintln!("Game server started on {}", listener.local_addr().unwrap());

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
    s1.set_nodelay(true)?;
    eprintln!("Connection from {}", addr);
    let mut c1 = Channel::with_stream(s1);

    // Send player 1 initial position
    player_1.write_pos(&mut c1)?;
    eprintln!("Sent player 1 initial position");

    // Wait for connection from player 2
    let (s2, addr) = listener.accept()?;
    s2.set_nodelay(true)?;
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

    let mut s1_closed = false;
    let mut s2_closed = false;

    // Send players current position and receive their next position
    loop {
        if s1_closed && s2_closed {
            break;
        }

        // Send enemies
        if let Err(ChannelError::Io(_)) = player_2.write_pos(&mut c1) {
            s1_closed = true;
        }
        if let Err(ChannelError::Io(_)) = player_1.write_pos(&mut c2) {
            s2_closed = true;
        }

        // Send players health
        if let Err(ChannelError::Io(_)) = player_1.write_health(&mut c1) {
            s1_closed = true;
        }
        if let Err(ChannelError::Io(_)) = player_2.write_health(&mut c2) {
            s2_closed = true;
        }

        // Receive players
        if let Err(ChannelError::Io(_)) = player_1.read_pos(&mut c1) {
            s1_closed = true;
        }
        if let Err(ChannelError::Io(_)) = player_2.read_pos(&mut c2) {
            s2_closed = true;
        }

        // Receive enemies health
        if let Err(ChannelError::Io(e)) = player_2.read_health(&mut c1) {
            if e.kind() == ErrorKind::ConnectionAborted {
                s1_closed = true;
            }
        }
        if let Err(ChannelError::Io(e)) = player_1.read_health(&mut c2) {
            if e.kind() == ErrorKind::ConnectionAborted {
                s2_closed = true;
            }
        }

        // eprintln!("player_1: {:?}", player_1.pos);
        // eprintln!("player_2: {:?}", player_2.pos);
    }
    Ok(())
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

    fn write_pos(&self, channel: &mut Channel<TcpStream>) -> Result<(), ChannelError> {
        channel.send(Packet::Player {
            pos: self.pos,
            target: self.target,
        })
    }

    fn read_pos(&mut self, channel: &mut Channel<TcpStream>) -> Result<(), ChannelError> {
        channel.receive().and_then(|packet| {
            if let Packet::Player { pos, target } = packet {
                self.pos = pos;
                self.target = target;
            }
            Err(ChannelError::Bincode)
        })
    }

    fn write_health(&self, channel: &mut Channel<TcpStream>) -> Result<(), ChannelError> {
        channel.send(Packet::Health(self.health))
    }

    fn read_health(&mut self, channel: &mut Channel<TcpStream>) -> Result<(), ChannelError> {
        channel.receive().and_then(|packet| {
            if let Packet::Health(health) = packet {
                self.health = health;
            }
            Err(ChannelError::Bincode)
        })
    }
}
