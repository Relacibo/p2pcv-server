# p2pcv-server

Small server that is supposed to establish the p2p connections of the clients.
Stack: diesel, actix-web, libp2p

## Notes
### Generate pubkey for libp2p:
```bash
openssl genpkey -algorithm ed25519 -out private.pem
```
### Generate cert:
```rust
let mut params = rcgen::CertificateParams::new(vec![
    rand::distributions::Alphanumeric.sample_string(&mut rand::thread_rng(), 16)
]);
params.alg = &rcgen::PKCS_ECDSA_P256_SHA256;
let a = webrtc::peer_connection::certificate::RTCCertificate::from_params(params)
    .expect("default params to work");
println!(a.serialize_pem().replace("\r\n", "$"));
```
```bash
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -sha256 -days 365
```
