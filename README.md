# Rust WebRTC Client-Server example

## About

An example with WebRTC-client and WebRTC-server.
Network model is Client-Server-Client.

WebRTC-client works in two modes: sender or receiver.
In sender mode, it sends text and video data to the server.
The WebRTC server forwards data from one sender client to a receiver client.

The client side of this example uses async rust and web-sys including the use of WebRTC.
The server side of this example uses async rust, WebSocket using tokio-tungstenite and WebRTC using webrtc-rs.

In order to simplify the code, all the errors are unwrapped.

## State

- [x] Signaling protocol,
- [x] Signaling server,
- [x] Multiple clients per server,
- [x] Sender-Client-To-Server WebRTC-connection,
- [x] Sender-Client-To-Server text,
- [x] Sender-Client-To-Server video,
- [x] Sender-Client-To-Server audio,
- [ ] Sender-Client-To-Server video and audio as a single stream,
- [x] Server-To-Receiver-Client WebRTC-connection,
- [x] Server-To-Receiver-Client text,
- [x] Server-To-Receiver-Client video,
- [x] Server-To-Receiver-Client audio,
- [ ] Server-To-Receiver-Client video and audio as a single stream,
- [x] Each Sender-Client paired with the Receiver-Client,
- [x] Data transfer from Sender-Client to Receiver-Client via server,
- [x] Media transfer from Sender-Client to Receiver-Client via server.

## Setup

* Run `bash setup.sh`

## Usage

* Run `bash watch.sh`
* Open `localhost:8080` in browser
* Edit the server address if necessary and click button `Start sender` or `Start receiver`.
* Type in sender TextArea, the message will be displayed on the receiver TextArea.
* If the receiver is started before the sender, you will see the video as soon as the sender is started.
* If the sender starts before the receiver, the video will start after the sender sends keyframes.
* A separate `HtmlVideoElement` is used for audio playback on the Client-Receiver side.

## License

Licensed under either of

* Apache License, Version 2.0,
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any
additional terms or conditions.
