extern crate hyper;
extern crate log;
extern crate pusher;
extern crate tokio;
#[cfg(test)]
#[macro_use]
extern crate yup_hyper_mock;

use hyper::Client;
use pusher::PusherBuilder;

mock_connector!(BadRequest {
    "http://127.0.0.1" =>       "HTTP/1.1 400 Bad Request\r\n\
                                 Server: mock1\r\n\
                                 \r\n\
                                 Cannot retrieve the user count unless the channel is a presence channel"
});

mock_connector!(TriggerEBTest {
    "http://127.0.0.1" =>       "HTTP/1.1 200 OK\r\n\
                                 Server: mock1\r\n\
                                 \r\n\
                                 {\"event_ids\":{\"test_channel\":\"eudhq1809scss2\"}}"
});

mock_connector!(ChannelsRequest {
    "http://127.0.0.1" =>       "HTTP/1.1 200 OK\r\n\
                                 Server: mock1\r\n\
                                 \r\n\
                                 {\"channels\":{\"presence-session-d41a439c438a100756f5-4bf35003e819bb138249-5cbTiUiPNGI\":{\"user_count\":1},\"presence-session-d41a439c438a100756f5-4bf35003e819bb138249-PbZ5E1pP8uF\":{\"user_count\":1},\"presence-session-d41a439c438a100756f5-4bf35003e819bb138249-oz6iqpSxMwG\":{\"user_count\":1}}}"
});

mock_connector!(ChannelRequest {
    "http://127.0.0.1" =>       "HTTP/1.1 200 OK\r\n\
                                 Server: mock1\r\n\
                                 \r\n\
                                 {\"user_count\":1,\"occupied\":true,\"subscription_count\":1}"
});

mock_connector!(ChannelUsersRequest {
    "http://127.0.0.1" =>       "HTTP/1.1 200 OK\r\n\
                                 Server: mock1\r\n\
                                 \r\n\
                                 {\"users\":[{\"id\":\"red\"},{\"id\":\"blue\"}]}"
});

#[tokio::test]
async fn test_error_response_handler() {
    let client = Client::builder().build(BadRequest::default());
    let pusher = PusherBuilder::new_with_client(client, "1", "2", "3")
        .host("127.0.0.1")
        .finalize();
    let query_params = vec![(
        "info".to_string(),
        "user_count,subscription_count".to_string(),
    )];
    let res = pusher
        .channel_with_options("this_is_not_a_presence_channel", query_params)
        .await;
    assert_eq!(res.unwrap_err(), "Error: 400 Bad Request. Cannot retrieve the user count unless the channel is a presence channel")
}

#[tokio::test]
async fn test_eb_trigger() {
    let client = Client::builder().build(TriggerEBTest::default());
    let pusher = PusherBuilder::new_with_client(client, "1", "2", "3")
        .host("127.0.0.1")
        .finalize();
    let res = pusher.trigger("woot", "yolo", "huh").await;
    let events = res.unwrap();
    println!("{:?}", events);
    let event_id = events.event_ids.unwrap();

    assert_eq!(event_id.get("test_channel").unwrap(), "eudhq1809scss2")
}

#[tokio::test]
async fn test_get_channels() {
    let client = Client::builder().build(ChannelsRequest::default());
    let pusher = PusherBuilder::new_with_client(client, "1", "2", "3")
        .host("127.0.0.1")
        .finalize();
    let res = pusher.channels().await;
    let channels = res.unwrap();
    let item = channels
        .channels
        .get("presence-session-d41a439c438a100756f5-4bf35003e819bb138249-5cbTiUiPNGI")
        .unwrap();
    assert_eq!(item.user_count.unwrap(), 1)
}

#[tokio::test]
async fn test_get_channel() {
    let client = Client::builder().build(ChannelRequest::default());
    let pusher = PusherBuilder::new_with_client(client, "1", "2", "3")
        .host("127.0.0.1")
        .finalize();
    let res = pusher.channel("presence-for-all").await;
    let channel = res.unwrap();
    let user_count = channel.user_count.unwrap();
    let occupied = channel.occupied.unwrap();
    let subscription_count = channel.subscription_count.unwrap();
    assert_eq!(user_count, 1);
    assert_eq!(occupied, true);
    assert_eq!(subscription_count, 1)
}

#[tokio::test]
async fn test_get_channel_users() {
    let client = Client::builder().build(ChannelUsersRequest::default());
    let pusher = PusherBuilder::new_with_client(client, "1", "2", "3")
        .host("127.0.0.1")
        .finalize();
    let res = pusher.channel_users("presence-yolo").await;
    let users = res.unwrap().users;
    let user_one = &users[0];
    let user_two = &users[1];
    assert_eq!(user_one.id, "red");
    assert_eq!(user_two.id, "blue")
}
