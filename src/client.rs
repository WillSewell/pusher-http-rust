use hyper::client::connect::Connect;
use hyper::client::HttpConnector;
use hyper::Client;
use std::env;
use std::fmt::Debug;
use url::Url;

use crate::Error;

use super::json_structures::*;
use super::request::*;
use super::request_url::*;
use super::signature::*;
use super::util::*;

#[cfg(feature = "rustls")]
mod rustls;

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

    /// This method changes the cluster to which API requests will be made.
    ///
    /// ```
    /// # use pusher::PusherBuilder;
    /// let pusher = PusherBuilder::new("id", "key", "secret").cluster("eu").finalize();
    /// ```
    pub fn cluster(self, cluster: &str) -> PusherBuilder<C> {
        self.host(&format!("api-{cluster}.pusher.com"))
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

impl<C> Debug for PusherBuilder<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PusherBuilder")
            .field("app_id", &self.app_id)
            .field("key", &self.key)
            .field("secret", &"<redacted>")
            .field("host", &self.host)
            .field("secure", &self.secure)
            .field("http_client", &self.http_client)
            .finish()
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
    ) -> Result<TriggeredEvents, Error> {
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
    ) -> Result<TriggeredEvents, Error> {
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
    ) -> Result<TriggeredEvents, Error> {
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
    ) -> Result<TriggeredEvents, Error> {
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
    ) -> Result<TriggeredEvents, Error> {
        validate_event(event)?;
        validate_channels(&channels)?;

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
            return Err(Error::EventDataTooLarge(body.len()));
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
    pub async fn channels(&self) -> Result<ChannelList, Error> {
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
    ) -> Result<ChannelList, Error> {
        self._channels(Some(params)).await
    }

    async fn _channels(&self, params: Option<QueryParameters>) -> Result<ChannelList, Error> {
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
    pub async fn channel(&self, channel_name: &str) -> Result<Channel, Error> {
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
    ) -> Result<Channel, Error> {
        self._channel(channel_name, Some(params)).await
    }

    async fn _channel(
        &self,
        channel_name: &str,
        params: Option<QueryParameters>,
    ) -> Result<Channel, Error> {
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
    pub async fn channel_users(&self, channel_name: &str) -> Result<ChannelUserList, Error> {
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
    /// authenticating a user.
    pub fn authenticate_user(
        &self,
        socket_id: &str,
        user: User,
    ) -> Result<UserAuthResponse, Error> {
        validate_socket_id(socket_id)?;

        let user_data = serde_json::to_string(&user).unwrap();

        let to_sign = format!("{socket_id}::user::{user_data}");

        let auth = compute_auth_key(&self.key, &self.secret, &to_sign);
        Ok(UserAuthResponse { auth, user_data })
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
    ) -> Result<ChannelAuthResponse, Error> {
        validate_socket_id(socket_id)?;

        let to_sign = format!("{socket_id}:{channel_name}");
        let auth = compute_auth_key(&self.key, &self.secret, &to_sign);
        Ok(ChannelAuthResponse {
            auth,
            channel_data: None,
            shared_secret: None,
        })
    }

    /// Using presence channels is similar to private channels, but in order to identify a user,
    /// clients are sent a user_id and, optionally, custom data.
    ///
    /// In this library, one does this by passing a `pusher::Member` instance. The `id` field of this instance
    /// must be a string, and any custom data will be a `HashMap` wrapped in `Some`.
    ///
    /// **Example with hyper**
    ///
    /// async fn pusher_auth(req: Request<Body>) -> Result<Response<Body>, Error> {
    ///   let body = to_bytes(req).await.unwrap();
    ///   let params = parse(body.as_ref()).into_owned().collect::<HashMap<String, String>>();
    ///   let channel_name = params.get("channel_name").unwrap();
    ///   let socket_id = params.get("socket_id").unwrap();
    ///
    ///   let mut member_data = HashMap::new();
    ///   member_data.insert("twitter", "jamiepatel");
    ///   let member = pusher::Member{user_id: "4", user_info: Some(member_data)};
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
    ) -> Result<ChannelAuthResponse, Error> {
        validate_socket_id(socket_id)?;

        let channel_data = serde_json::to_string(member).expect("Serializationg to be infallible");
        let to_sign = format!("{socket_id}:{channel_name}:{channel_data}");

        let auth = compute_auth_key(&self.key, &self.secret, &to_sign);
        Ok(ChannelAuthResponse {
            channel_data: Some(channel_data),
            auth,
            shared_secret: None,
        })
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
        if self.key == key && verify(signature, &self.secret, body) {
            let decoded_webhook: Webhook = serde_json::from_str(&body[..]).unwrap();
            return Ok(decoded_webhook);
        }
        Err("Invalid webhook")
    }
}

impl<C> Debug for Pusher<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pusher")
            .field("app_id", &self.app_id)
            .field("key", &self.key)
            .field("secret", &"<redacted>")
            .field("host", &self.host)
            .field("secure", &self.secure)
            .field("http_client", &self.http_client)
            .finish()
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
        let expected =
            "278d425bdf160c739803:58df8b0c36d6982b82c3ecf6b4662e34fe8c25bba48f5369f135bf843651c3a4"
                .to_string();
        let result = pusher.authenticate_private_channel("private-foobar", "1234.1234");
        assert_eq!(result.unwrap().auth, expected)
    }

    #[test]
    fn test_presence_channel_authentication() {
        let pusher =
            PusherBuilder::new("id", "278d425bdf160c739803", "7ad3773142a6692b25b8").finalize();
        let expected = "{\"auth\":\"278d425bdf160c739803:48dac51d2d7569e1e9c0f48c227d4b26f238fa68e5c0bb04222c966909c4f7c4\",\"channel_data\":\"{\\\"user_id\\\":\\\"10\\\",\\\"user_info\\\":{\\\"name\\\":\\\"Mr. Pusher\\\"}}\"}";
        let expected_encoded: HashMap<String, String> = serde_json::from_str(expected).unwrap();
        let mut member_data = HashMap::new();
        member_data.insert("name", "Mr. Pusher");
        let presence_data = Member {
            user_id: "10",
            user_info: Some(member_data),
        };
        let result = pusher
            .authenticate_presence_channel("presence-foobar", "1234.1234", &presence_data)
            .unwrap();

        assert_eq!(result.auth, expected_encoded["auth"]);
        assert_eq!(
            result.channel_data.unwrap(),
            expected_encoded["channel_data"]
        );
    }

    #[test]
    fn test_socket_id_validation() {
        let pusher =
            PusherBuilder::new("id", "278d425bdf160c739803", "7ad3773142a6692b25b8").finalize();
        let result = pusher.authenticate_private_channel("private-foobar", "12341234");
        assert!(matches!(result, Err(Error::InvalidSocketId(_))));
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
        assert!(matches!(res, Err(Error::TooManyChannels(_))));
    }

    #[tokio::test]
    async fn test_channel_format_validation() {
        let pusher = PusherBuilder::new("id", "key", "secret").finalize();
        let res = pusher.trigger("w000^$$£@@@", "yolo", "woot").await;
        assert!(matches!(res, Err(Error::InvalidChannelName(_))));
    }

    #[tokio::test]
    async fn test_channel_length_validation() {
        let pusher = PusherBuilder::new("id", "key", "secret").finalize();
        let mut channel = "".to_string();

        for _ in 1..202 {
            channel = channel + "a"
        }

        let res = pusher.trigger(&channel, "yolo", "woot").await;
        assert!(matches!(res, Err(Error::InvalidChannelName(_))));
    }

    #[tokio::test]
    async fn test_trigger_payload_size_validation() {
        let pusher = PusherBuilder::new("id", "key", "secret").finalize();
        let mut data = "".to_string();

        for _ in 1..10242 {
            data = data + "a"
        }

        let res = pusher.trigger("yolo", "new_yolo", &data).await;
        assert!(matches!(res, Err(Error::EventDataTooLarge(_))));
    }

    #[tokio::test]
    async fn test_event_name_length_validation() {
        let pusher = PusherBuilder::new("id", "key", "secret").finalize();
        let mut event = "".to_string();

        for _ in 1..202 {
            event = event + "a"
        }

        let res = pusher.trigger("yolo", &event, "woot").await;
        assert!(matches!(res, Err(Error::InvalidEventName(_))));
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
