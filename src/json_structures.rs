use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::util::serde_utils::optional_sorted_map;

#[derive(Serialize)]
pub struct TriggerEventData {
    pub name: String,
    pub channels: Vec<String>,
    pub data: String,
    pub socket_id: Option<String>,
}

/// When querying the state of Pusher channels, you can pass this in to specify
/// options.
pub type QueryParameters = Vec<(String, String)>;

/// Any event_ids returned by the HTTP API, if connected to certain clusters.
#[derive(Deserialize, Debug)]
pub struct TriggeredEvents {
    /// For certain clusters, event_ids will be returned upon triggering.
    /// Otherwise, this value will be `None`.
    pub event_ids: Option<HashMap<String, String>>,
}

/// A list of channels returned by the API.
#[derive(Deserialize, Debug)]
pub struct ChannelList {
    pub channels: HashMap<String, Channel>, // something fishy in practice
}

/// When authenticating presence-channels, this represents a particular member
/// of the channel. This object becomes associated with that user's subscription.
#[derive(Serialize)]
pub struct Member<'a> {
    /// Supply an id of the member
    pub user_id: &'a str,
    /// Supply any optional information to be associated with the member
    #[serde(serialize_with = "optional_sorted_map", skip_serializing_if = "Option::is_none")]
    pub user_info: Option<HashMap<&'a str, &'a str>>,
}


/// When authenticating user, this represents a particular user.
/// This object becomes associated with that user's subscription.
#[derive(Serialize)]
pub struct User<'a> {
    /// Supply an id of the user
    pub id: &'a str,
    /// Supply any optional information to be associated with the user
    #[serde(serialize_with = "optional_sorted_map", skip_serializing_if = "Option::is_none")]
    pub user_info: Option<HashMap<&'a str, &'a str>>,
    /// Supply optional list of user IDs to allow viewing presence information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watchlist: Option<Vec<&'a str>>,
}

/// This is returned upon validating that a webhook is indeed from Pusher,
/// carrying all the data received by that POST request.
#[derive(Deserialize, Debug)]
pub struct Webhook {
    /// The timestamp of the webhook
    pub time_ms: i64,
    /// The events received with the webhook
    pub events: Vec<HashMap<String, String>>,
}

/// This represents the data received upon querying the state of a Pusher channel.
#[derive(Deserialize, Debug)]
pub struct Channel {
    /// Is the channel occupied?
    pub occupied: Option<bool>,
    /// The number of users presently subscribed to the channel
    pub user_count: Option<i32>,
    /// For accounts with subscription-counting enabled, the number of users currently
    /// subscribed to the channel.
    pub subscription_count: Option<i32>,
}

/// The list of users subscribed to a presence channel, as returned by the Pusher
/// API.
#[derive(Deserialize, Debug)]
pub struct ChannelUserList {
    pub users: Vec<ChannelUser>,
}

/// A particular user who occupies a presence channel.
#[derive(Deserialize, Debug)]
pub struct ChannelUser {
    pub id: String,
}
