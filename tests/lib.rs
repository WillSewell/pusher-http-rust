extern crate pusher; 
extern crate hyper;

#[macro_use] extern crate yup_hyper_mock;
#[macro_use] extern crate log;

use pusher::Pusher;
use std::io::Read;

mod client_test;

mock_connector!(TestRequest {
    "http://127.0.0.1" =>       "HTTP/1.1 200 Redirect\r\n\
                                 Server: mock1\r\n\
                                 \r\n\

                                 hello"
});


// #[test]
// fn test_trigger_success() {
//     let mut client = hyper::Client::with_connector(TestRequest);
//     let mut pusher = Pusher::new("1", "2", "3").client(client).host("127.0.0.1").finalize();
//     let res = pusher.trigger("hello", "hi", "waddup");
//     assert_eq!(res, "uolo")
// }