use std::sync::OnceLock;

use regex::Regex;

use crate::Error;

pub fn validate_socket_id(socket_id: &str) -> Result<(), Error> {
    static SOCKET_ID_REGEX: OnceLock<Regex> = OnceLock::new();
    let socket_id_regex = SOCKET_ID_REGEX.get_or_init(|| Regex::new(r"\A\d+\.\d+\z").unwrap());
    if socket_id_regex.is_match(socket_id) {
        Ok(())
    } else {
        Err(Error::InvalidSocketId(socket_id.into()))
    }
}

pub fn validate_channel(channel: &str) -> Result<(), Error> {
    static CHANNEL_REGEX: OnceLock<Regex> = OnceLock::new();
    let channel_regex = CHANNEL_REGEX.get_or_init(|| Regex::new(r"^[-a-zA-Z0-9_=@,.;]+$").unwrap());

    if channel.len() > 200 || !channel_regex.is_match(channel) {
        Err(Error::InvalidChannelName(channel.into()))
    } else {
        Ok(())
    }
}

pub fn validate_event(event: &str) -> Result<(), Error> {
    if event.len() > 200 {
        Err(Error::InvalidEventName(event.into()))
    } else {
        Ok(())
    }
}

pub fn validate_channels(channels: &[String]) -> Result<(), Error> {
    if channels.len() > 10 {
        return Err(Error::TooManyChannels(channels.len()));
    }

    for channel in channels {
        validate_channel(channel)?;
    }
    Ok(())
}
