use hyper::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("Invalid socket ID: `{0}`")]
    InvalidSocketId(String),
    #[error("Invalid channel name: `{0}`")]
    InvalidChannelName(String),
    #[error("Invalid event name: `{0}`")]
    InvalidEventName(String),
    #[error("Too many channels: {0} (maximum 10)")]
    TooManyChannels(usize),
    #[error("Event data too large: {0} bytes (maximum 10kb)")]
    EventDataTooLarge(usize),
    #[error(transparent)]
    Http(#[from] hyper::http::Error),
    #[error(transparent)]
    Hyper(#[from] hyper::Error),
    #[error("Server responded with {0} - {1}")]
    Response(StatusCode, String),
}
