use std::collections::HashMap;

#[derive(RustcEncodable)]
pub struct TriggerEventData {
    pub name: String,
    pub channels: Vec<String>,
    pub data: String,
    pub socket_id: Option<String>,
}

/// When querying the state of Pusher channels, you can pass this in to specify
/// options.
pub type QueryParameters<'a> = Vec<(&'a str, &'a str)>;

/// Any event_ids returned by the HTTP API, if connected to certain clusters.
#[derive(RustcDecodable, Debug)]
pub struct TriggeredEvents {
  /// For certain clusters, event_ids will be returned upon triggering.
  /// Otherwise, this value will be `None`.
  pub event_ids: Option<HashMap<String, String>>
}

/// A list of channels returned by the API.
#[derive(RustcDecodable, Debug)]
pub struct ChannelList {
    pub channels: HashMap<String, Channel>, // something fishy in practice
}

/// When authenticating presence-channels, this represents a particular member
/// of the channel. This object becomes associated with that user's subscription.
#[derive(RustcEncodable)]
pub struct Member<'a> {
  /// Supply an id of the member
  pub user_id: &'a str,
  /// Supply any optional information to be associated with the member
  pub user_info: Option<HashMap<&'a str, &'a str>>
}

/// This is returned upon validating that a webhook is indeed from Pusher,
/// carrying all the data received by that POST request.
#[derive(RustcDecodable, Debug)]
pub struct Webhook {
  /// The timestamp of the webhook
  pub time_ms: i64,
  /// The events received with the webhook
  pub events: Vec<HashMap<String, String>>,
}

/// This represents the data received upon querying the state of a Pusher channel.
#[derive(RustcDecodable, Debug)]
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
#[derive(RustcDecodable, Debug)]
pub struct ChannelUserList {
  pub users: Vec<ChannelUser>,
}

/// A particular user who occupies a presence channel.
#[derive(RustcDecodable, Debug)]
pub struct ChannelUser {
  pub id: String,
}
