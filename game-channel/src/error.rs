use bincode::ErrorKind;
use std::fmt::Display;

#[derive(Debug)]
pub struct ChannelError(Box<ErrorKind>);

impl From<Box<ErrorKind>> for ChannelError {
    fn from(value: Box<ErrorKind>) -> Self {
        ChannelError(value)
    }
}

impl Display for ChannelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl std::error::Error for ChannelError {}
