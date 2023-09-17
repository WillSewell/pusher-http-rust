use hyper::client::connect::Connect;
use hyper::client::HttpConnector;
use hyper::Client;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use url::Url;

use super::json_structures::*;
use super::request::*;
use super::request_url::*;
use super::signature::*;
use super::util::*;

/// A client to interact with Pusher's HTTP API to trigger, query application state,
/// authenticate private- or presence-channels, and validate webhooks.
pub struct Pusher<C> {
    /// Your app_id from <http://app.pusher.com>
    pub app_id: String,
    /// Your key from <http://app.pusher.com>
    pub key: String,
    /// Your secret from <http://app.pusher.com>
    pub secret: String,
    /// The host[:port] you wish to connect to. Defaults to api.pusherapp.com
    pub host: String,
    /// If true, requests are made over HTTPS.
    pub secure: bool,
    /// The underlying Hyper HTTP client.
    pub http_client: Client<C>,
}

/// An ephemeral object upon which to pass configuration options to when
/// initializing a Pusher instance.
pub struct PusherBuilder<C> {
    pub app_id: String,
    pub key: String,
    pub secret: String,
    pub host: String,
    pub secure: bool,
    pub http_client: Client<C>,
}

impl PusherBuilder<HttpConnector> {
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
    /// # use pusher::PusherBuilder;
    /// let pusher = PusherBuilder::new("id", "key", "secret").host("foo.bar.com").finalize();
    /// ```
    pub fn new(app_id: &str, key: &str, secret: &str) -> PusherBuilder<HttpConnector> {
        let http_client = Client::new();
        PusherBuilder::new_with_client(http_client, app_id, key, secret)
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
    /// # use pusher::PusherBuilder;
    /// PusherBuilder::from_url("http://key:secret@api.host.com/apps/id").finalize();
    /// ```
    pub fn from_url(url: &str) -> PusherBuilder<HttpConnector> {
        let http_client = Client::new();
        PusherBuilder::from_url_with_client(http_client, url)
    }

    /// Initializes a client from an environment variable Pusher URL.
    ///
    /// This returns a `PusherBuilder`, on which to set configuration options
    /// before calling `finalize()`.
    ///
    ///
    /// **Example:**
    ///
    /// ```ignore
    /// # use pusher::Pusher;
    /// Pusher::from_env("PUSHER_URL").finalize();
    /// ```
    pub fn from_env(key: &str) -> PusherBuilder<HttpConnector> {
        let http_client = Client::new();
        PusherBuilder::from_env_with_client(http_client, key)
    }
}

impl<C> PusherBuilder<C> {
    /// Initializes the client with a specified hyper::client::connect::Connect.
    /// See pusher::PusherBuilder::new for more detail.
    pub fn new_with_client(
        http_client: Client<C>,
        app_id: &str,
        key: &str,
        secret: &str,
    ) -> PusherBuilder<C> {
        PusherBuilder {
            app_id: app_id.to_string(),
            key: key.to_string(),
            secret: secret.to_string(),
            host: "api.pusherapp.com".to_string(),
            secure: false,
            http_client,
        }
    }

    /// Initializes the client with a specified hyper::client::connect::Connect.
    /// See pusher::PusherBuilder::from_url for more detail.
    pub fn from_url_with_client(http_client: Client<C>, url: &str) -> PusherBuilder<C> {
        let pusher_url = Url::parse(url).unwrap();

        let key = pusher_url.username();
        let secret = pusher_url.password().unwrap();
        let host = pusher_url.host().unwrap();
        let v: Vec<&str> = pusher_url.path_segments().unwrap().collect();
        let app_id = v[1];
        let secure = pusher_url.scheme() == "https";

        PusherBuilder {
            app_id: app_id.to_string(),
            key: key.to_string(),
            secret: secret.to_string(),
            host: host.to_string(),
            secure,
            http_client,
        }
    }

