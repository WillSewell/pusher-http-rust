#[macro_use] extern crate nickel;
extern crate rustc_serialize;
extern crate plugin;
extern crate hyper;

extern crate pusher;

use pusher::Pusher;

use nickel::{Nickel, Request, Response, HttpRouter, MiddlewareResult, JsonBody, QueryString};
use std::collections::HashMap;
use plugin::Pluggable;
use std::io::Read;

use std::path::Path;

fn main() {
    let mut server = Nickel::new();

    fn handler<'a>(_: &mut Request, res: Response<'a>) -> MiddlewareResult<'a> {
        // let mut data = HashMap::<&str, &str>::new();
        // data.insert("name", "user");
        // res.render("src/index.html", &data)
        let index = Path::new("src/index.html");
        res.send_file(index)
    }

    fn pusher_auth<'a>(req: &mut Request, res: Response<'a>) -> MiddlewareResult<'a> {
      
      let app_id = env!("RUST_ID");
      let key = env!("RUST_KEY");
      let secret = env!("RUST_SECRET");

      let pusher = Pusher::new(app_id, key, secret)
                          .finalize();


      let mut body = String::new();
      req.origin.read_to_string(&mut body).unwrap();

      let mut member_data = HashMap::new();
      member_data.insert("twitter", "jamiepatel");

      let member = pusher::Member{user_id: "4", user_info: member_data};

      let auth = pusher.authenticate_presence_channel(&body, &member);
      println!("{:?}", auth);
      res.send(auth)
    }

    server.get("/", handler);
    server.post("/pusher/auth", pusher_auth);

    server.listen("127.0.0.1:6767");
}

// extern crate pusher;

// use pusher::Pusher;
// use std::collections::HashMap;

// fn main() {

//   let app_id = env!("RUST_ID");
//   let key = env!("RUST_KEY");
//   let secret = env!("RUST_SECRET");

//   let pusher = Pusher::new(app_id, key, secret)
//                       .finalize();

//   println!("{:?}", pusher);  

//   let mut hash_map = HashMap::new();
//   hash_map.insert("message", "hello world");


//   pusher.trigger("test_channel", "my_event", &hash_map);

//   // let trigger_chans = vec!["test_channel", "test_channel2"];
//   // pusher.trigger_multi(trigger_chans, "my_event", &hash_map);
//   // // pusher.trigger_exclusive("test_channel", "my_event", &hash_map, "45553.5500569");


//   // pusher.channels(None);

//   // let channels_params = vec![("filter_by_prefix", "presence-"), ("info", "user_count")];

//   // let channels = pusher.channels(Some(channels_params));

//   // println!("{:?}", channels);

//   // let channel_params = vec![("info", "user_count,subscription_count")];

//   // let channel = pusher.channel("presence-session-d41a439c438a100756f5-4bf35003e819bb138249-hu9e5NecuNr", Some(channel_params));
//   // println!("{:?}", channel);

//   // let users = pusher.channel_users("presence-session-d41a439c438a100756f5-4bf35003e819bb138249-hu9e5NecuNr");
//   // println!("{:?}", users);


// }