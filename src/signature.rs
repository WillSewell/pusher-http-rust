use hmac::{Hmac, Mac};
use md5::{Md5, Digest};
use sha2::Sha256;
use hex::{FromHex, ToHex};
use std::collections::HashMap;

type HmacSha256 = Hmac<Sha256>;

pub fn create_body_md5(body: &str) -> String {
    let mut sh = Md5::new();
    sh.update(body.as_bytes());
    let result = sh.finalize();
    result.encode_hex()
}

pub fn create_channel_auth<'a>(
    auth_map: &mut HashMap<&'a str, String>,
    key: &str,
    secret: &str,
    to_sign: &str,
) {
    let auth_signature = create_auth_signature(to_sign, secret);
    let auth_string = format!("{}:{}", key, auth_signature);
    auth_map.insert("auth", auth_string);
}

pub fn check_signature(signature: &str, secret: &str, body: &str) -> bool {
    let mut hmac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    hmac.update(body.as_bytes());
    let decoded_signature = Vec::from_hex(signature).unwrap();
    hmac.verify_slice(&decoded_signature).is_ok()
}

pub fn create_auth_signature<'a>(to_sign: &str, secret: &'a str) -> String {
    let mut hmac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    hmac.update(to_sign.as_bytes());
    let result = hmac.finalize();
    let code = result.into_bytes();
    code.encode_hex()
}