    /// Initializes the client with a specified hyper::client::connect::Connect.
    /// See pusher::PusherBuilder::from_env for more detail.
    pub fn from_env_with_client(http_client: Client<C>, key: &str) -> PusherBuilder<C> {
        let url_opt = env::var_os(key).unwrap();
        let os_url = url_opt.to_str();
        let url = os_url.unwrap();
        PusherBuilder::from_url_with_client(http_client, url)
    }

    /// This method changes the host to which API requests will be made.
    /// This defaults to `api.pusherapp.com`.
    ///
    /// ```
    /// # use pusher::PusherBuilder;
    /// let pusher = PusherBuilder::new("id", "key", "secret").host("foo.bar.com").finalize();
    /// ```
    pub fn host(mut self, host: &str) -> PusherBuilder<C> {
        self.host = host.to_string();
        self
    }

    /// This method determines whether requests will be made over HTTPS. This defaults to `false`.
    ///
    /// ```
    /// # use pusher::PusherBuilder;
    /// let pusher = PusherBuilder::new("id", "key", "secret").secure().finalize();
    /// ```
    pub fn secure(mut self) -> PusherBuilder<C> {
        self.secure = true;
        self
    }

    /// If you wish to configure a [Hyper client](http://hyper.rs/hyper/hyper/client/struct.Client.html),
    /// pass it in to this method.
    pub fn client(mut self, http_client: Client<C>) -> PusherBuilder<C> {
        self.http_client = http_client;
        self
    }

