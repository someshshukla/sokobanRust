

use super::game;
use std::convert::From;
use std::error;
use std::fmt::{self, Display, Formatter};
use std::io;

/// Represents an application error
#[derive(Debug)]
pub enum SokobanError {
    IoError(io::Error),
    ParseError(game::InvalidChar),
}

impl error::Error for SokobanError {
    fn description(&self) -> &str {
        match *self {
            SokobanError::IoError(..) => "I/O error",
            SokobanError::ParseError(..) => "Level parsing error",
        }
    }
}

impl Display for SokobanError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            SokobanError::IoError(ref err) => write!(f, "{}", *err),
            SokobanError::ParseError(ref err) => write!(f, "{}", *err),
        }
    }
}

impl From<io::Error> for SokobanError {
    fn from(err: io::Error) -> Self {
        SokobanError::IoError(err)
    }
}

impl From<game::InvalidChar> for SokobanError {
    fn from(err: game::InvalidChar) -> Self {
        SokobanError::ParseError(err)
    }
}
