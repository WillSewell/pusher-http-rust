extern crate pusher; 
extern crate hyper;

#[macro_use] extern crate yup_hyper_mock;
#[macro_use] extern crate log;

use pusher::Pusher;
use std::io::Read;

mod client_test;

mock_connector!(BadRequest {
    "http://127.0.0.1" =>       "HTTP/1.1 400 Bad Request\r\n\
                                 Server: mock1\r\n\
                                 \r\n\

                                 Cannot retrieve the user count unless the channel is a presence channel"
});


#[test]
fn test_error_response_handler() {
    let mut client = hyper::Client::with_connector(BadRequest);
    let mut pusher = Pusher::new("1", "2", "3").client(client).host("127.0.0.1").finalize();
    let query_params = vec![("info", "user_count,subscription_count")];
    let res = pusher.channel("this_is_not_a_presence_channel", Some(query_params));
    assert_eq!(res.unwrap_err(), "Error: 400 Bad Request. Cannot retrieve the user count unless the channel is a presence channel")
}