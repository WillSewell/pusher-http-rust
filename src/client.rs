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
use super::util::*;

/// A client to interact with Pusher's HTTP API to trigger, query application state,
/// authenticate private- or presence-channels, and validate webhooks.
pub struct Pusher {
  /// Your app_id from <http://app.pusher.com>
  pub app_id: String, 
  /// Your key from <http://app.pusher.com>
  pub key: String, 
  /// Your secret from <http://app.pusher.com>
  pub secret: String, 
  /// The host you wish to connect to. Defaults to api.pusherapp.com
  pub host: String, 
  /// If true, requests are made over HTTPS.
  pub secure: bool, 
  /// The underlying Hyper HTTP client.
  pub http_client: Client, 
}

/// An ephemeral object upon which to pass configuration options to when
/// initializing a Pusher instance.
pub struct PusherBuilder {
  pub app_id: String,
  pub key: String,
  pub secret: String, 
  pub host: String,
  pub secure: bool,
  pub http_client: Client,
}

impl PusherBuilder{
  
  /// This method changes the host to which API requests will be made.
  /// This defaults to `api.pusherapp.com`.
  ///
  /// ```
  /// # use pusher::Pusher;
  /// let mut pusher = Pusher::new("id", "key", "secret").host("foo.bar.com").finalize();
  /// ```
  pub fn host(mut self, host: &str) -> PusherBuilder{
    self.host = host.to_string();
    self
  }

  /// This method determines whether requests will be made over HTTPS. This defaults to `false`.
  ///
  /// ```
  /// # use pusher::Pusher;
  /// let mut pusher = Pusher::new("id", "key", "secret").secure().finalize();
  /// ```
  pub fn secure(mut self) -> PusherBuilder {  
    self.secure = true;
    self
  }

  /// If you wish to configure a [Hyper client](http://hyper.rs/hyper/hyper/client/struct.Client.html),
  /// pass it in to this method.
  pub fn client(mut self, http_client: Client) -> PusherBuilder {
    self.http_client = http_client;
    self
  }

  /// This method actually creates the `Pusher` instance from your chained configuration.
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

  /// Initializes the client that makes requests to the HTTP API, authenticates
  /// private- or presence-channels, and validates webhooks.
  /// 
  /// This returns a `PusherBuilder`, on which to set configuration options
  /// before calling `finalize()`.
  ///
  /// **Example:**
  ///
  ///
  /// ```
  /// # use pusher::Pusher;
  /// let mut pusher = Pusher::new("id", "key", "secret").host("foo.bar.com").finalize();
  /// ```
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

  /// Initializes a client from a Pusher URL.
  ///
  /// This returns a `PusherBuilder`, on which to set configuration options
  /// before calling `finalize()`.
  ///
  /// **Example:**
  /// 
  ///
  /// ```
  /// # use pusher::Pusher;
  /// Pusher::from_url("http://key:secret@api.host.com/apps/id").finalize();
  /// ```
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


  /// Initializes a client from an environment variable Pusher URL.
  ///
  /// This returns a `PusherBuilder`, on which to set configuration options
  /// before calling `finalize()`.
  ///
  ///
  /// **Example:**
  /// 
  /// ```
  /// # use pusher::Pusher;
  /// Pusher::from_env("PUSHER_URL").finalize();
  /// ```
  pub fn from_env(key: &str) -> PusherBuilder {
    let url_opt = env::var_os(key).unwrap();
    let os_url = url_opt.to_str();
    let url = os_url.unwrap();
    Pusher::from_url(url)
  }

