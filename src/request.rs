use hyper::Url;
use hyper::Client;
use hyper::header::ContentType;
use hyper::method::Method;
use rustc_serialize::{self, json};

use std::io::Read;

pub fn create_request<T : rustc_serialize::Decodable>(method: &str, request_url: Url, data: Option<&str>) -> T {
  let encoded = send_request(method, request_url, data);
  let decoded : T = json::decode(&encoded[..]).unwrap();
  decoded
}

pub fn send_request(method: &str, request_url: Url, data: Option<&str>) -> String {
    let mut client = Client::new();

    let request_method = match method {
      "POST" => Method::Post,
      _ => Method::Get,
    };

    let mut builder = client.request(request_method, request_url)
                            .header(ContentType::json());

    if let Some(body) = data {
      builder = builder.body(body);
    }

    let mut res = builder.send().unwrap();

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();
    println!("{:?}", body);

    body

}
