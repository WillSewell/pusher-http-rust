// #[macro_use] extern crate nickel;

// extern crate rustc_serialize;

// #[macro_use] extern crate hyper;

// extern crate pusher;

// use pusher::Pusher;

// use nickel::{Nickel, Request, Response, HttpRouter, MiddlewareResult};
// use std::collections::HashMap;
// use std::io::Read;
// use std::path::Path;

// header! {
//     (Foo, "X-Foo") => [String]
// }


// fn main() {


//     let mut server = Nickel::new();

//     fn handler<'a>(_: &mut Request, res: Response<'a>) -> MiddlewareResult<'a> {
//         let index = Path::new("src/index.html");
//         res.send_file(index)
//     }

//     fn pusher_auth<'a>(req: &mut Request, res: Response<'a>) -> MiddlewareResult<'a> {
      
//       let app_id = env!("RUST_ID");
//       let key = env!("RUST_KEY");
//       let secret = env!("RUST_SECRET");

//       let pusher = Pusher::new(app_id, key, secret)
//                           .finalize();


//       let mut body = String::new();
//       req.origin.read_to_string(&mut body).unwrap();

//       let mut member_data = HashMap::new();
//       member_data.insert("twitter", "jamiepatel");

//       let member = pusher::Member{user_id: "4", user_info: member_data};

//       let auth = pusher.authenticate_presence_channel(&body, &member);
//       println!("{:?}", auth);
//       res.send(auth)
//     }

//     fn pusher_webhook<'a>(req: &mut Request, res: Response<'a>) -> MiddlewareResult<'a> {

//       let app_id = env!("RUST_ID");
//       let key = env!("RUST_KEY");
//       let secret = env!("RUST_SECRET");

//       let pusher = Pusher::new(app_id, key, secret)
//                           .finalize();

//       // let header_key = req.origin.headers.get::<Foo>();

//       let mut body = String::new();
//       req.origin.read_to_string(&mut body).unwrap();

//       let header_key = req.origin.headers.get_raw("X-Pusher-Key").unwrap();
//       let parsed_key : Vec<String> = hyper::header::parsing::from_comma_delimited(header_key).unwrap();

//       let header_signature = req.origin.headers.get_raw("X-Pusher-Signature").unwrap();
//       let parsed_signature : Vec<String> = hyper::header::parsing::from_comma_delimited(header_signature).unwrap();

//       println!("{:?}", parsed_key);

//       println!("{:?}", parsed_signature);



//       let webhook = pusher.webhook(&parsed_key[0], &parsed_signature[0], &body);

//       println!("{:?}", webhook);

//       // let header_key = &header_raw;

//       // let header_key = header_raw[0];
//       // let ref header_vec = header_key;


//       // let header = str::from_utf8(header_key).unwrap();

//       // println!("{:?}", header_raw);



//       // let signature = req.origin.headers.get_raw("X-Pusher-Signature");

//       // println!("{:?}", header_key);
//       // println!("{:?}", signature);

//       res.send("hello")

//     }



//     server.get("/", handler);
//     server.post("/pusher/auth", pusher_auth);
//     server.post("/pusher/webhook", pusher_webhook);

//     server.listen("127.0.0.1:6767");
// }







extern crate pusher;

use pusher::Pusher;
use std::collections::HashMap;

fn main() {

  let app_id = env!("RUST_ID");
  let key = env!("RUST_KEY");
  let secret = env!("RUST_SECRET");


  let mut pusher = Pusher::new(app_id, key, secret)
                      .finalize();

  // println!("{:?}", pusher);  

  let mut hash_map = HashMap::new();
  hash_map.insert("message", "hello world");


  pusher.trigger("test_channel", "my_event", &hash_map);

  let trigger_chans = vec!["test_channel", "test_channel2"];
  pusher.trigger_multi(trigger_chans, "my_event", &hash_map);


  pusher.channels(None);

  let channels_params = vec![("filter_by_prefix", "presence-"), ("info", "user_count")];

  let channels = pusher.channels(Some(channels_params));

  println!("{:?}", channels);

  let channel_params = vec![("info", "user_count,subscription_count")];

  let channel = pusher.channel("presence-session-d41a439c438a100756f5-4bf35003e819bb138249-hu9e5NecuNr", Some(channel_params));
  println!("{:?}", channel);

  let users = pusher.channel_users("presence-session-d41a439c438a100756f5-4bf35003e819bb138249-hu9e5NecuNr");
  println!("{:?}", users);


}