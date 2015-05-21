use rustc_serialize::{self, json};
use hyper::{Url, Client};
use queryst::parse;
use std::collections::HashMap;

use std::env;
use regex::Regex;

use super::signature::*;
use super::request::*;
use super::request_url::*;
use super::json_structures::*;
use super::QueryParameters;
use super::util::*;

pub struct Pusher {
  app_id: String,
  key: String,
  secret: String, 
  host: String,
  secure: bool,
  http_client: Client,
}

pub struct PusherBuilder {
  app_id: String,
  key: String,
  secret: String, 
  host: String,
  secure: bool,
  http_client: Client,
}

impl PusherBuilder{
  pub fn host(mut self, host: &str) -> PusherBuilder{
    self.host = host.to_string();
    self
  }

  pub fn secure(mut self, secure: bool) -> PusherBuilder {  
    self.secure = secure;
    self
  }

  pub fn client(mut self, http_client: Client) -> PusherBuilder {
    self.http_client = http_client;
    self
  }

  pub fn finalize(self) -> Pusher {
   Pusher {
      app_id: self.app_id,
      key: self.key,
      secret: self.secret,
      host: self.host,
      secure: self.secure,
      http_client: self.http_client,
    } 
  }

}

impl Pusher{

  pub fn new(app_id: &str, key: &str, secret: &str) -> PusherBuilder {
    let http_client = Client::new();

    PusherBuilder {
      app_id: app_id.to_string(),
      key: key.to_string(),
      secret: secret.to_string(),
      host: "api.pusherapp.com".to_string(),
      secure: false,
      http_client: http_client,
    }

  }

  pub fn from_env(key: &str) -> PusherBuilder {
    let url_opt = env::var_os(key).unwrap();
    let os_url = url_opt.to_str();
    let url = os_url.unwrap();
    Pusher::from_url(url)
  }

  pub fn from_url(url: &str) -> PusherBuilder {
    let pusher_url = Url::parse(url).unwrap();

    let key = pusher_url.username().unwrap();
    let secret = pusher_url.password().unwrap();
    let host = pusher_url.host().unwrap();
    let path = pusher_url.path().unwrap();
    let app_id = &path[1];
    let mut secure  = false;

    if pusher_url.scheme == "https" {
      secure = true;
    }

    let http_client = Client::new();

    PusherBuilder {
      app_id: app_id.to_string(),
      key: key.to_string(),
      secret: secret.to_string(),
      host: host.to_string(),
      secure: secure,
      http_client: http_client,
    }

  }

  pub fn trigger<Payload : rustc_serialize::Encodable>(&mut self, channel: &str, event: &str, payload: Payload)-> Result<String, String> {
    let channels = vec![channel.to_string()];
    self._trigger(channels, event, payload, None)
  }

  pub fn trigger_exclusive<Payload : rustc_serialize::Encodable>(&mut self, channel: &str, event: &str, payload: Payload, socket_id: &str)-> Result<String, String> {
    let channels = vec![channel.to_string()];
    self._trigger(channels, event, payload, Some(socket_id.to_string()))
  }

  pub fn trigger_multi<Payload : rustc_serialize::Encodable>(&mut self, channels: &Vec<&str>, event: &str, payload: Payload)-> Result<String, String> {
    let channel_strings = channels.into_iter().map(|c| c.to_string()).collect();
    self._trigger(channel_strings, event, payload, None)
  }

  pub fn trigger_multi_exclusive<Payload : rustc_serialize::Encodable>(&mut self, channels: Vec<&str>, event: &str, payload: Payload, socket_id: &str)-> Result<String, String> {
    let channel_strings = channels.into_iter().map(|c| c.to_string()).collect();
    self._trigger(channel_strings, event, payload, Some(socket_id.to_string()))
  }

  fn _trigger<Payload : rustc_serialize::Encodable>(&mut self, channels: Vec<String>, event: &str, payload: Payload, socket_id: Option<String>) -> Result<String, String> { 

    if event.len() > 200 {
      return Err("Event name is limited to 200 chars".to_string())
    }
    
    if let Err(message) = validate_channels(&channels) {
      return Err(message)
    }

    let request_url_string = format!("{}://{}/apps/{}/events", self.scheme(), self.host, self.app_id);
    let mut request_url = Url::parse(&request_url_string).unwrap();

    let json_payload = json::encode(&payload).unwrap();

    let raw_body = TriggerEventData{
      name: event.to_string(),
      channels: channels,
      data: json_payload,
      socket_id: socket_id,
    };

    let body = json::encode(&raw_body).unwrap();

    if body.len() > 10240 {
      return Err("Data must be smaller than 10kb".to_string())
    }

    let method = "POST";
    update_request_url(method, &mut request_url, &self.key, &self.secret, timestamp(), Some(&body), None);
    create_request::<String>(&mut self.http_client, method, request_url, None)
  }

