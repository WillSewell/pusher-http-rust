# Pusher HTTP Rust Library

> [!WARNING]  
> **This library is unsupported**. I (the maintainer) do not currently have the bandwidth to review PRs, address security issues etc. Forks are encouraged.

[![Build Status](https://github.com/pusher-community/pusher-http-rust/workflows/Tests/badge.svg)](https://github.com/pusher-community/pusher-http-rust/actions)
[![Crates Badge](https://img.shields.io/crates/v/pusher)](https://crates.io/crates/pusher)
[![Docs.rs Badge](https://docs.rs/pusher/badge.svg)](https://docs.rs/pusher/)

The Rust library for interacting with the Pusher HTTP API.

This package lets you trigger events to your client and query the state of your Pusher channels. When used with a server, you can validate Pusher webhooks and authenticate private- or presence-channels.

The functions that make HTTP requests are [async](https://rust-lang.github.io/async-book/), so you will need to run them with an executer like [tokio](https://tokio.rs/). The library is a wrapper around the [hyper](https://hyper.rs/) client.

In order to use this library, you need to have a free account on <http://pusher.com>. After registering, you will need the application credentials for your app.

## Table of Contents

- [Installation](#installation)
- [Getting Started](#getting-started)
- [Configuration](#configuration)
  - [Additional options](#additional-options)
- [Usage](#usage)
  - [Triggering events](#triggering-events)
  - [Excluding event recipients](#excluding-event-recipients)
  - [Authenticating Channels](#authenticating-channels)
  - [Application state](#application-state)
  - [Webhook validation](#webhook-validation)
- [Feature Support](#feature-support)
- [Developing the Library](#developing-the-library)
  - [Running the tests](#running-the-tests)
- [License](#license)

## Installation

Add to your `Cargo.toml`:

```rust
pusher="*"
```

## Supported platforms

- Rust versions 1.39 and above

## Getting Started

```rust
extern crate pusher; // imports the `pusher` module

use pusher::PusherBuilder; // brings the PusherBuilder into scope

// the functions are async, so we need a reactor running (e.g. tokio)
// this example uses "current_thread" for simplicity
#[tokio::main(flavor = "current_thread")]
async fn main() {
  // initializes a Pusher object with your app credentials
  let pusher = PusherBuilder::new("APP_ID", "KEY", "SECRET").finalize();

  // triggers an event called "my_event" on a channel called "test_channel", with the payload "hello world!"
  let result = pusher.trigger("test_channel", "my_event", "hello world!").await;
  match result {
    Ok(events) => println!("Successfully published: {:?}", events),
    Err(err) => println!("Failed to publish: {}", err),
  }
}
```

## Configuration

There easiest way to configure the library is by creating a new `Pusher` instance:

```rust
let pusher = PusherBuilder::new("id", "key", "secret").finalize();
```

`PusherBuilder::new` returns a `PusherBuilder`, on which to chain configuration methods, before calling `finalize()`.

### Additional options

#### Instantiation From URL

```rust
PusherBuilder::from_url("http://key:secret@api.host.com/apps/id").finalize();
```

#### Instantiation From Environment Variable

```rust
PusherBuilder::from_env("PUSHER_URL").finalize();
```

This is particularly relevant if you are using Pusher as a Heroku add-on, which stores credentials in a `"PUSHER_URL"` environment variable.

#### HTTPS

To ensure requests occur over HTTPS, call `secure()` before `finalize()`.

```rust
let pusher = PusherBuilder::new("id", "key", "secret").secure().finalize();
```

#### Changing Host

Calling `host()` before `finalize()` will make sure requests are sent to your specified host.

```rust
let pusher = PusherBuilder::new("id", "key", "secret").host("foo.bar.com").finalize();
```

By default, this is `"api.pusherapp.com"`.

#### Changing the underlying `hyper::client::connect::Connect`

The above functions have equivalent functions that also allow a custom [`Connect`](https://docs.rs/hyper/0.13.1/hyper/client/connect/struct.Connected.html) to be provided. E.g.:

```rust
let pusher = PusherBuilder::new_with_client(my_client, "id", "key", "secret").host("foo.bar.com").finalize();
```

## Usage

### Triggering events

It is possible to trigger an event on one or more channels. Channel names can contain only characters which are alphanumeric, `_` or `-`` and have to be at most 200 characters long. Event name can be at most 200 characters long too. It is also possbie to trigger an event to a specific user.


#### Single channel

##### `async fn trigger<S: serde::Serialize>(&self, channel: &str, event: &str, payload: S)`

|Argument   |Description   |
|:-:|:-:|
|channel `&str`   |The name of the channel you wish to trigger on.   |
|event `&str` | The name of the event you wish to trigger |
|data `S: serde::Serialize` | The payload you wish to send. Must be marshallable into JSON. |

|Return Value|Description|
|:-:|:-:|
|result `Result<TriggeredEvents, String>` | If the trigger was successful and you are connected to certain clusters, an object containing the `event_ids` field will be returned as part of a `Result`. An `Err` value will be returned if any errors were encountered.  |

###### Example

```rust
let mut hash_map = HashMap::new();
hash_map.insert("message", "hello world");

pusher.trigger("test_channel", "my_event", &hash_map).await;
```

#### Multiple channels

##### `async fn trigger_multi<S: serde::Serialize>(&self, channels: &[&str], event: &str, payload: S)`

|Argument | Description |
|:-:|:-:|
|channels `&[&str]`| A vector of channel names you wish to send an event on. The maximum length is 10.|
|event `&str` | As above.|
|data `S: serde::Serialize` |As above.|

|Return Value|Description|
|:-:|:-:|
|result `Result<TriggeredEvents, String>` | As above. |

###### Example

```rust
let channels = vec!["test_channel", "test_channel2"];

pusher.trigger_multi(&channels, "my_event", "hello").await;
```

#### Excluding event recipients

`trigger_exclusive` and `trigger_multi_exclusive` follow the patterns above, except a `socket_id` is given as the last parameter.

These methods allow you to exclude a recipient whose connection has that `socket_id` from receiving the event. You can read more [here](http://pusher.com/docs/duplicates).

###### Examples

**On one channel**:

```rust
pusher.trigger_exclusive("test_channel", "my_event", "hello", "123.12").await;
```

**On multiple channels**:

```rust
let channels = vec!["test_channel", "test_channel2"];
pusher.trigger_multi_exclusive(&channels, "my_event", "hello", "123.12").await;
```

#### Send to user

##### `async fn send_to_user<S: serde::Serialize>(&self, user_id: &str, event: &str, payload: S)`

|Argument | Description |
|:-:|:-:|
|user_id `&str`| The id of the user you wish to send an event to.|
|event `&str` | As above.|
|data `S: serde::Serialize` |As above.|

|Return Value|Description|
|:-:|:-:|
|result `Result<TriggeredEvents, String>` | As above. |

###### Example

```rust
let user_id = "10";
pusher.send_to_user(user_id, "my_event", "hello").await;
```

### Terminating user connections

Authenticating a user allows you to terminate all connections for that given user.

##### `async fn terminate_user_connections(&self, user_id: &str)`

|Argument | Description |
|:-:|:-:|
|user_id `&str`| The id of the user whose connections you wish to terminate.|

|Return Value|Description|
|:-:|:-:|
|result `Result<(), String>` | If the request was successful, an `Ok` value will be returned. An `Err` value will be returned if any errors were encountered. |

###### Example

```rust
let user_id = "10";
pusher.terminate_user_connections(user_id).await;
```

### Authentication

Application security is very important so Pusher provides a mechanism for authenticating a user’s access to a channel at the point of subscription.

This can be used both to restrict access to private channels, and in the case of presence channels notify subscribers of who else is also subscribed via presence events.

This library provides a mechanism for generating an authentication signature to send back to the client and authorize them.

For more information see our [docs](http://pusher.com/docs/authenticating_users).

#### Private channels

##### `fn authenticate_private_channel(&self, channel_name: &str, socket_id: &str)`

|Argument|Description|
|:-:|:-:|
|channel_name `&str`| The channel name in the request sent by the client|
|socket_id `&str`| The socket id in the request sent by the client|

|Return Value|Description|
|:-:|:-:|
|Result `<String, &str>` | The `Ok` value will be the response to send back to the client, carrying an authentication signature. An `Err` value will be a string describing any errors generated |

###### Example using hyper

```rust
async fn pusher_auth(req: Request<Body>) -> Result<Response<Body>, Error> {
  let body = to_bytes(req).await.unwrap();
  let params = parse(body.as_ref()).into_owned().collect::<HashMap<String, String>>();
  let channel_name = params.get("channel_name").unwrap();
  let socket_id = params.get("socket_id").unwrap();
  let auth_signature = pusher.authenticate_private_channel(channel_name, socket_id).unwrap();
  Ok(Response::new(auth_signature.into()))
}
```

#### Authenticating presence channels

Using presence channels is similar to private channels, but in order to identify a user, clients are sent a user_id and, optionally, custom data.

##### `fn authenticate_presence_channel(&self, channel_name: &str, socket_id: &str, member: &Member)`

|Argument|Description|
|:-:|:-:|
|channel_name `&str`| The channel name in the request sent by the client|
|socket_id `&str`| The socket id in the request sent by the client|
|member `&pusher::Member`| A struct representing what to assign to a channel member, consisting of a `user_id` and any custom `user_info`. See below |

###### Custom Types

**pusher::Member**

```rust
pub struct Member<'a> {
  pub user_id: &'a str,
  pub user_info: Option<HashMap<&'a str, &'a str>>,
}
```

###### Example using hyper

```rust
async fn pusher_auth(req: Request<Body>) -> Result<Response<Body>, Error> {
  let body = to_bytes(req).await.unwrap();
  let params = parse(body.as_ref()).into_owned().collect::<HashMap<String, String>>();
  let channel_name = params.get("channel_name").unwrap();
  let socket_id = params.get("socket_id").unwrap();

  let mut member_data = HashMap::new();
  member_data.insert("twitter", "jamiepatel");
  let member = pusher::Member{user_id: "4", user_info: Some(member_data)};

  let auth_signature = pusher.authenticate_presence_channel(channel_name, socket_id, &member).unwrap();
  Ok(Response::new(auth_signature.into()))
}
```

#### Authenticating users

We can authenticate a user once per connection session. Authenticating a user gives your application access to user based features such as sending events to a user based on user id on terminating a user’s connections immediately.

##### `fn authenticate_user(&self, socket_id: &str, user: &User)`

|Argument|Description|
|:-:|:-:|
|socket_id `&str`| The socket id in the request sent by the client|
|user `&pusher::User`| A struct representing what to assign to a user, consisting of a `id` and any custom `user_info` and `watchlist`. See below |

###### Custom Types

**pusher::User**

```rust
pub struct User<'a> {
  pub id: &'a str,
  pub user_info: Option<HashMap<&'a str, &'a str>>,
  pub watchlist: Option<Vec<&'a str>>,
}
```

###### Example using hyper

```rust
async fn pusher_user_auth(req: Request<Body>) -> Result<Response<Body>, Error> {
  let body = to_bytes(req).await.unwrap();
  let params = parse(body.as_ref()).into_owned().collect::<HashMap<String, String>>();
  let socket_id = params.get("socket_id").unwrap();
  let mut user_info = HashMap::new();
  user_info.insert("username", "nikhilpatel");
  let watchlist = vec!["some-user-id", "some-other-user-id"];
  let user = pusher::User {id: "10", user_info: Some(user_info), watchlist: Some(watchlist)};
  let auth_signature = pusher.authenticate_user(socket_id, &user).unwrap();
  Ok(Response::new(auth_signature.into()))
}
```

### Application state

This library allows you to query our API to retrieve information about your application's channels, their individual properties, and, for presence-channels, the users currently subscribed to them.

#### Get the list of channels in an application

##### `async fn channels(&self)`

Requesting a list of application channels without any query options.

|Return Value|Description|
|:-:|:-:|
|result `Result<ChannelList, String>`| The `Ok` value will be a struct representing the list of channels. See below. An `Err` value will represent any errors encountered.|

##### `async fn channels_with_options(&self, params: QueryParameters)`

Adding options to your `channels` request.

|Argument|Description|
|:-:|:-:|
|params `QueryParameters`| A vector of tuples with query options. Where the first value of a tuple is `"filter_by_prefix"`, the API will filter the returned channels with the second value. To get number of users subscribed to a presence-channel, specify an `"info"` value in a tuple with a corresponding `"user_count"` value. |

|Return Value|Description|
|:-:|:-:|
|result `Result<ChannelList, String>`| As above.|

###### Custom Types

**pusher::ChannelsList**

```rust
pub struct ChannelList {
  pub channels: HashMap<String, Channel>,
}
```

**pusher::Channel**

```rust
pub struct Channel {
  pub occupied: Option<bool>,
  pub user_count: Option<i32>,
  pub subscription_count: Option<i32>,
}

```
###### Example

**Without options**:

```rust
pusher.channels().await;
//=> Ok(ChannelList { channels: {"presence-chatroom": Channel { occupied: None, user_count: None, subscription_count: None }, "presence-notifications": Channel { occupied: None, user_count: None, subscription_count: None }} })
```

**With options**:

```rust
let channels_params = vec![("filter_by_prefix", "presence-"), ("info", "user_count")];
pusher.channels_with_options(channels_params).await;
//=> Ok(ChannelList { channels: {"presence-chatroom": Channel { occupied: None, user_count: Some(92), subscription_count: None }, "presence-notifications": Channel { occupied: None, user_count: Some(29), subscription_count: None }} })
```

#### Get the state of a single channel

##### `async fn channel(&self, channel_name: &str)`

Requesting the state of a single channel without any query options.

|Return Value|Description|
|:-:|:-:|
|result `Result<Channel, String>`| The `Ok` value will be a struct representing a channel. See above. An `Err` value will represent any errors encountered.|

##### `async fn channel_with_options(&self, channel_name: &str, params: QueryParameters)`

Adding options to your `channel` request.

|Argument|Description|
|:-:|:-:|
|channel_name `&str`| The name of the channel|
|params `QueryParameters`| A vector of tuples with query options. To request information regarding user_count and subscription_count, a tuple must have an "info" value and a value containing a comma-separated list of attributes. An `Err` will be returned for any invalid API requests. |

|Return Value|Description|
|:-:|:-:|
|result `Result<Channel, String>`| As above.|

###### Example

**Without options**:

```rust
pusher.channel("presence-chatroom").await;
//=> Ok(Channel { occupied: Some(true), user_count: None, subscription_count: None })
```

**With options**:

```rust
let channel_params = vec![("info", "user_count,subscription_count")];
pusher.channel_with_options("presence-chatroom", channel_params).await;
//=> Ok(Channel { occupied: Some(true), user_count: Some(96), subscription_count: Some(96) })
```

#### Get a list of users in a presence channel

##### `async fn channel_users(&self, channel_name : &str)`

|Argument|Description|
|:-:|:-:|
|channel_name `&str`| The channel name|

|Return Value|Description|
|:-:|:-:|
|result `Result<ChannelUserList, String>`| The `Ok` value will be a struct representing a list of the users subscribed to the presence-channel. See below. The `Err` value will represent any errors encountered. |

###### Custom Types

**pusher::ChannelUserList**

```rust
pub struct ChannelUserList {
  pub users: Vec<ChannelUser>,
}
```

**pusher::ChannelUser**

```rust
pub struct ChannelUser {
  pub id: String,
}
```

###### Example

```rust
pusher.channel_users("presence-chatroom").await;
//=> Ok(ChannelUserList { users: [ChannelUser { id: "red" }, ChannelUser { id: "blue" }] })
```

### Webhook validation

On your [dashboard](http://app.pusher.com), you can set up webhooks to POST a payload to your server after certain events. Such events include channels being occupied or vacated, members being added or removed in presence-channels, or after client-originated events. For more information see <https://pusher.com/docs/webhooks>.

This library provides a mechanism for checking that these POST requests are indeed from Pusher, by checking the token and authentication signature in the header of the request.

##### `fn webhook(&self, key: &str, signature: &str, body: &str)`

|Argument|Description|
|:-:|:-:|
|key `&str` | The key supplied in the "X-Pusher-Key" header |
|signature `&str` | The signature supplied in the "X-Pusher-Signature" header |
|body `&str` | The body of the request |

|Return Value|Description|
|:-:|:-:|
|result `Result<Webhook, &str>`| If the webhook is valid, the `Ok` value will be a representation of that webhook that includes its timestamp and associated events. If the webhook is invalid, an `Err` value will be passed.|

##### Custom Types

**pusher::Webhook**

```rust
pub struct Webhook {
  pub time_ms: i64,
  pub events: Vec<HashMap<String, String>>,
}
```

##### Example

```rust
pusher.webhook("supplied_key", "supplied_signature", "body")
```

## Feature Support

Feature                                    | Supported
-------------------------------------------| :-------:
Trigger event on single channel            | *&#10004;*
Trigger event on multiple channels         | *&#10004;*
Trigger event to specific users            | *&#10004;*
Excluding recipients from events           | *&#10004;*
Authenticating private channels            | *&#10004;*
Authenticating presence channels           | *&#10004;*
Authenticating users                       | *&#10004;*
Terminating user connections               | *&#10004;*
Get the list of channels in an application | *&#10004;*
Get the state of a single channel          | *&#10004;*
Get a list of users in a presence channel  | *&#10004;*
WebHook validation                         | *&#10004;*
Heroku add-on support                      | *&#10004;*
Debugging & Logging                        | *&#10004;*
Cluster configuration                      | *&#10004;*
HTTPS                                      | *&#10004;*
Timeouts                                   | *&#10008;*
HTTP Proxy configuration                   | *&#10008;*
HTTP KeepAlive                             | *&#10008;*

### Helper Functionality

These are helpers that have been implemented to to ensure interactions with the HTTP API only occur if they will not be rejected e.g. [channel naming conventions](https://pusher.com/docs/client_api_guide/client_channels#naming-channels).

Helper Functionality                     | Supported
-----------------------------------------| :-------:
Channel name validation                  | &#10004;
Limit to 10 channels per trigger         | &#10004;
Limit event name length to 200 chars     | &#10004;

## Developing the Library

Feel more than free to fork this repo, improve it in any way you'd prefer, and send us a pull request :)

### Running the tests

Simply type:

```bash
$ cargo test
```

## License

This code is free to use under the terms of the MIT license.

## To Do

* Review the use of different string types.
* More test coverage
