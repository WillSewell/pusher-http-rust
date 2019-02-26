use hyper::header::ContentType;
use hyper::method::Method;
use hyper::status::StatusCode;
use hyper::Client;
use hyper::Url;
use rustc_serialize::{self, json};

use std::io::Read;

pub fn create_request<T: rustc_serialize::Decodable>(
    client: &Client,
    method: &str,
    request_url: Url,
    data: Option<&str>,
) -> Result<T, String> {
    let response = send_request(client, method, request_url, data);

    if let Ok(encoded) = response {
        let decoded: T = json::decode(&encoded[..]).unwrap();
        return Ok(decoded);
    }
    Err(response.unwrap_err())
}

pub fn send_request(
    client: &Client,
    method: &str,
    request_url: Url,
    data: Option<&str>,
) -> Result<String, String> {
    let request_method = match method {
        "POST" => Method::Post,
        _ => Method::Get,
    };

    let mut builder = client
        .request(request_method, request_url.clone())
        .header(ContentType::json());

    if let Some(body) = data {
        builder = builder.body(body);
    }

    match builder.send() {
        Ok(mut res) => {
            let mut body = String::new();
            res.read_to_string(&mut body).unwrap();

            match res.status {
                StatusCode::Ok => Ok(body),
                _ => Err(format!("Error: {}. {}", res.status, body)),
            }
        }
        Err(e) => {
            if let hyper::error::Error::Io(_e) = e {
                send_request(client, method, request_url, data)
            } else {
                Err(e.to_string())
            }
        }
    }
}
