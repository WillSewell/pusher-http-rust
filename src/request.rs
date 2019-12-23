use bytes::buf::BufExt;
use hyper::{Body, Client, StatusCode, Uri};
use hyper::body;
use hyper::client::connect::Connect;
use hyper::header::CONTENT_TYPE;
use std::io::Read;
use std::str::FromStr;

pub async fn send_request<C, T>(
  client: &mut Client<C>,
  method: &str,
  request_url: url::Url,
  data: Option<String>
) -> Result<T, String>
  where
    C: Connect + Clone + Send + Sync + 'static,
    T: serde::de::DeserializeOwned
{
  let request_uri: Uri = FromStr::from_str(request_url.as_str()).unwrap();
  let request_builder =
    hyper::Request::builder()
      .method(method)
      .uri(request_uri)
      .header(CONTENT_TYPE, "application/json");
  let request = match data {
    Some(body) => request_builder.body(Body::from(body)),
    None => request_builder.body(Body::empty()),
  }.unwrap();

  let response = client.request(request).await.unwrap();
  let status = response.status();
  let mut body_reader = body::aggregate(response).await.unwrap().reader();

  match status {
    StatusCode::OK => {
      let body = serde_json::from_reader(body_reader).unwrap();
      Ok(body)
    },
    _ =>  {
      let mut body = String::new();
      body_reader.read_to_string(&mut body).unwrap();
      Err(format!("Error: {}. {}", status, body))
    }
  }
}
