use bincode::ErrorKind;
use std::fmt::Display;

#[derive(Debug)]
pub enum ChannelError {
    Io(std::io::Error),
    Bincode,
}

impl From<Box<ErrorKind>> for ChannelError {
    fn from(value: Box<ErrorKind>) -> Self {
        match *value {
            ErrorKind::Io(error) => ChannelError::Io(error),
            _ => ChannelError::Bincode,
        }
    }
}

impl Display for ChannelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelError::Io(error) => error.fmt(f),
            ChannelError::Bincode => writeln!(f, "Bincode failed to (de)serialize"),
        }
    }
}
impl std::error::Error for ChannelError {}
