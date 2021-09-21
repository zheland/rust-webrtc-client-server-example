# rust-wasm-client-and-server-webrtc-mvp (WIP)

## About

Work in progress mvp includes a WebRTC-client written in rust using the web-sys library and a signal and WebRTC-server.

## State

WebRTC communication itself does not work yet.

- [x] Signaling protocol,
- [x] Signaling server,
- [x] Multiple clients per server,
- [x] Sender-Client-To-Server WebRTC-connection,
- [x] Sender-Client-To-Server text,
- [ ] Sender-Client-To-Server video,
- [ ] Server-To-Receiver-Client WebRTC-connection,
- [ ] Server-To-Receiver-Client text,
- [ ] Server-To-Receiver-Client video,
- [ ] Each Sender-Client paired with the Receiver-Client,
- [ ] Data transfer from Sender-Client to Receiver-Client via server.
- [ ] Video transfer from Sender-Client to Receiver-Client via server.

## Setup

* Run `bash setup.sh`

## Usage

* Run `bash watch.sh`
* Open `localhost:8080` in browser
* Edit the server address if necessary and click button `[Start sender]`.
* Type in TextArea, the message will be displayed in the server console.

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
