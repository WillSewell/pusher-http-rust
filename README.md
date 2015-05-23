
[![Build Status](https://travis-ci.org/pusher/pusher-http-rust.svg?branch=master)](https://travis-ci.org/pusher/pusher-http-rust)
[![Coverage Status](https://coveralls.io/repos/pusher/pusher-http-rust/badge.svg?branch=master)](https://coveralls.io/r/pusher/pusher-http-rust?branch=master) 
[![Crates Badge](http://meritbadge.herokuapp.com/pusher)](https://crates.io/crates/pusher)

## Feature Support

Feature                                    | Supported
-------------------------------------------| :-------:
Trigger event on single channel            | *&#10004;*
Trigger event on multiple channels         | *&#10004;*
Excluding recipients from events           | *&#10004;*
Authenticating private channels            | *&#10004;*
Authenticating presence channels           | *&#10004;*
Get the list of channels in an application | *&#10004;*
Get the state of a single channel          | *&#10004;*
Get a list of users in a presence channel  | *&#10004;*
WebHook validation                         | *&#10004;*
Heroku add-on support                      | *&#10004;*
Debugging & Logging                        | *&#10004;*
Cluster configuration                      | *&#10004;*
HTTPS                                      | *&#10004;*
Timeouts                                   | *&#10008;*
HTTP Proxy configuration                   | *&#10008;*
HTTP KeepAlive                             | *&#10008;*


#### Helper Functionality

These are helpers that have been implemented to to ensure interactions with the HTTP API only occur if they will not be rejected e.g. [channel naming conventions](https://pusher.com/docs/client_api_guide/client_channels#naming-channels).

Helper Functionality                     | Supported
-----------------------------------------| :-------:
Channel name validation            | &#10004;
Limit to 10 channels per trigger         | &#10004;
Limit event name length to 200 chars     | &#10004;

## TODO

* Better error handling (e.g. changing unwraps to pattern matches)
* Clean up code (e.g. use of string types) and request repetition
* Tests

