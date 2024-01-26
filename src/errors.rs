use std::{
  fmt,
  num::{ParseFloatError, ParseIntError},
};

use rodio::{decoder::DecoderError, source::SeekError};

#[derive(Debug)]
pub enum CommandError {
  ParseFloatError(ParseFloatError),
  ParseIntError(ParseIntError),
  SeekError(SeekError),
  InvalidArg(String),
  Error(std::io::Error),
  DecodeError(DecoderError),
}

impl From<&str> for CommandError {
  fn from(value: &str) -> Self {
    return CommandError::InvalidArg(value.to_string());
  }
}

impl From<ParseFloatError> for CommandError {
  fn from(value: ParseFloatError) -> Self {
    return CommandError::ParseFloatError(value);
  }
}

impl From<ParseIntError> for CommandError {
  fn from(value: ParseIntError) -> Self {
    return CommandError::ParseIntError(value);
  }
}

impl From<SeekError> for CommandError {
  fn from(value: SeekError) -> Self {
    return CommandError::SeekError(value);
  }
}

impl From<std::io::Error> for CommandError {
  fn from(value: std::io::Error) -> Self {
    return CommandError::Error(value);
  }
}

impl From<DecoderError> for CommandError {
  fn from(value: DecoderError) -> Self {
    return CommandError::DecodeError(value);
  }
}

impl fmt::Display for CommandError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}
