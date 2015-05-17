extern crate pusher;

use pusher::Pusher;

fn main() {

  let app_id = env!("RUST_ID");
  let key = env!("RUST_KEY");
  let secret = env!("RUST_SECRET");

  let pusher = Pusher::new(app_id, key, secret);

  pusher.trigger("test_channel", "my_event", "{\"message\":\"hello world\"}")


}