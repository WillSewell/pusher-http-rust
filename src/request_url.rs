use hyper::Url;
use time;

use super::QueryParameters;
use super::signature::*;

const AUTH_VERSION : &'static str = "1.0";

pub fn update_request_url(method: &str, request_url: &mut Url, key: &str, secret: &str, data: Option<&str>, query_parameters: QueryParameters) {

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

  let query_string = match request_url.lossy_percent_decode_query() {
    Some(ref qs) => qs.to_string(),
    None => panic!("No query string!"),
  };

  let to_sign = format!("{}\n{}\n{}", method, path, query_string);

  auth_signature = create_auth_signature(&to_sign, &secret);

  query_pairs.push(("auth_signature", &auth_signature));

  request_url.set_query_from_pairs(query_pairs.iter().map(|&(k,v)| (k,v)));

}