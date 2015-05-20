extern crate pusher; 
extern crate hyper;

#[macro_use] extern crate yup_hyper_mock;
#[macro_use] extern crate log;

mod client_test;

mock_connector!(TestRequest {
    "http://127.0.0.1" =>       "HTTP/1.1 200 Redirect\r\n\
                                 Server: mock1\r\n\
                                 \r\n\

                                 hello"
});

use std::io::Read;

#[test]
fn test_redirect_followall() {
    let mut client = hyper::Client::with_connector(TestRequest);

    let mut res = client.get("http://127.0.0.1").send().unwrap();

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();
    println!("{:?}", body);
    assert_eq!(body, "hello")
}