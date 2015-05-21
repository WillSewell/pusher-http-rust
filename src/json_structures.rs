use std::collections::HashMap;

#[derive(RustcEncodable)]
pub struct TriggerEventData {
    pub name: String,
    pub channels: Vec<String>,
    pub data: String,
    pub socket_id: Option<String>,
}

#[derive(RustcDecodable, Debug)]
pub struct TriggeredEvents {
  pub event_ids: HashMap<String, String>
}

#[derive(RustcDecodable, Debug)]
pub struct ChannelList {
    pub channels: HashMap<String, Channel>, // something fishy in practice
}

#[derive(RustcEncodable)]
pub struct Member<'a> {
  pub user_id: &'a str,
  pub user_info: HashMap<&'a str, &'a str>
}

#[derive(RustcDecodable, Debug)]
pub struct Webhook {
  pub time_ms: i64,
  pub events: Vec<HashMap<String, String>>,
}

#[derive(RustcDecodable, Debug)]
pub struct Channel {
  pub occupied: Option<bool>,
  pub user_count: Option<i32>,
  pub subscription_count: Option<i32>,
}

#[derive(RustcDecodable, Debug)]
pub struct ChannelUserList {
  pub users: Vec<ChannelUser>,
}

#[derive(RustcDecodable, Debug)]
pub struct ChannelUser {
  pub id: String,
}

#[derive(RustcDecodable, Debug)]
pub struct AuthParams {
  pub channel_name: String,
  pub socket_id: String,
}