  pub fn channels(&mut self, params: QueryParameters) -> Result<ChannelList, String>{
    let request_url_string = format!("{}://{}/apps/{}/channels", self.scheme(), self.host, self.app_id);
    let mut request_url = Url::parse(&request_url_string).unwrap();
    let method = "GET";
    update_request_url(method, &mut request_url, &self.key, &self.secret, timestamp(), None, params);
    create_request::<ChannelList>(&mut self.http_client, method, request_url, None)
  }

  fn scheme(&self) -> &str {
    if self.secure {
      "https"
    } else {
      "http"
    }
  }

  pub fn channel(&mut self, channel_name: &str, params: QueryParameters) -> Result<Channel, String>{
    let request_url_string = format!("{}://{}/apps/{}/channels/{}", self.scheme(), self.host, self.app_id, channel_name);
    let mut request_url = Url::parse(&request_url_string).unwrap();
    let method = "GET";
    update_request_url(method, &mut request_url, &self.key, &self.secret, timestamp(), None, params);
    create_request::<Channel>(&mut self.http_client, method, request_url, None)
  }

  pub fn channel_users(&mut self, channel_name : &str) -> Result<ChannelUserList, String> {
    let request_url_string = format!("{}://{}/apps/{}/channels/{}/users", self.scheme(), self.host, self.app_id, channel_name);
    let mut request_url = Url::parse(&request_url_string).unwrap();
    let method = "GET";
    update_request_url(method, &mut request_url, &self.key, &self.secret, timestamp(), None, None);
    create_request::<ChannelUserList>(&mut self.http_client, method, request_url, None)
  }

  pub fn authenticate_private_channel(&self, body: &String) -> Result<String, &str> {
    self.authenticate_channel(body, None)
  }

  pub fn authenticate_presence_channel(&self, body: &String, member: &Member) -> Result<String, &str> {
    self.authenticate_channel(body, Some(member))
  }

  fn authenticate_channel(&self, body: &String, member: Option<&Member>) -> Result<String, &str> {
    let object = parse(body);
    
    let json_params = match object {
      Ok(parsed_params) => parsed_params.to_string(),
      Err(_) => return Err("Could not parse body") 
    };

    let auth : AuthParams = match json::decode(&json_params) {
      Ok(parsed_auth) => parsed_auth,
      Err(_) => return Err("Could not parse body")
    };

    let mut auth_map = HashMap::new();
    let channel_name = auth.channel_name;
    let socket_id = auth.socket_id;

    let socket_id_regex = Regex::new(r"\A\d+\.\d+\z").unwrap(); // how to make this global?

    if !socket_id_regex.is_match(&socket_id) {
      return Err("Invalid socket_id")
    }

    let mut to_sign = format!("{}:{}", socket_id, channel_name);

    if let Some(presence_member) = member {
      let json_member = json::encode(presence_member).unwrap();
      to_sign = format!("{}:{}", to_sign, json_member);
      auth_map.insert("channel_data", json_member);
    }

    create_channel_auth(&mut auth_map, &self.key, &self.secret, &to_sign);
    Ok(json::encode(&auth_map).unwrap())
  }

  pub fn webhook(&self, key: &String, signature: &String, body: &str) -> Result<Webhook, &str> {
    if (&self.key == key) && check_signature(signature, &self.secret, body) {
      println!("Checks out");
      println!("{:?}", body);
      let decoded_webhook : Webhook = json::decode(&body[..]).unwrap();
      return Ok(decoded_webhook)
    }
    Err("Invalid webhook")
  }

}

#[test]
fn test_private_channel_authentication(){
  let mut pusher = Pusher::new("id", "278d425bdf160c739803", "7ad3773142a6692b25b8").finalize();
  let expected = "{\"auth\":\"278d425bdf160c739803:58df8b0c36d6982b82c3ecf6b4662e34fe8c25bba48f5369f135bf843651c3a4\"}".to_string();
  let body = "channel_name=private-foobar&socket_id=1234.1234".to_string();
  let result = pusher.authenticate_private_channel(&body);
  assert_eq!(result.unwrap(), expected)
}

