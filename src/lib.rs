extern crate hyper;
extern crate crypto;
extern crate rustc_serialize as rustc_serialize;
extern crate time;
extern crate queryst;

mod client;
mod signature;
mod request;
mod request_url;

pub type QueryParameters<'a> = Option<Vec<(&'a str, &'a str)>>;