use std::fmt;
use std::error::Error;
use crate::models::common::GameMode;

#[derive(Debug)]
pub enum ParseError<GameMode: fmt::Display + 'static> {
    InvalidChart(String),
    InvalidMode(String, GameMode),
    EmptyChartData,
    UnsupportedFormat,
}

#[derive(Debug)]
pub enum WriteError<GameMode: fmt::Display + 'static> {
    InvalidKeyCount(u8, String, String),
    Unimpl(GameMode),
}

impl<GameMode: fmt::Display + 'static> fmt::Display for ParseError<GameMode> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidChart(error) => write!(f, "Failed to parse because Invalid chart data provided or file is malformed: {error}"),
            Self::InvalidMode(mode, target) => write!(f, "Cannot parse because '{mode}' mode is invalid or not supported, parsing for {target}"),
            Self::EmptyChartData => write!(f, "Cannot parse because empty chart data was provided"),
            Self::UnsupportedFormat => write!(f, "Cannot parse because this is an unsupported file format"),
        }
    }
}

impl<GameMode: fmt::Display + 'static> fmt::Display for WriteError<GameMode> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidKeyCount(key_count, avaibable_key_counts, format) => write!(f, "Failed to write because {key_count}k is not supported, {format} only supports {avaibable_key_counts}"),
            Self::Unimpl(_gamemode) => {unimplemented!()},
        }
    }
}



impl<GameMode: fmt::Debug + fmt::Display + 'static> Error for ParseError<GameMode> {}
impl<GameMode: fmt::Debug + fmt::Display + 'static> Error for WriteError<GameMode> {}
