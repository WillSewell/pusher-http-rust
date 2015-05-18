extern crate pusher;

use pusher::Pusher;
use std::collections::HashMap;

fn main() {

  let app_id = env!("ATOM_ID");
  let key = env!("ATOM_KEY");
  let secret = env!("ATOM_SECRET");

  let pusher = Pusher::new(app_id, key, secret);

  let mut hash_map = HashMap::new();
  hash_map.insert("message", "hello world");


  pusher.trigger("test_channel", "my_event", &hash_map);

  pusher.channels();

}