  /// This method allows you to trigger Pusher events. You can test this out by
  /// going on your debug console at <http://app.pusher.com>.
  ///
  /// It is possible to trigger an event on one or more channels. Channel names 
  /// can contain only characters which are alphanumeric, _ or -` and have to be
  /// at most 200 characters long. Event name can be at most 200 characters long 
  /// too, and a payload is limited to 10kb.
  ///
  /// This method is for triggering on only one channel, and does not allow 
  /// socket_ids to be passed in for excluding recipients. If you wish to
  /// trigger on multiple channels, use `trigger_multi`. If you wish to exclude
  /// recipients by their socket_id, use `trigger_exclusive`. For doing both,
  /// use `trigger_multi_exclusive`.
  ///
  ///
  /// **Example:**
  /// 
  /// ```
  /// # use pusher:: Pusher;
  /// # use std::collections::HashMap;
  /// # let mut pusher = Pusher::from_env("PUSHER_URL").finalize();
  /// let mut hash_map = HashMap::new();
  /// hash_map.insert("message", "hello world");
  /// pusher.trigger("test_channel", "my_event", &hash_map);
  /// ```
  ///
  /// If you call this with <http://app.pusher.com> open, you should receive
  /// an alert saying, 'hello world'. 
  ///
  /// This method returns a `Result`. If successful, the `Ok` value will be a 
  /// `TriggeredEvents` instance, which, if you are connected to certain clusters, 
  /// holds the `event_ids` of published events. If an error has occured,
  /// the `Error` value will contain a `String` regarding what went wrong.
  pub fn trigger<Payload : rustc_serialize::Encodable>(&mut self, channel: &str, event: &str, payload: Payload)-> Result<TriggeredEvents, String> {
    let channels = vec![channel.to_string()];
    self._trigger(channels, event, payload, None)
  }

  /// This method allow you to exclude a recipient whose connection has that 
  /// `socket_id` from receiving the event. You can read more here: 
  /// <http://pusher.com/docs/duplicates>.
  ///
  /// **Example:**
  ///
  /// ```
  /// # use pusher:: Pusher;
  /// # let mut pusher = Pusher::new("id", "key", "secret").finalize();
  /// pusher.trigger_exclusive("test_channel", "my_event", "hello", "123.12");
  /// ```
  pub fn trigger_exclusive<Payload : rustc_serialize::Encodable>(&mut self, channel: &str, event: &str, payload: Payload, socket_id: &str)-> Result<TriggeredEvents, String> {
    let channels = vec![channel.to_string()];
    self._trigger(channels, event, payload, Some(socket_id.to_string()))
  }

  /// This method allow you to trigger an event on multiple channels, with a 
  /// maximum of 10.
  ///
  ///
  /// **Example:**
  ///
  /// ```
  /// # use pusher:: Pusher;
  /// # let mut pusher = Pusher::new("id", "key", "secret").finalize();
  /// let channels = vec!["test_channel", "test_channel2"];
  /// pusher.trigger_multi(&channels, "my_event", "hello");
  /// ```
  pub fn trigger_multi<Payload : rustc_serialize::Encodable>(&mut self, channels: &Vec<&str>, event: &str, payload: Payload)-> Result<TriggeredEvents, String> {
    let channel_strings = channels.into_iter().map(|c| c.to_string()).collect();
    self._trigger(channel_strings, event, payload, None)
  }

  /// This method allow you to trigger an event on multiple channels and exclude
  /// a recipient with a given `socket_id`.
  ///
  ///
  /// **Example:**
  ///
  /// ```
  /// # use pusher:: Pusher;
  /// # let mut pusher = Pusher::new("id", "key", "secret").finalize();
  /// let channels = vec!["test_channel", "test_channel2"];
  /// pusher.trigger_multi_exclusive(&channels, "my_event", "hello", "123.12");
  /// ```
  pub fn trigger_multi_exclusive<Payload : rustc_serialize::Encodable>(&mut self, channels: &Vec<&str>, event: &str, payload: Payload, socket_id: &str)-> Result<TriggeredEvents, String> {
    let channel_strings = channels.into_iter().map(|c| c.to_string()).collect();
    self._trigger(channel_strings, event, payload, Some(socket_id.to_string()))
  }

  fn _trigger<Payload : rustc_serialize::Encodable>(&mut self, channels: Vec<String>, event: &str, payload: Payload, socket_id: Option<String>) -> Result<TriggeredEvents, String> { 

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
    create_request::<TriggeredEvents>(&mut self.http_client, method, request_url, Some(&body))
  }

