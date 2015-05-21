use hyper::Url;
use hyper::Client;
use hyper::header::ContentType;
use hyper::method::Method;
use hyper::status::StatusCode;
use rustc_serialize::{self, json};

use std::io::Read;


pub fn create_request<T : rustc_serialize::Decodable>(client: &mut Client, method: &str, request_url: Url, data: Option<&str>) -> Result<T, String> {
  let response = send_request(client, method, request_url, data);

  if let Ok(encoded) = response {
    let decoded : T = json::decode(&encoded[..]).unwrap();
    return Ok(decoded)
  }
  return Err(response.unwrap_err())
}

pub fn send_request(client: &mut Client, method: &str, request_url: Url, data: Option<&str>) -> Result<String, String> {

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

   match res.status {
    StatusCode::Ok => return Ok(body),
    _ =>  {
      return Err(format!("Error: {}. {}", res.status, body))
    }
    }

}
