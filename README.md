# Pusher HTTP Rust Library

[![Build Status](https://travis-ci.org/pusher/pusher-http-rust.svg?branch=master)](https://travis-ci.org/pusher/pusher-http-rust)
[![Coverage Status](https://coveralls.io/repos/pusher/pusher-http-rust/badge.svg?branch=master)](https://coveralls.io/r/pusher/pusher-http-rust?branch=master) 
[![Crates Badge](http://meritbadge.herokuapp.com/pusher)](https://crates.io/crates/pusher)

The Rust library for interacting with the Pusher HTTP API.

This package lets you trigger events to your client and query the state of your Pusher channels. When used with a server, you can validate Pusher webhooks and authenticate private- or presence-channels.

In order to use this library, you need to have a free account on <http://pusher.com>. After registering, you will need the application credentials for your app.

This README is meant to give an overview of the library, but more in-depth documentation can be found on [our GitHub page](http://pusher.github.io/pusher-http-rust/pusher).

###Table of Contents

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

## Getting Started

```rust
extern crate pusher; // imports the `pusher` module
 
use pusher::Pusher; // brings the Pusher struct into scope
 
fn main(){

  // initializes a Pusher object with your app credentials
  let mut pusher = Pusher::new("APP_ID", "KEY", "SECRET").finalize();
 
  // triggers an event called "my_event" on a channel called "test_channel", with the payload "hello world!"
  pusher.trigger("test_channel", "my_event", "hello world!");
 
}
```

## Configuration

There easiest way to configure the library is by creating a new `Pusher` instance:

```rust
let pusher = Pusher::new("id", "key", "secret").finalize();
```

`Pusher::new` returns a `PusherBuilder`, on which to chain configuration methods, before calling `finalize()`.

### Additional options

#### Instantiation From URL

```rust
Pusher::from_url("http://key:secret@api.host.com/apps/id").finalize();
```

#### Instantiation From Environment Variable

```rust
Pusher::from_env("PUSHER_URL").finalize();
```

This is particularly relevant if you are using Pusher as a Heroku add-on, which stores credentials in a `"PUSHER_URL"` environment variable.

#### HTTPS

To ensure requests occur over HTTPS, call `secure()` before `finalize()`.

```rust
let pusher = Pusher::new("id", "key", "secret").secure().finalize();
```

#### Changing Host

Calling `host()` before `finalize()` will make sure requests are sent to your specified host.

```rust
let pusher = Pusher::new("id", "key", "secret").host("foo.bar.com").finalize();
```

By default, this is `"api.pusherapp.com"`.

## Usage

### Triggering events

It is possible to trigger an event on one or more channels. Channel names can contain only characters which are alphanumeric, `_` or `-`` and have to be at most 200 characters long. Event name can be at most 200 characters long too.


#### Single channel

#####`fn trigger<Payload: Encodable>(&mut self, channel: &str, event: &str, payload: Payload)`

|Argument   |Description   |
|:-:|:-:|
|channel `&str`   |The name of the channel you wish to trigger on.   |
|event `&str` | The name of the event you wish to trigger |
|data `Payload : Encodable` | The payload you wish to send. Must be marshallable into JSON. |

|Return Value|Description|
|:-:|:-:|
|result `Result<TriggeredEvents, String>` | If the trigger was successful and you are connected to certain clusters, an object containing the `event_ids` field will be returned as part of a `Result`. An `Err` value will be returned if any errors were encountered.  |

###### Example

```rust
let mut hash_map = HashMap::new();
hash_map.insert("message", "hello world");

pusher.trigger("test_channel", "my_event", &hash_map);
```

#### Multiple channels

#####`fn trigger_multi<Payload: Encodable>(&mut self, channels: &Vec<&str>, event: &str, payload: Payload)`

|Argument | Description |
|:-:|:-:|
|channels `&Vec<&str>`| A vector of channel names you wish to send an event on. The maximum length is 10.|
|event `&str` | As above.|
|data `Payload : Encodable` |As above.|

|Return Value|Description|
|:-:|:-:|
|result `Result<TriggeredEvents, String>` | As above. |

######Example

```rust
let channels = vec!["test_channel", "test_channel2"];

pusher.trigger_multi(&channels, "my_event", "hello");
```

### Excluding event recipients

`trigger_exclusive` and `trigger_multi_exclusive` follow the patterns above, except a `socket_id` is given as the last parameter.

These methods allow you to exclude a recipient whose connection has that `socket_id` from receiving the event. You can read more [here](http://pusher.com/docs/duplicates).

######Examples

**On one channel**:

```rust
pusher.trigger_exclusive("test_channel", "my_event", "hello", "123.12");
```

**On multiple channels**:

```rust
let channels = vec!["test_channel", "test_channel2"];
pusher.trigger_multi_exclusive(&channels, "my_event", "hello", "123.12");
```

### Authenticating Channels

Application security is very important so Pusher provides a mechanism for authenticating a user’s access to a channel at the point of subscription.

This can be used both to restrict access to private channels, and in the case of presence channels notify subscribers of who else is also subscribed via presence events.

This library provides a mechanism for generating an authentication signature to send back to the client and authorize them.

For more information see our [docs](http://pusher.com/docs/authenticating_users).

#### Private channels


##### `fn authenticate_private_channel(&self, body: &String)`

|Argument|Description|
|:-:|:-:|
|params `&String`| The request body sent by the client|

|Return Value|Description|
|:-:|:-:|
|Result `<String, &str>` | The `Ok` value will be the response to send back to the client, carrying an authentication signature. An `Err` value will be a string describing any errors generated |

######Example Using Nickel.rs

```rust
fn pusher_auth<'a>(req: &mut Request, res: Response<'a>) -> MiddlewareResult<'a> {
 
  let mut body = String::new();
  req.origin.read_to_string(&mut body).unwrap(); // get the body from the request
  let auth = pusher.authenticate_private_channel(&body).unwrap(); // unwrap the result of the method
  res.send(auth)

}
```

#### Authenticating presence channels

Using presence channels is similar to private channels, but in order to identify a user, clients are sent a user_id and, optionally, custom data.

##### `fn authenticate_presence_channel(&self, body: &String, member: &Member)`

|Argument|Description|
|:-:|:-:|
|params `&String`| The request body sent by the client|
|member `Option<pusher::Member>`| An optional struct representing what to assign to a channel member, consisting of a `user_id` and any custom `user_info`. See below |

###### Custom Types

**pusher::Member**

```rust
pub struct Member<'a> {
    pub user_id: &'a str,
    pub user_info: Option<HashMap<&'a str, &'a str>>,
}
```

###### Example

```rust
fn pusher_auth<'a>(req: &mut Request, res: Response<'a>) -> MiddlewareResult<'a> {

  let mut body = String::new();
  req.origin.read_to_string(&mut body).unwrap();

  let mut member_data = HashMap::new();
  member_data.insert("twitter", "jamiepatel");

  let member = pusher::Member{user_id: "4", user_info: Some(member_data)};

  let auth = pusher.authenticate_presence_channel(&body, &member).unwrap();
  res.send(auth)

}
```

### Application state

This library allows you to query our API to retrieve information about your application's channels, their individual properties, and, for presence-channels, the users currently subscribed to them.

#### Get the list of channels in an application

##### `fn channels(&mut self)`

Requesting a list of application channels without any query options.

|Return Value|Description|
|:-:|:-:|
|result `Result<ChannelList, String>`| The `Ok` value will be a struct representing the list of channels. See below. An `Err` value will represent any errors encountered.|

##### `fn channels_with_options(&mut self, params: QueryParameters)`

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
######Example

**Without options**:

```rust
pusher.channels();
//=> Ok(ChannelList { channels: {"presence-chatroom": Channel { occupied: None, user_count: None, subscription_count: None }, "presence-notifications": Channel { occupied: None, user_count: None, subscription_count: None }} })
```

**With options**:

```rust
let channels_params = vec![("filter_by_prefix", "presence-"), ("info", "user_count")];
pusher.channels_with_options(channels_params);
//=> Ok(ChannelList { channels: {"presence-chatroom": Channel { occupied: None, user_count: Some(92), subscription_count: None }, "presence-notifications": Channel { occupied: None, user_count: Some(29), subscription_count: None }} })
```

#### Get the state of a single channel

##### `fn channel(&mut self, channel_name: &str)`

Requesting the state of a single channel without any query options.

|Return Value|Description|
|:-:|:-:|
|result `Result<Channel, String>`| The `Ok` value will be a struct representing a channel. See above. An `Err` value will represent any errors encountered.|

##### `fn channel_with_options(&mut self, channel_name: &str, params: QueryParameters)`

Adding options to your `channel` request.

|Argument|Description|
|:-:|:-:|
|channel `&str`| The name of the channel|
|params `QueryParameters`| A vector of tuples with query options. To request information regarding user_count and subscription_count, a tuple must have an "info" value and a value containing a comma-separated list of attributes. An `Err` will be returned for any invalid API requests. |

|Return Value|Description|
|:-:|:-:|
|result `Result<Channel, String>`| As above.|

###### Example

**Without options**:

```rust
pusher.channel("presence-chatroom");
//=> Ok(Channel { occupied: Some(true), user_count: None, subscription_count: None })
```
**With options**: 

```rust
let channel_params = vec![("info", "user_count,subscription_count")];
pusher.channel_with_options("presence-chatroom", channel_params);
//=> Ok(Channel { occupied: Some(true), user_count: Some(96), subscription_count: Some(96) })
```
#### Get a list of users in a presence channel

##### `fn channel_users(&mut self, channel_name: &str)`

|Argument|Description|
|:-:|:-:|
|name `&str`| The channel name|

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
pusher.channel_users("presence-chatroom");
//=> Ok(ChannelUserList { users: [ChannelUser { id: "red" }, ChannelUser { id: "blue" }] })
```

### Webhook validation

On your [dashboard](http://app.pusher.com), you can set up webhooks to POST a payload to your server after certain events. Such events include channels being occupied or vacated, members being added or removed in presence-channels, or after client-originated events. For more information see <https://pusher.com/docs/webhooks>.

This library provides a mechanism for checking that these POST requests are indeed from Pusher, by checking the token and authentication signature in the header of the request. 

##### `fn webhook(&self, key: &String, signature: &String, body: &str)`

|Argument|Description|
|:-:|:-:|
|key `&String` | The key supplied in the "X-Pusher-Key" header |
|signature `&String` | The signature supplied in the "X-Pusher-Signature" header | 
|body `&str` | The body of the request |

|Return Value|Description|
|:-:|:-:|
|result `Result<Webhook, &str>`| If the webhook is valid, the `Ok` value will be a representation of that webhook that includes its timestamp and associated events. If the webhook is invalid, an `Err` value will be passed.|

###### Custom Types

**pusher::Webhook**

```rust
pub struct Webhook {
    pub time_ms: i64,
    pub events: Vec<HashMap<String, String>>,
}
```

###### Example

```rust
pusher.webhook("supplied_key", "supplied_signature", "body")
```

## Feature Support

Feature                                    | Supported
-------------------------------------------| :-------:
Trigger event on single channel            | *&#10004;*
Trigger event on multiple channels         | *&#10004;*
Excluding recipients from events           | *&#10004;*
Authenticating private channels            | *&#10004;*
Authenticating presence channels           | *&#10004;*
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


#### Helper Functionality

These are helpers that have been implemented to to ensure interactions with the HTTP API only occur if they will not be rejected e.g. [channel naming conventions](https://pusher.com/docs/client_api_guide/client_channels#naming-channels).

Helper Functionality                     | Supported
-----------------------------------------| :-------:
Channel name validation            | &#10004;
Limit to 10 channels per trigger         | &#10004;
Limit event name length to 200 chars     | &#10004;

## Developing the Library

Feel more than free to fork this repo, improve it in any way you'd prefer, and send us a pull request :)

### Running the tests

Simply type:

    $ cargo test

## License

This code is free to use under the terms of the MIT license.

## To Do

* Review the use of different string types.
* More test coverage

