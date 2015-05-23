[![Build Status](https://travis-ci.org/Byron/yup-hyper-mock.svg?branch=master)](https://travis-ci.org/Byron/yup-hyper-mock)

`hyper-mock` is a utility library to help hyper clients with their testing. It provides types used to test hyper itself, most notably, mock connections and macros to ease their use.

## Usage

Set it up for use in tests in `Cargo.toml`
```toml
[dev-dependencies]
yup-hyper-mock = "*"
log = "*"  # log macros are used within yup-hyper-mock
```

Link it in case you are testing only in your `src/(lib.rs|main.rs)`
```Rust
#[cfg(test)] #[macro_use]
extern crate "yup-hyper-mock" as hyper_mock
```

In your tests module
```Rust
#[cfg(test)]
mod tests {
    use hyper;

    mock_connector!(MockRedirectPolicy {
        "http://127.0.0.1" =>       "HTTP/1.1 301 Redirect\r\n\
                                     Location: http://127.0.0.2\r\n\
                                     Server: mock1\r\n\
                                     \r\n\
                                    "
        "http://127.0.0.2" =>       "HTTP/1.1 302 Found\r\n\
                                     Location: https://127.0.0.3\r\n\
                                     Server: mock2\r\n\
                                     \r\n\
                                    "
        "https://127.0.0.3" =>      "HTTP/1.1 200 OK\r\n\
                                     Server: mock3\r\n\
                                     \r\n\
                                    "

    #[test]
    fn test_redirect_followall() {
        let mut client = hyper::Client::with_connector(MockRedirectPolicy);
        client.set_redirect_policy(hyper::RedirectPolicy::FollowAll);

        let res = client.get("http://127.0.0.1").send().unwrap();
        assert_eq!(res.headers.get(), Some(&hyper::Server("mock3".to_string())));
    }
}
```

## Credits

`yup-hyper-mock` is code from `hyper/src/mock.rs`, which was adjusted to work within its very own crate.