#[test]
fn test_presence_channel_authentication(){
  let mut pusher = Pusher::new("id", "278d425bdf160c739803", "7ad3773142a6692b25b8").finalize();
  let expected = "{\"auth\":\"278d425bdf160c739803:48dac51d2d7569e1e9c0f48c227d4b26f238fa68e5c0bb04222c966909c4f7c4\",\"channel_data\":\"{\\\"user_id\\\":\\\"10\\\",\\\"user_info\\\":{\\\"name\\\":\\\"Mr. Pusher\\\"}}\"}";
  let expected_encoded : HashMap<String, String> = json::decode(expected).unwrap(); 
  let mut member_data = HashMap::new();
  member_data.insert("name", "Mr. Pusher");
  let presence_data = Member{user_id: "10", user_info: member_data};
  let body = "channel_name=presence-foobar&socket_id=1234.1234".to_string();
  let result_json = pusher.authenticate_presence_channel(&body, &presence_data);
  let result_decoded : HashMap<String, String> = json::decode(&result_json.unwrap()).unwrap();
  
  assert_eq!(result_decoded["auth"], expected_encoded["auth"]);
  assert_eq!(result_decoded["channel_data"], expected_encoded["channel_data"]);
}

#[test]
fn test_socket_id_validation(){
  let mut pusher = Pusher::new("id", "278d425bdf160c739803", "7ad3773142a6692b25b8").finalize();
  let body = "channel_name=private-foobar&socket_id=12341234".to_string();
  let result = pusher.authenticate_private_channel(&body);
  assert_eq!(result.unwrap_err(), "Invalid socket_id")
}

#[test]
fn test_client_webhook_validation(){
  let mut pusher = Pusher::new("id", "key", "secret").finalize();
  let key = "key".to_string();
  let signature = "05a115b7898e4956cf46df2dd2822b3b913a4255343acd82d31609f222765c6a".to_string();
  let result = pusher.webhook(&key, &signature, "{\"time_ms\":1327078148132,\"events\":[{\"name\":\"event_name\",\"some\":\"data\"}]}");

  let webhook = result.unwrap();
  assert_eq!(webhook.time_ms, 1327078148132);
  assert_eq!(webhook.events[0]["name"], "event_name");
  assert_eq!(webhook.events[0]["some"], "data")
}

#[test]
fn test_webhook_improper_key_case(){
  let mut pusher = Pusher::new("id", "key", "secret").finalize();
  let key = "narr you're going down!".to_string();
  let signature = "2677ad3e7c090b2fa2c0fb13020d66d5420879b8316eb356a2d60fb9073bc778".to_string();
  let result = pusher.webhook(&key, &signature,"{\"hello\":\"world\"}");
  assert_eq!(result.unwrap_err(), "Invalid webhook")
}

#[test]
fn test_webhook_improper_signature_case(){
  let mut pusher = Pusher::new("id", "key", "secret").finalize();
  let key = "key".to_string();
  let signature = "26c778".to_string();
  let result = pusher.webhook(&key, &signature,"{\"hello\":\"world\"}");
  assert_eq!(result.unwrap_err(), "Invalid webhook")
}

#[test]
fn test_channel_number_validation(){
  let mut pusher = Pusher::new("id", "key", "secret").finalize();
  let channels = vec!["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11"];
  let res = pusher.trigger_multi(&channels, "yolo", "woot");
  assert_eq!(res.unwrap_err(), "Cannot trigger on more than 10 channels")
}

#[test]
fn test_channel_format_validation(){
  let mut pusher = Pusher::new("id", "key", "secret").finalize();
  let res = pusher.trigger("w000^$$£@@@", "yolo", "woot");
  assert_eq!(res.unwrap_err(), "Channels must be formatted as such: ^[-a-zA-Z0-9_=@,.;]+$")
}

#[test]
fn test_channel_length_validation(){
  let mut pusher = Pusher::new("id", "key", "secret").finalize();
  let mut channel = "".to_string();

  for i in 1..202 {
    channel = channel + "a"
  }

  let res = pusher.trigger(&channel, "yolo", "woot");
  assert_eq!(res.unwrap_err(), "Channel names must be under 200 characters")
}

#[test]
fn test_trigger_payload_size_validation(){
  let mut pusher = Pusher::new("id", "key", "secret").finalize();
  let mut data = "".to_string();

  for i in 1..10242 {
    data = data + "a"
  }

  let res = pusher.trigger("yolo", "new_yolo", &data);
  assert_eq!(res.unwrap_err(), "Data must be smaller than 10kb")
}

#[test]
fn test_event_name_length_validation(){
  let mut pusher = Pusher::new("id", "key", "secret").finalize();
  let mut event = "".to_string();

  for i in 1..202 {
    event = event + "a"
  }

  let res = pusher.trigger("yolo", &event, "woot");
  assert_eq!(res.unwrap_err(), "Event name is limited to 200 chars")
}




