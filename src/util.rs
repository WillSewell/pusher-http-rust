use regex::Regex;

pub fn validate_channels(channels: &Vec<String>) -> Result<bool, String> {
    if channels.len() > 10 {
        return Err("Cannot trigger on more than 10 channels".to_string());
    }

    let channel_regex = Regex::new(r"^[-a-zA-Z0-9_=@,.;]+$").unwrap(); // how to make this global?

    for channel in channels {
        if channel.len() > 200 {
            return Err("Channel names must be under 200 characters".to_string());
        }
        if !channel_regex.is_match(channel) {
            return Err("Channels must be formatted as such: ^[-a-zA-Z0-9_=@,.;]+$".to_string());
        }
    }
    Ok(true)
}


pub mod serde_utils {
    use serde::{Serialize, ser::Serializer};
    use std::collections::{BTreeMap, HashMap};

    pub fn sorted_map<S: Serializer, K: Serialize + Ord, V: Serialize>(
        value: &HashMap<K, V>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let items: Vec<(_, _)> = value.iter().collect();
        BTreeMap::from_iter(items).serialize(serializer)
    }
    
    pub fn optional_sorted_map<S: Serializer, K: Serialize + Ord, V: Serialize>(
        value: &Option<HashMap<K, V>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(map) => sorted_map(map, serializer),
            None => serializer.serialize_none(),
        }
    }
}