  /// One can use this method to get a list of all the channels in an application from the HTTP API.
  /// 
  /// Without any supplied options, all fields for each `Channel` will be `None`.
  /// If you wish to specify options for your query, see the `channels_with_options` method.
  ///
  /// An `Err` will be returned for any invalid API requests.
  ///
  /// **Example:**
  ///
  ///
  /// ```
  /// # use pusher:: Pusher;
  /// # let mut pusher = Pusher::new("id", "key", "secret").finalize();
  /// pusher.channels();
  /// //=> Ok(ChannelList { channels: {"presence-chatroom": Channel { occupied: None, user_count: None, subscription_count: None }, "presence-notifications": Channel { occupied: None, user_count: None, subscription_count: None }} })
  /// ```
  pub fn channels(&mut self) -> Result<ChannelList, String> {
    self._channels(None)
  }

  /// When adding options to your GET channels request, pass in a vector of tuples.
  /// A tuple whose first value is "filter_by_prefix" will filter the returned channels.
  /// To request more information, you can add a tuple beginning with `"info"` to that vector.
  /// To get number of users subscribed to a presence-channel, pass in a vector 
  /// with a `("info", "user_count")` tuple. 
  /// 
  /// An `Err` will be returned for any invalid API requests.
  ///
  /// **Example:**
  ///
  ///
  /// ```
  /// # use pusher:: Pusher;
  /// # let mut pusher = Pusher::new("id", "key", "secret").finalize();
  /// let channels_params = vec![("filter_by_prefix", "presence-"), ("info", "user_count")];
  /// pusher.channels_with_options(channels_params);
  /// //=> Ok(ChannelList { channels: {"presence-chatroom": Channel { occupied: None, user_count: Some(92), subscription_count: None }, "presence-notifications": Channel { occupied: None, user_count: Some(29), subscription_count: None }} })
  /// ```
  pub fn channels_with_options(&mut self, params: QueryParameters) -> Result<ChannelList, String> {
    self._channels(Some(params))
  }

