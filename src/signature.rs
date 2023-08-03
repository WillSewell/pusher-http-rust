use hex::{FromHex, ToHex};
use hmac::{Hmac, Mac};
use md5::{Digest, Md5};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn create_body_md5(body: &str) -> String {
    let mut sh = Md5::new();
    sh.update(body.as_bytes());
    let result = sh.finalize();
    result.encode_hex()
}

pub fn compute_auth_key(key: &str, secret: &str, data: &str) -> String {
    let auth_signature = sign(data, secret);
    format!("{}:{}", key, auth_signature)
}

pub fn verify(signature: &str, secret: &str, body: &str) -> bool {
    let mut hmac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    hmac.update(body.as_bytes());
    let decoded_signature = Vec::from_hex(signature).unwrap();
    hmac.verify_slice(&decoded_signature).is_ok()
}

pub fn sign(data: &str, secret: &str) -> String {
    let mut hmac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    hmac.update(data.as_bytes());
    let result = hmac.finalize();
    let code = result.into_bytes();
    code.encode_hex()
}