    /// This method actually creates the `Pusher` instance from your chained configuration.
    pub fn finalize(self) -> Pusher<C> {
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

impl<C: Connect + Clone + Send + Sync + 'static> Pusher<C> {
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
    /// # use pusher::PusherBuilder;
    /// # use std::collections::HashMap;
    /// # let pusher = PusherBuilder::new("id", "key", "secret").finalize();
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
    pub async fn trigger<S: serde::Serialize>(
        &self,
        channel: &str,
        event: &str,
        payload: S,
    ) -> Result<TriggeredEvents, String> {
        let channels = vec![channel.to_string()];
        self._trigger(channels, event, payload, None).await
    }

    /// This method allow you to exclude a recipient whose connection has that
    /// `socket_id` from receiving the event. You can read more here:
    /// <http://pusher.com/docs/duplicates>.
    ///
    /// **Example:**
    ///
    /// ```
    /// # use pusher::PusherBuilder;
    /// # let pusher = PusherBuilder::new("id", "key", "secret").finalize();
    /// pusher.trigger_exclusive("test_channel", "my_event", "hello", "123.12");
    /// ```
    pub async fn trigger_exclusive<S: serde::Serialize>(
        &self,
        channel: &str,
        event: &str,
        payload: S,
        socket_id: &str,
    ) -> Result<TriggeredEvents, String> {
        let channels = vec![channel.to_string()];
        self._trigger(channels, event, payload, Some(socket_id.to_string()))
            .await
    }

    /// This method allow you to trigger an event on multiple channels, with a
    /// maximum of 10.
    ///
    ///
    /// **Example:**
    ///
    /// ```
    /// # use pusher::PusherBuilder;
    /// # let pusher = PusherBuilder::new("id", "key", "secret").finalize();
    /// let channels = vec!["test_channel", "test_channel2"];
    /// pusher.trigger_multi(&channels, "my_event", "hello");
    /// ```
    pub async fn trigger_multi<S: serde::Serialize>(
        &self,
        channels: &[&str],
        event: &str,
        payload: S,
    ) -> Result<TriggeredEvents, String> {
        let channel_strings = channels.iter().map(|c| (*c).to_string()).collect();
        self._trigger(channel_strings, event, payload, None).await
    }

    /// This method allow you to trigger an event on multiple channels and exclude
    /// a recipient with a given `socket_id`.
    ///
    ///
    /// **Example:**
    ///
    /// ```
    /// # use pusher::PusherBuilder;
    /// # let pusher = PusherBuilder::new("id", "key", "secret").finalize();
    /// let channels = vec!["test_channel", "test_channel2"];
    /// pusher.trigger_multi_exclusive(&channels, "my_event", "hello", "123.12");
    /// ```
    pub async fn trigger_multi_exclusive<S: serde::Serialize>(
        &self,
        channels: &[&str],
        event: &str,
        payload: S,
        socket_id: &str,
    ) -> Result<TriggeredEvents, String> {
        let channel_strings = channels.iter().map(|c| (*c).to_string()).collect();
        self._trigger(channel_strings, event, payload, Some(socket_id.to_string()))
            .await
    }

    async fn _trigger<S: serde::Serialize>(
        &self,
        channels: Vec<String>,
        event: &str,
        payload: S,
        socket_id: Option<String>,
    ) -> Result<TriggeredEvents, String> {
        if event.len() > 200 {
            return Err("Event name is limited to 200 chars".to_string());
        }

        if let Err(message) = validate_channels(&channels) {
            return Err(message);
        }

        let request_url_string = format!(
            "{}://{}/apps/{}/events",
            self.scheme(),
            self.host,
            self.app_id
        );
        let mut request_url = Url::parse(&request_url_string).unwrap();

        let json_payload = serde_json::to_string(&payload).unwrap();

        let raw_body = TriggerEventData {
            name: event.to_string(),
            channels,
            data: json_payload,
            socket_id,
        };

        let body = serde_json::to_string(&raw_body).unwrap();

        if body.len() > 10240 {
            return Err("Data must be smaller than 10kb".to_string());
        }

        let method = "POST";
        let query = build_query(
            method,
            request_url.path(),
            &self.key,
            &self.secret,
            timestamp(),
            Some(&body),
            None,
        );
        request_url.set_query(Some(&query));
        send_request::<C, TriggeredEvents>(&self.http_client, method, request_url, Some(body)).await
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
    /// # use pusher::PusherBuilder;
    /// # let pusher = PusherBuilder::new("id", "key", "secret").finalize();
    /// pusher.channels();
    /// //=> Ok(ChannelList { channels: {"presence-chatroom": Channel { occupied: None, user_count: None, subscription_count: None }, "presence-notifications": Channel { occupied: None, user_count: None, subscription_count: None }} })
    /// ```
    pub async fn channels(&self) -> Result<ChannelList, String> {
        self._channels(None).await
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
    /// # use pusher::PusherBuilder;
    /// # let pusher = PusherBuilder::new("id", "key", "secret").finalize();
    /// let channels_params = vec![("filter_by_prefix".to_string(), "presence-".to_string()), ("info".to_string(), "user_count".to_string())];
    /// pusher.channels_with_options(channels_params);
    /// //=> Ok(ChannelList { channels: {"presence-chatroom": Channel { occupied: None, user_count: Some(92), subscription_count: None }, "presence-notifications": Channel { occupied: None, user_count: Some(29), subscription_count: None }} })
    /// ```
    pub async fn channels_with_options(
        &self,
        params: QueryParameters,
    ) -> Result<ChannelList, String> {
        self._channels(Some(params)).await
    }

    async fn _channels(&self, params: Option<QueryParameters>) -> Result<ChannelList, String> {
        let request_url_string = format!(
            "{}://{}/apps/{}/channels",
            self.scheme(),
            self.host,
            self.app_id
        );
        let mut request_url = Url::parse(&request_url_string).unwrap();
        let method = "GET";
        let query = build_query(
            method,
            request_url.path(),
            &self.key,
            &self.secret,
            timestamp(),
            None,
            params,
        );
        request_url.set_query(Some(&query));
        send_request::<C, ChannelList>(&self.http_client, method, request_url, None).await
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
    /// # use pusher::PusherBuilder;
    /// # let pusher = PusherBuilder::new("id", "key", "secret").finalize();
    /// pusher.channel("presence-chatroom");
    /// //=> Ok(Channel { occupied: Some(true), user_count: None, subscription_count: None })
    /// ```
    pub async fn channel(&self, channel_name: &str) -> Result<Channel, String> {
        self._channel(channel_name, None).await
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
    /// # use pusher::PusherBuilder;
    /// # let pusher = PusherBuilder::new("id", "key", "secret").finalize();
    /// let channel_params = vec![("info".to_string(), "user_count,subscription_count".to_string())];
    /// pusher.channel_with_options("presence-chatroom", channel_params);
    /// //=> Ok(Channel { occupied: Some(true), user_count: Some(96), subscription_count: Some(96) })
    /// ```
    pub async fn channel_with_options(
        &self,
        channel_name: &str,
        params: QueryParameters,
    ) -> Result<Channel, String> {
        self._channel(channel_name, Some(params)).await
    }

    async fn _channel(
        &self,
        channel_name: &str,
        params: Option<QueryParameters>,
    ) -> Result<Channel, String> {
        let request_url_string = format!(
            "{}://{}/apps/{}/channels/{}",
            self.scheme(),
            self.host,
            self.app_id,
            channel_name
        );
        let mut request_url = Url::parse(&request_url_string).unwrap();
        let method = "GET";
        let query = build_query(
            method,
            request_url.path(),
            &self.key,
            &self.secret,
            timestamp(),
            None,
            params,
        );
        request_url.set_query(Some(&query));
        send_request::<C, Channel>(&self.http_client, method, request_url, None).await
    }

    /// This method retrieves the ids of users that are currently subscribed to a
    /// given presence-channel.
    ///
    /// An `Err` will be returned for any invalid API requests.
    ///
    /// **Example:**
    ///
    /// ```
    /// # use pusher::PusherBuilder;
    /// # let pusher = PusherBuilder::new("id", "key", "secret").finalize();
    /// pusher.channel_users("presence-chatroom");
    /// //=> Ok(ChannelUserList { users: [ChannelUser { id: "red" }, ChannelUser { id: "blue" }] })
    /// ```
    pub async fn channel_users(&self, channel_name: &str) -> Result<ChannelUserList, String> {
        let request_url_string = format!(
            "{}://{}/apps/{}/channels/{}/users",
            self.scheme(),
            self.host,
            self.app_id,
            channel_name
        );
        let mut request_url = Url::parse(&request_url_string).unwrap();
        let method = "GET";
        let query = build_query(
            method,
            request_url.path(),
            &self.key,
            &self.secret,
            timestamp(),
            None,
            None,
        );
        request_url.set_query(Some(&query));
        send_request::<C, ChannelUserList>(&self.http_client, method, request_url, None).await
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
    /// **Example with hyper:**
    ///
    /// ```ignore
    /// async fn pusher_auth(req: Request<Body>) -> Result<Response<Body>, Error> {
    ///   let body = to_bytes(req).await.unwrap();
    ///   let params = parse(body.as_ref()).into_owned().collect::<HashMap<String, String>>();
    ///   let channel_name = params.get("channel_name").unwrap();
    ///   let socket_id = params.get("socket_id").unwrap();
    ///   let auth_signature = pusher.authenticate_private_channel(channel_name, socket_id).unwrap();
    ///   Ok(Response::new(auth_signature.into()))
    /// }
    /// ```
    pub fn authenticate_private_channel(
        &self,
        channel_name: &str,
        socket_id: &str,
    ) -> Result<String, &str> {
        self.authenticate_channel(channel_name, socket_id, None)
    }

    /// Using presence channels is similar to private channels, but in order to identify a user,
    /// clients are sent a user_id and, optionally, custom data.
    ///
    /// In this library, one does this by passing a `pusher::Member` instance. The `id` field of this instance
    /// must be a string, and any custom data will be a `HashMap` wrapped in `Some`.
    ///
    /// **Example with hyper**
    ///
    /// ```ignore
    /// async fn pusher_auth(req: Request<Body>) -> Result<Response<Body>, Error> {
    ///   let body = to_bytes(req).await.unwrap();
    ///   let params = parse(body.as_ref()).into_owned().collect::<HashMap<String, String>>();
    ///   let channel_name = params.get("channel_name").unwrap();
    ///   let socket_id = params.get("socket_id").unwrap();
    ///
    ///   let mut member_data = HashMap::new();
    ///   member_data.insert("twitter", "jamiepatel");
    ///   let member = pusher::Member{user_id: "4", user_info: Some(member_data), watchlist: None};
    ///
    ///   let auth_signature = pusher.authenticate_presence_channel(channel_name, socket_id, &member).unwrap();
    ///   Ok(Response::new(auth_signature.into()))
    /// }
    /// ```
    pub fn authenticate_presence_channel(
        &self,
        channel_name: &str,
        socket_id: &str,
        member: &Member,
    ) -> Result<String, &str> {
        self.authenticate_channel(channel_name, socket_id, Some(member))
    }

    fn authenticate_channel(
        &self,
        channel_name: &str,
        socket_id: &str,
        member: Option<&Member>,
    ) -> Result<String, &str> {
        let socket_id_regex = Regex::new(r"\A\d+\.\d+\z").unwrap(); // how to make this global?

        if !socket_id_regex.is_match(&socket_id) {
            return Err("Invalid socket_id");
        }

        let mut to_sign = format!("{}:{}", socket_id, channel_name);

        let mut auth_map = HashMap::new();

        if let Some(presence_member) = member {
            let json_member = serde_json::to_string(presence_member).unwrap();
            to_sign = format!("{}:{}", to_sign, json_member);
            auth_map.insert("channel_data", json_member);
        }

        create_auth_token(&mut auth_map, &self.key, &self.secret, &to_sign);
        Ok(serde_json::to_string(&auth_map).unwrap())
    }

    /// This method allows you to authenticate a user once per connection session.
    /// Authenticating a user gives your application access to user based
    /// features such as sending events to a user based on user id or terminating
    /// a user’s connections immediately.
    ///
    /// **Example with hyper**
    ///
    /// ```ignore
    /// async fn pusher_user_auth(req: Request<Body>) -> Result<Response<Body>, Error> {
    ///   let body = to_bytes(req).await.unwrap();
    ///   let params = parse(body.as_ref()).into_owned().collect::<HashMap<String, String>>();
    ///   let socket_id = params.get("socket_id").unwrap();
    ///
    ///   let mut member_data = HashMap::new();
    ///   member_data.insert("username".to_string(), "nikhilpatel".to_string());
    ///   member_data.insert("group".to_string(), "the-cool-one".to_string());
    ///   let watchlist = vec!["some-user-id", "some-other-user-id"];
    ///   let member = pusher::Member {
    ///       user_id: "10",
    ///       user_info: Some(member_data),
    ///       watchlist: Some(watchlist)
    ///   };
    ///
    ///   let auth_signature = pusher.authenticate_user(socket_id, member).unwrap();
    ///   Ok(Response::new(auth_signature.into()))
    /// }
    /// ```
    pub fn authenticate_user(
        &self,
        socket_id: &str,
        user: &User,
    ) -> Result<String, &str> {
        let socket_id_regex = Regex::new(r"\A\d+\.\d+\z").unwrap(); // how to make this global?

        if !socket_id_regex.is_match(&socket_id) {
            return Err("Invalid socket_id");
        }

        let json_user = serde_json::to_string(user).unwrap();
        
        let to_sign = format!("{}:user:{}", socket_id, json_user);

        let mut auth_map = HashMap::new();
        auth_map.insert("user_data", json_user);

        create_auth_token(&mut auth_map, &self.key, &self.secret, &to_sign);
        Ok(serde_json::to_string(&auth_map).unwrap())
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
    /// # use pusher::PusherBuilder;
    /// # let pusher = PusherBuilder::new("id", "key", "secret").finalize();
    /// pusher.webhook("supplied_key", "supplied_signature", "body")
    /// ```
    pub fn webhook(&self, key: &str, signature: &str, body: &str) -> Result<Webhook, &str> {
        if self.key == key && check_signature(signature, &self.secret, body) {
            let decoded_webhook: Webhook = serde_json::from_str(&body[..]).unwrap();
            return Ok(decoded_webhook);
        }
        Err("Invalid webhook")
    }
}

#[cfg(test)]
mod tests {
    extern crate tokio;

    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_private_channel_authentication() {
        let pusher =
            PusherBuilder::new("id", "278d425bdf160c739803", "7ad3773142a6692b25b8").finalize();
        let expected = "{\"auth\":\"278d425bdf160c739803:58df8b0c36d6982b82c3ecf6b4662e34fe8c25bba48f5369f135bf843651c3a4\"}".to_string();
        let result = pusher.authenticate_private_channel("private-foobar", "1234.1234");
        assert_eq!(result.unwrap(), expected)
    }

    #[test]
    fn test_presence_channel_authentication() {
        let pusher =
            PusherBuilder::new("id", "278d425bdf160c739803", "7ad3773142a6692b25b8").finalize();
        let expected = "{\"auth\":\"278d425bdf160c739803:57a64aa30b116d4d495d6bb56bf187698a3298c3d4959770ffd38cb05bc504fc\",\"channel_data\":\"{\\\"user_id\\\":\\\"10\\\",\\\"user_info\\\":{\\\"clan\\\":\\\"Vikings\\\",\\\"name\\\":\\\"Mr. Pusher\\\"}}\"}";
        let expected_encoded: HashMap<String, String> = serde_json::from_str(expected).unwrap();
        let mut member_data = HashMap::new();
        member_data.insert("name", "Mr. Pusher");
        member_data.insert("clan", "Vikings");
        let presence_data = Member {
            user_id: "10",
            user_info: Some(member_data),
        };
        let result_json =
            pusher.authenticate_presence_channel("presence-foobar", "1234.1234", &presence_data);
        let result_decoded: HashMap<String, String> =
            serde_json::from_str(&result_json.unwrap()).unwrap();

        assert_eq!(result_decoded["auth"], expected_encoded["auth"]);
        assert_eq!(
            result_decoded["channel_data"],
            expected_encoded["channel_data"]
        );
    }

    #[test]
    fn test_user_authentication() {
        let pusher =
            PusherBuilder::new("id", "278d425bdf160c739803", "7ad3773142a6692b25b8").finalize();
        let expected = "{\"auth\":\"278d425bdf160c739803:2a475eafe42c10a641c2ae25156e14d68de2e39135f82fe27cb01c8926af22f8\",\"user_data\":\"{\\\"id\\\":\\\"10\\\",\\\"user_info\\\":{\\\"age\\\":\\\"101\\\",\\\"name\\\":\\\"Mr. Pusher\\\"},\\\"watchlist\\\":[\\\"43\\\",\\\"513\\\",\\\"12\\\"]}\"}";
        let expected_encoded: HashMap<String, String> = serde_json::from_str(expected).unwrap();
        let mut member_data = HashMap::new();
        member_data.insert("name", "Mr. Pusher");
        member_data.insert("age", "101");
        let watchlist = vec!["43", "513", "12"];
        let user = User {
            id: "10",
            user_info: Some(member_data),
            watchlist: Some(watchlist),
        };
        let result_json =
            pusher.authenticate_user("1234.1234", &user);
        let result_decoded: HashMap<String, String> =
            serde_json::from_str(&result_json.unwrap()).unwrap();

        assert_eq!(result_decoded["auth"], expected_encoded["auth"]);
        assert_eq!(
            result_decoded["user_data"],
            expected_encoded["user_data"]
        );
    }

    #[test]
    fn test_socket_id_validation() {
        let pusher =
            PusherBuilder::new("id", "278d425bdf160c739803", "7ad3773142a6692b25b8").finalize();
        let result = pusher.authenticate_private_channel("private-foobar", "12341234");
        assert_eq!(result.unwrap_err(), "Invalid socket_id")
    }

    #[test]
    fn test_client_webhook_validation() {
        let pusher = PusherBuilder::new("id", "key", "secret").finalize();
        let key = "key".to_string();
        let signature =
            "05a115b7898e4956cf46df2dd2822b3b913a4255343acd82d31609f222765c6a".to_string();
        let result = pusher.webhook(
            &key,
            &signature,
            "{\"time_ms\":1327078148132,\"events\":[{\"name\":\"event_name\",\"some\":\"data\"}]}",
        );

        let webhook = result.unwrap();
        assert_eq!(webhook.time_ms, 1327078148132);
        assert_eq!(webhook.events[0]["name"], "event_name");
        assert_eq!(webhook.events[0]["some"], "data")
    }

    #[test]
    fn test_webhook_improper_key_case() {
        let pusher = PusherBuilder::new("id", "key", "secret").finalize();
        let key = "narr you're going down!".to_string();
        let signature =
            "2677ad3e7c090b2fa2c0fb13020d66d5420879b8316eb356a2d60fb9073bc778".to_string();
        let result = pusher.webhook(&key, &signature, "{\"hello\":\"world\"}");
        assert_eq!(result.unwrap_err(), "Invalid webhook")
    }

    #[test]
    fn test_webhook_improper_signature_case() {
        let pusher = PusherBuilder::new("id", "key", "secret").finalize();
        let key = "key".to_string();
        let signature = "26c778".to_string();
        let result = pusher.webhook(&key, &signature, "{\"hello\":\"world\"}");
        assert_eq!(result.unwrap_err(), "Invalid webhook")
    }

    #[tokio::test]
    async fn test_channel_number_validation() {
        let pusher = PusherBuilder::new("id", "key", "secret").finalize();
        let channels = vec!["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11"];
        let res = pusher.trigger_multi(&channels, "yolo", "woot").await;
        assert_eq!(res.unwrap_err(), "Cannot trigger on more than 10 channels")
    }

    #[tokio::test]
    async fn test_channel_format_validation() {
        let pusher = PusherBuilder::new("id", "key", "secret").finalize();
        let res = pusher.trigger("w000^$$£@@@", "yolo", "woot").await;
        assert_eq!(
            res.unwrap_err(),
            "Channels must be formatted as such: ^[-a-zA-Z0-9_=@,.;]+$"
        )
    }

    #[tokio::test]
    async fn test_channel_length_validation() {
        let pusher = PusherBuilder::new("id", "key", "secret").finalize();
        let mut channel = "".to_string();

        for _ in 1..202 {
            channel = channel + "a"
        }

        let res = pusher.trigger(&channel, "yolo", "woot").await;
        assert_eq!(
            res.unwrap_err(),
            "Channel names must be under 200 characters"
        )
    }

    #[tokio::test]
    async fn test_trigger_payload_size_validation() {
        let pusher = PusherBuilder::new("id", "key", "secret").finalize();
        let mut data = "".to_string();

        for _ in 1..10242 {
            data = data + "a"
        }

        let res = pusher.trigger("yolo", "new_yolo", &data).await;
        assert_eq!(res.unwrap_err(), "Data must be smaller than 10kb")
    }

    #[tokio::test]
    async fn test_event_name_length_validation() {
        let pusher = PusherBuilder::new("id", "key", "secret").finalize();
        let mut event = "".to_string();

        for _ in 1..202 {
            event = event + "a"
        }

        let res = pusher.trigger("yolo", &event, "woot").await;
        assert_eq!(res.unwrap_err(), "Event name is limited to 200 chars")
    }

    #[test]
    fn test_initialization_from_url() {
        let pusher = PusherBuilder::from_url("https://key:secret@api.host.com/apps/id").finalize();
        assert_eq!(pusher.key, "key");
        assert_eq!(pusher.secret, "secret");
        assert_eq!(pusher.host, "api.host.com");
        assert_eq!(pusher.secure, true);
        assert_eq!(pusher.app_id, "id")
    }

    #[test]
    fn test_initialization_from_env() {
        env::set_var("PUSHER_URL", "https://key:secret@api.host.com/apps/id");
        let pusher = PusherBuilder::from_env("PUSHER_URL").finalize();
        assert_eq!(pusher.key, "key");
        assert_eq!(pusher.secret, "secret");
        assert_eq!(pusher.host, "api.host.com");
        assert_eq!(pusher.secure, true);
        assert_eq!(pusher.app_id, "id")
    }
}
