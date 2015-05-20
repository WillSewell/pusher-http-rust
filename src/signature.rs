use crypto::md5::Md5;
use crypto::digest::Digest;
use crypto::hmac::Hmac;
use crypto::sha2::Sha256;
use rustc_serialize::hex::{ToHex, FromHex};
use crypto::mac::{Mac, MacResult};
use std::collections::HashMap;

pub fn create_body_md5(body: &str) -> String {
  let mut sh = Md5::new();
  sh.input_str(body);
  sh.result_str()
}

pub fn create_channel_auth<'a>(auth_map: &mut HashMap<&'a str,String>, key: &str, secret: &str, to_sign: &str){
  let auth_signature = create_auth_signature(to_sign, secret);
  let auth_string = format!("{}:{}", key, auth_signature);
  auth_map.insert("auth", auth_string);
}

pub fn check_signature(signature: &str, secret: &str, body: &str) -> bool {
  let mut expected_hmac = Hmac::new(Sha256::new(), secret.as_bytes());
  expected_hmac.input(body.as_bytes());
  let decoded_signature = signature.from_hex().unwrap();
  let result = MacResult::new(&decoded_signature);
  result.eq(&expected_hmac.result())
}

pub fn create_auth_signature<'a>(to_sign: &str, secret: &'a str) -> String {
  let mut hmac = Hmac::new(Sha256::new(), secret.as_bytes());
  hmac.input(to_sign.as_bytes());
  let result = hmac.result();
  let code = result.code();
  code.to_hex()
}