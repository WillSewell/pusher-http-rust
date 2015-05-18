extern crate hyper;
extern crate url;
extern crate crypto;
extern crate rustc_serialize;
extern crate time;

use std::io::Read;
use hyper::Client;
use hyper::header::ContentType;
use hyper::method::Method;
use rustc_serialize::json;
use hyper::Url;


use crypto::md5::Md5;
use crypto::digest::Digest;
use crypto::hmac::Hmac;
use crypto::sha2::Sha256;

use rustc_serialize::hex::ToHex;

use crypto::mac::Mac;


#[derive(RustcDecodable, RustcEncodable)]
struct TriggerEventData {
    name: String,
    channels: Vec<String>,
    data: String,
}

const AUTH_VERSION : &'static str = "1.0";

pub struct Pusher<'a> {
  app_id: &'a str,
  key: &'a str,
  secret: &'a str,
}

pub type QueryParameters<'a> = Option<Vec<(&'a str, &'a str)>>;

impl <'a>Pusher<'a> {

  pub fn new(app_id: &'a str, key: &'a str, secret: &'a str) -> Pusher<'a> {
    Pusher {
      app_id: app_id,
      key: key,
      secret: secret,
    }
  }

  pub fn trigger<Payload : rustc_serialize::Encodable>(&self, channel: &str, event: &str, payload: Payload) { 
    let request_url_string = format!("http://api.pusherapp.com/apps/{}/events", self.app_id);
    let mut request_url = Url::parse(&request_url_string).unwrap();

    let channels = vec![channel.to_string()];

    let json_payload = json::encode(&payload).unwrap();


    let raw_body = TriggerEventData{
      name: event.to_string(),
      channels: channels,
      data: json_payload,
    };

    let body = json::encode(&raw_body).unwrap();

    let method = "POST";
    update_request_url(method, &mut request_url, self.key, self.secret, Some(&body), None);
    send_request(method, request_url, Some(&body));
  }

  pub fn channels(&self, params: QueryParameters){
    let request_url_string = format!("http://api.pusherapp.com/apps/{}/channels", self.app_id);
    let mut request_url = Url::parse(&request_url_string).unwrap();
    let method = "GET";
    update_request_url(method, &mut request_url, self.key, self.secret, None, params);
    send_request(method, request_url, None);
  }

  // pub fn channel(&self, channe_name: &str, params: )

}

fn send_request(method: &str, request_url: Url, data: Option<&str>) {
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

    println!("Response: {}", body);

}

fn create_body_md5(body: &str) -> String {
  let mut sh = Md5::new();
  sh.input_str(body);
  sh.result_str()
}

fn create_auth_signature<'a>(to_sign: &str, secret: &'a str) -> String {
  let mut hmac = Hmac::new(Sha256::new(), secret.as_bytes());
  hmac.input(to_sign.as_bytes());
  let result = hmac.result();
  let code = result.code();
  code.to_hex()
}

fn update_request_url(method: &str, request_url: &mut Url, key: &str, secret: &str, data: Option<&str>, query_parameters: QueryParameters) {

  let mut auth_signature : String;
  let body_md5 : String;


  let auth_timestamp = time::get_time().sec.to_string();
  let path = request_url.serialize_path().unwrap();

  let mut query_pairs: Vec<(&str, &str)> = vec![
      ("auth_key", key),
      ("auth_timestamp", &auth_timestamp),
      ("auth_version", AUTH_VERSION)
  ];

  if let Some(body) = data {
    body_md5 = create_body_md5(body);
    query_pairs.push(("body_md5", &body_md5));
  }

  if let Some(params) = query_parameters {
    for param in params {
      query_pairs.push(param);
    }
  }


  request_url.set_query_from_pairs(query_pairs.iter().map(|&(k,v)| (k,v)));

  let query_string = match request_url.query {
    Some(ref qs) => qs.to_string(),
    None => panic!("No query string!"),
  };

  let to_sign = format!("{}\n{}\n{}", method, path, query_string);

  auth_signature = create_auth_signature(&to_sign, &secret);

  query_pairs.push(("auth_signature", &auth_signature));

  request_url.set_query_from_pairs(query_pairs.iter().map(|&(k,v)| (k,v)));

}
