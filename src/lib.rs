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
//! use pusher::Pusher; // brings the Pusher struct into scope
//! 
//! fn main(){
//!   // initializes a Pusher object with your app credentials
//!   let mut pusher = Pusher::new("APP_ID", "KEY", "SECRET").finalize();
//! 
//!   // triggers an event called "my_event" on a channel called "test_channel", with the payload "hello world!"
//!   let res = pusher.trigger("test_channel", "my_event", "hello world!");
//! 
//! }
//! ```


extern crate hyper;
extern crate crypto;
extern crate rustc_serialize as rustc_serialize;
extern crate time;
extern crate queryst;

extern crate regex;

mod client;
mod signature;
mod request;
mod request_url;
mod json_structures;
mod util;

pub use self::client::{Pusher,PusherBuilder};
pub use self::json_structures::{Member, Webhook, Channel, ChannelList, ChannelUserList, TriggeredEvents, QueryParameters};