extern crate hyper;
extern crate pusher;
extern crate serde;
extern crate serde_json;
extern crate tokio;
extern crate url;

use hyper::body::to_bytes;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Request, Response, Server};
use pusher::PusherBuilder;
use std::collections::HashMap;
use std::net::SocketAddr;
use url::form_urlencoded::parse;

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let make_svc = make_service_fn(|_conn| async { Ok::<_, Error>(service_fn(authenticate)) });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn authenticate(req: Request<Body>) -> Result<Response<Body>, Error> {
    let pusher = PusherBuilder::from_url("http://key:secret@api.host.com/apps/id").finalize();
    let body = to_bytes(req).await.unwrap();
    let params = parse(body.as_ref())
        .into_owned()
        .collect::<HashMap<String, String>>();
    let channel_name = params.get("channel_name").unwrap();
    let socket_id = params.get("socket_id").unwrap();

    let mut member_data = HashMap::new();
    member_data.insert("twitter", "jamiepatel");
    let member = pusher::Member {
        user_id: "4",
        user_info: Some(member_data),
    };
    let auth_signature = pusher
        .authenticate_presence_channel(channel_name, socket_id, &member)
        .unwrap();

    Ok(Response::new(auth_signature.into()))
}
