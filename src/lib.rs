//! # Pusher-HTTP-Rust
//!
//! The Rust library for interacting with the Pusher HTTP API.
//!
//! This package lets you trigger events to your client and query the state of your
//! Pusher channels. When used with a server, you can validate Pusher webhooks and
//! authenticate private- or presence-channels.
//!
//! In order to use this library, you need to have a free account on
//! http://pusher.com. After registering, you will need the application credentials
//! for your app.
//!
//! ## Getting Started
//!
//! Firstly, add `pusher` to your `Cargo.toml`.
//!
//! To trigger an event:
//!
//! ```
//! extern crate pusher; // imports the `pusher` module
//!
//! use pusher::PusherBuilder; // brings the PusherBuilder struct into scope
//!
//! // the functions are async, so we need a reactor running (e.g. tokio)
//! // this example uses "current_thread" for simplicity
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() {
//!   // initializes a Pusher object with your app credentials
//!   let pusher = PusherBuilder::new("APP_ID", "KEY", "SECRET").finalize();
//!
//!   // triggers an event called "my_event" on a channel called "test_channel", with the payload "hello world!"
//!   let result = pusher.trigger("test_channel", "my_event", "hello world!").await;
//!   match result {
//!     Ok(events) => println!("Successfully published: {:?}", events),
//!     Err(err) => println!("Failed to publish: {}", err),
//!   }
//! }
//! ```

extern crate hyper;
extern crate regex;
extern crate serde;

mod client;
mod error;
mod json_structures;
mod request;
mod request_url;
mod signature;
mod util;

pub use self::client::{Pusher, PusherBuilder};
pub use self::error::Error;
pub use self::json_structures::{
    Channel, ChannelAuthResponse, ChannelList, ChannelUser, ChannelUserList, Member,
    QueryParameters, TriggeredEvents, User, UserAuthResponse, Webhook,
};