  fn _channels(&mut self, params: Option<QueryParameters>) -> Result<ChannelList, String>{
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

  /// This method gets the state of a single channel.
  /// 
  /// Without any options specified, only the `occupied` field of the `Channel` instance
  /// will have a value. To specify options, see the `channel_with_options` method.
  ///
  /// An `Err` will be returned for any invalid API requests.
  ///
  /// **Example:**
  ///
  ///
  /// ```
  /// # use pusher:: Pusher;
  /// # let mut pusher = Pusher::new("id", "key", "secret").finalize();
  /// pusher.channel("presence-chatroom");
  /// //=> Ok(Channel { occupied: Some(true), user_count: None, subscription_count: None })
  /// ```
  pub fn channel(&mut self, channel_name: &str) -> Result<Channel, String>{
    self._channel(channel_name, None)
  }

  /// Pass in a vector of tuples to specify options. To request information regarding
  /// `user_count` and `subscription_count`, a tuple must have an `"info"` value 
  /// and a value containing a comma-separated list of attributes.
  ///
  /// An `Err` will be returned for any invalid API requests.
  /// 
  ///
  /// **Example:**
  ///
  ///
  /// ```
  /// # use pusher:: Pusher;
  /// # let mut pusher = Pusher::new("id", "key", "secret").finalize();
  /// let channel_params = vec![("info", "user_count,subscription_count")];
  /// pusher.channel_with_options("presence-chatroom", channel_params);
  /// //=> Ok(Channel { occupied: Some(true), user_count: Some(96), subscription_count: Some(96) })
  /// ```
  pub fn channel_with_options(&mut self, channel_name: &str, params: QueryParameters) -> Result<Channel, String> {
    self._channel(channel_name, Some(params))
  }

  fn _channel(&mut self, channel_name: &str, params: Option<QueryParameters>) -> Result<Channel, String>{
    let request_url_string = format!("{}://{}/apps/{}/channels/{}", self.scheme(), self.host, self.app_id, channel_name);
    let mut request_url = Url::parse(&request_url_string).unwrap();
    let method = "GET";
    update_request_url(method, &mut request_url, &self.key, &self.secret, timestamp(), None, params);
    create_request::<Channel>(&mut self.http_client, method, request_url, None)
  }

  /// This method retrieves the ids of users that are currently subscribed to a
  /// given presence-channel.
  ///
  /// An `Err` will be returned for any invalid API requests.
  ///
  /// **Example:**
  ///
  /// ```
  /// # use pusher:: Pusher;
  /// # let mut pusher = Pusher::new("id", "key", "secret").finalize();
  /// pusher.channel_users("presence-chatroom");
  /// //=> Ok(ChannelUserList { users: [ChannelUser { id: "red" }, ChannelUser { id: "blue" }] })
  /// ```
  pub fn channel_users(&mut self, channel_name : &str) -> Result<ChannelUserList, String> {
    let request_url_string = format!("{}://{}/apps/{}/channels/{}/users", self.scheme(), self.host, self.app_id, channel_name);
    let mut request_url = Url::parse(&request_url_string).unwrap();
    let method = "GET";
    update_request_url(method, &mut request_url, &self.key, &self.secret, timestamp(), None, None);
    create_request::<ChannelUserList>(&mut self.http_client, method, request_url, None)
  }

  /// Application security is very important so Pusher provides a mechanism for 
  /// authenticating a user’s access to a channel at the point of subscription.
  /// 
  /// This can be used both to restrict access to private channels, and in the 
  /// case of presence channels notify subscribers of who else is also subscribed via presence events.
  /// 
  /// This library provides a mechanism for generating an authentication signature 
  /// to send back to the client and authorize them.
  /// 
  /// For more information see our docs: <http://pusher.com/docs/authenticating_users>.
  ///
  /// In order to authenticate a channel, pass in the body sent to your authentication
  /// endpoint upon subscription.
  ///
  /// If an invalid body is passed in, this method will return an `Err` value.
  ///
  /// **Example With Nickel.rs:**
  ///
  /// ```ignore
  /// fn pusher_auth<'a>(req: &mut Request, res: Response<'a>) -> MiddlewareResult<'a> {
  /// 
  ///  let mut body = String::new();
  ///  req.origin.read_to_string(&mut body).unwrap(); // get the body from the request
  ///  let auth = pusher.authenticate_private_channel(&body).unwrap(); // unwrap the result of the method
  ///  res.send(auth)
  ///
  ///}
  /// ```
  pub fn authenticate_private_channel(&self, body: &String) -> Result<String, &str> {
    self.authenticate_channel(body, None)
  }

  /// Using presence channels is similar to private channels, but in order to identify a user, 
  /// clients are sent a user_id and, optionally, custom data.
  /// 
  /// In this library, one does this by passing a `pusher::Member` instance. The `id` field of this instance
  /// must be a string, and any custom data will be a `HashMap` wrapped in `Some`.
  ///
  /// **Example With Nickel.rs**
  ///
  /// ```ignore
  /// fn pusher_auth<'a>(req: &mut Request, res: Response<'a>) -> MiddlewareResult<'a> {
  ///
  ///   let mut body = String::new();
  ///   req.origin.read_to_string(&mut body).unwrap();
  ///
  ///   let mut member_data = HashMap::new();
  ///   member_data.insert("twitter", "jamiepatel");
  ///
  ///   let member = pusher::Member{user_id: "4", user_info: Some(member_data)};
  ///
  ///   let auth = pusher.authenticate_presence_channel(&body, &member).unwrap();
  ///   res.send(auth)
  ///
  /// }
  /// ```
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

  /// On your dashboard at http://app.pusher.com, you can set up webhooks to POST a 
  /// payload to your server after certain events. Such events include channels being 
  /// occupied or vacated, members being added or removed in presence-channels, or 
  /// after client-originated events. For more information see https://pusher.com/docs/webhooks.
  ///
  /// This library provides a mechanism for checking that these POST requests are 
  /// indeed from Pusher, by checking the token and authentication signature in the 
  /// header of the request.
  ///
  /// Pass in the key supplied in the `"X-Pusher-Key"` header, the signature supplied
  /// in the `"X-Pusher-Signature"` header, and the body of the request.
  ///
  /// If the webhook is valid, a `pusher::Webhook` instance will be returned within the `Result` enum.
  /// If not, an `Err` will be returned.
  ///
  /// **Example:**
  ///
  /// ```ignore
  /// # use pusher:: Pusher;
  /// # let mut pusher = Pusher::new("id", "key", "secret").finalize();
  /// pusher.webhook("supplied_key", "supplied_signature", "body")
  /// ```
  pub fn webhook(&self, key: &String, signature: &String, body: &str) -> Result<Webhook, &str> {
    if (&self.key == key) && check_signature(signature, &self.secret, body) {
      let decoded_webhook : Webhook = json::decode(&body[..]).unwrap();
      return Ok(decoded_webhook)
    }
    Err("Invalid webhook")
  }

}

#[test]
fn test_private_channel_authentication(){
  let pusher = Pusher::new("id", "278d425bdf160c739803", "7ad3773142a6692b25b8").finalize();
  let expected = "{\"auth\":\"278d425bdf160c739803:58df8b0c36d6982b82c3ecf6b4662e34fe8c25bba48f5369f135bf843651c3a4\"}".to_string();
  let body = "channel_name=private-foobar&socket_id=1234.1234".to_string();
  let result = pusher.authenticate_private_channel(&body);
  assert_eq!(result.unwrap(), expected)
}

#[test]
fn test_presence_channel_authentication(){
  let pusher = Pusher::new("id", "278d425bdf160c739803", "7ad3773142a6692b25b8").finalize();
  let expected = "{\"auth\":\"278d425bdf160c739803:48dac51d2d7569e1e9c0f48c227d4b26f238fa68e5c0bb04222c966909c4f7c4\",\"channel_data\":\"{\\\"user_id\\\":\\\"10\\\",\\\"user_info\\\":{\\\"name\\\":\\\"Mr. Pusher\\\"}}\"}";
  let expected_encoded : HashMap<String, String> = json::decode(expected).unwrap(); 
  let mut member_data = HashMap::new();
  member_data.insert("name", "Mr. Pusher");
  let presence_data = Member{user_id: "10", user_info: Some(member_data)};
  let body = "channel_name=presence-foobar&socket_id=1234.1234".to_string();
  let result_json = pusher.authenticate_presence_channel(&body, &presence_data);
  let result_decoded : HashMap<String, String> = json::decode(&result_json.unwrap()).unwrap();
  
  assert_eq!(result_decoded["auth"], expected_encoded["auth"]);
  assert_eq!(result_decoded["channel_data"], expected_encoded["channel_data"]);
}

#[test]
fn test_socket_id_validation(){
  let pusher = Pusher::new("id", "278d425bdf160c739803", "7ad3773142a6692b25b8").finalize();
  let body = "channel_name=private-foobar&socket_id=12341234".to_string();
  let result = pusher.authenticate_private_channel(&body);
  assert_eq!(result.unwrap_err(), "Invalid socket_id")
}

#[test]
fn test_client_webhook_validation(){
  let pusher = Pusher::new("id", "key", "secret").finalize();
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
  let pusher = Pusher::new("id", "key", "secret").finalize();
  let key = "narr you're going down!".to_string();
  let signature = "2677ad3e7c090b2fa2c0fb13020d66d5420879b8316eb356a2d60fb9073bc778".to_string();
  let result = pusher.webhook(&key, &signature,"{\"hello\":\"world\"}");
  assert_eq!(result.unwrap_err(), "Invalid webhook")
}

#[test]
fn test_webhook_improper_signature_case(){
  let pusher = Pusher::new("id", "key", "secret").finalize();
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

#[test]
fn test_initialization_from_url(){
  let pusher = Pusher::from_url("https://key:secret@api.host.com/apps/id").finalize();
  assert_eq!(pusher.key, "key");
  assert_eq!(pusher.secret, "secret");
  assert_eq!(pusher.host, "api.host.com");
  assert_eq!(pusher.secure, true);
  assert_eq!(pusher.app_id, "id")
}

#[test]
fn test_initialization_from_env(){
  env::set_var("PUSHER_URL", "https://key:secret@api.host.com/apps/id");
  let pusher = Pusher::from_env("PUSHER_URL").finalize();
  assert_eq!(pusher.key, "key");
  assert_eq!(pusher.secret, "secret");
  assert_eq!(pusher.host, "api.host.com");
  assert_eq!(pusher.secure, true);
  assert_eq!(pusher.app_id, "id") 
}


