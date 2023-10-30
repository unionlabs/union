<p align="center">
  <img width="460" height="300" src="https://raw.githubusercontent.com/rustls/rustls/main/admin/rustls-logo-web.png">
</p>

<p align="center">
Rustls is a modern TLS library written in Rust.  It uses <a href = "https://github.com/briansmith/ring"><em>ring</em></a> for cryptography and <a href = "https://github.com/rustls/webpki">webpki</a> for certificate
verification.
</p>

# Status
Rustls is mature and widely used. While most of the API surface is stable, we expect the next
few releases will make further changes as needed to accomodate new features or performance improvements.

If you'd like to help out, please see [CONTRIBUTING.md](CONTRIBUTING.md).

[![Build Status](https://github.com/rustls/rustls/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/rustls/rustls/actions/workflows/build.yml?query=branch%3Amain)
[![Coverage Status (codecov.io)](https://codecov.io/gh/rustls/rustls/branch/main/graph/badge.svg)](https://codecov.io/gh/rustls/rustls/)
[![Documentation](https://docs.rs/rustls/badge.svg)](https://docs.rs/rustls/)
[![Chat](https://img.shields.io/discord/976380008299917365?logo=discord)](https://discord.gg/MCSB76RU96)

## Release history

* Release 0.21.3 (2023-07-05)
  - Added `with_crls` function to `AllowAnyAuthenticatedClient` and
    `AllowAnyAnonymousOrAuthenticatedClient` client certificate verifiers to
    support revocation checking of client certificates using certificate
    revocation lists (CRLs).
  - Exposed `verify_signed_by_trust_anchor` and `verify_server_name` certificate
    validation helper functions when using the "dangerous_configuration"
    feature.
* Release 0.21.2 (2023-06-14)
  - Bump MSRV to 1.60 to track similar change in dependencies.
  - Differentiate between unexpected and expected EOF in `Stream` and `OwnedStream`.
  - `RootCertStore::add_parsable_certificates` now takes a `&[impl AsRef<[u8]>]`.
  - Add QUIC V2 support.
* Release 0.21.1 (2023-05-01)
  - Remove `warn`-level logging from code paths that also return a `rustls::Error` with
    the same information.
  - Bug fix: ensure `ConnectionCommon::complete_io` flushes pending writes.
  - Bug fix: correct encoding of acceptable issuer subjects when rustls operates as a server
    requesting client authentication.  This was a regression introduced in 0.21.0.
* Release 0.21.0 (2023-03-29)
  - Support for connecting to peers named with IP addresses.  This means
    rustls now depends on a fork of webpki - `rustls-webpki` - with a suitably
    extended API.
  - *Breaking change*: `StoresClientSessions` trait renamed to `ClientSessionStore` and
    reworked to allow storage of multiple TLS1.3 tickets and avoid reuse of them.
    This is a privacy improvement, see RFC8446 appendix C.4.
  - *Breaking change*: the `DistinguishedNames` type alias no longer exists; the public
    API now exports a `DistinguishedName` type, and the
    `ClientCertVerifier::client_auth_root_subjects()` method now returns a
    `&[DistinguishedName]` instead (with the lifetime constrained to the
    verifier's).
  - *Breaking change*: the `ClientCertVerifier` methods `client_auth_mandatory()`
    and `client_auth_root_subjects()` no longer return an `Option`. You can now
    use an `Acceptor` to decide whether to accept the connection based on information
    from the `ClientHello` (like server name).
  - *Breaking change*: rework `rustls::Error` to avoid String usage in
    `PeerMisbehavedError`, `PeerIncompatibleError` and certificate errors.
    Especially note that custom certificate verifiers should move to use the
    new certificate errors. `Error` is now `non_exhaustive`, and so are the
    inner enums used in its variants.
  - *Breaking change*: replace `webpki::Error` appearing in the public API
    in `RootCertStore::add`.
  - The number of tickets sent by a TLS1.3 server is now configurable via
    `ServerConfig::send_tls13_tickets`.  Previously one ticket was sent, now
    the default is four.
  - *Breaking change*: remove deprecated methods from `Acceptor`.
  - *Breaking change*: `AllowAnyAuthenticatedClient` and `AllowAnyAnonymousOrAuthenticatedClient`
    `new` functions now return `Self`. A `boxed` function was added to both types to easily acquire
    an `Arc<dyn ClientCertVerifier>`.
  - *Breaking change*: `NoClientAuth::new` was renamed to `boxed`.
  - *Breaking change*: the QUIC API has changed to provide QUIC-specific `ClientConnection` and
    `ServerConnection` types, instead of using an extension trait.
  - *Breaking change*: the QUIC `Secrets` constructor was changed to take
    a `Side` instead of `bool`.
  - *Breaking change*: the `export_keying_material` function on a `Connection`
    was changed from returning `Result<(), Error>` to `Result<T, Error>` where
    `T: AsMut<[u8]>`.
  - *Breaking change*: the `sni_hostname` function on a `Connection` was renamed
    to `server_name`.
  - *Breaking change*: remove alternative type names deprecated in 0.20.0 (`RSASigningKey` vs.
    `RsaSigningKey` etc.)
  - *Breaking change*: the client config `session_storage` and `enable_tickets`
    fields have been replaced by a more misuse resistant `Resumption` type that
    combines the two options.

See [RELEASE_NOTES.md](RELEASE_NOTES.md) for further change history.

# Documentation
Lives here: https://docs.rs/rustls/

# Approach
Rustls is a TLS library that aims to provide a good level of cryptographic security,
requires no configuration to achieve that security, and provides no unsafe features or
obsolete cryptography.

## Current features

* TLS1.2 and TLS1.3.
* ECDSA, Ed25519 or RSA server authentication by clients.
* ECDSA, Ed25519 or RSA server authentication by servers.
* Forward secrecy using ECDHE; with curve25519, nistp256 or nistp384 curves.
* AES128-GCM and AES256-GCM bulk encryption, with safe nonces.
* ChaCha20-Poly1305 bulk encryption ([RFC7905](https://tools.ietf.org/html/rfc7905)).
* ALPN support.
* SNI support.
* Tunable fragment size to make TLS messages match size of underlying transport.
* Optional use of vectored IO to minimise system calls.
* TLS1.2 session resumption.
* TLS1.2 resumption via tickets ([RFC5077](https://tools.ietf.org/html/rfc5077)).
* TLS1.3 resumption via tickets or session storage.
* TLS1.3 0-RTT data for clients.
* TLS1.3 0-RTT data for servers.
* Client authentication by clients.
* Client authentication by servers.
* Extended master secret support ([RFC7627](https://tools.ietf.org/html/rfc7627)).
* Exporters ([RFC5705](https://tools.ietf.org/html/rfc5705)).
* OCSP stapling by servers.
* SCT stapling by servers.
* SCT verification by clients.

## Possible future features

* PSK support.
* OCSP verification by clients.
* Certificate pinning.

## Non-features

For reasons [explained in the manual](https://docs.rs/rustls/latest/rustls/manual/_02_tls_vulnerabilities/index.html),
rustls does not and will not support:

* SSL1, SSL2, SSL3, TLS1 or TLS1.1.
* RC4.
* DES or triple DES.
* EXPORT ciphersuites.
* MAC-then-encrypt ciphersuites.
* Ciphersuites without forward secrecy.
* Renegotiation.
* Kerberos.
* Compression.
* Discrete-log Diffie-Hellman.
* Automatic protocol version downgrade.

There are plenty of other libraries that provide these features should you
need them.

### Platform support

While Rustls itself is platform independent it uses
[`ring`](https://crates.io/crates/ring) for implementing the cryptography in
TLS. As a result, rustls only runs on platforms
supported by `ring`. At the time of writing this means x86, x86-64, armv7, and
aarch64. For more information see [the supported `ring` CI
targets](https://github.com/briansmith/ring/blob/9cc0d45f4d8521f467bb3a621e74b1535e118188/.github/workflows/ci.yml#L151-L167).

Rustls requires Rust 1.60 or later.

# Example code
There are two example programs which use
[mio](https://github.com/carllerche/mio) to do asynchronous IO.

## Client example program
The client example program is named `tlsclient-mio`.  The interface looks like:

```tlsclient-mio
Connects to the TLS server at hostname:PORT.  The default PORT
is 443.  By default, this reads a request from stdin (to EOF)
before making the connection.  --http replaces this with a
basic HTTP GET request for /.

If --cafile is not supplied, a built-in set of CA certificates
are used from the webpki-roots crate.

Usage:
  tlsclient-mio [options] [--suite SUITE ...] [--proto PROTO ...] [--protover PROTOVER ...] <hostname>
  tlsclient-mio (--version | -v)
  tlsclient-mio (--help | -h)

Options:
    -p, --port PORT     Connect to PORT [default: 443].
    --http              Send a basic HTTP GET request for /.
    --cafile CAFILE     Read root certificates from CAFILE.
    --auth-key KEY      Read client authentication key from KEY.
    --auth-certs CERTS  Read client authentication certificates from CERTS.
                        CERTS must match up with KEY.
    --protover VERSION  Disable default TLS version list, and use
                        VERSION instead.  May be used multiple times.
    --suite SUITE       Disable default cipher suite list, and use
                        SUITE instead.  May be used multiple times.
    --proto PROTOCOL    Send ALPN extension containing PROTOCOL.
                        May be used multiple times to offer several protocols.
    --no-tickets        Disable session ticket support.
    --no-sni            Disable server name indication support.
    --insecure          Disable certificate verification.
    --verbose           Emit log output.
    --max-frag-size M   Limit outgoing messages to M bytes.
    --version, -v       Show tool version.
    --help, -h          Show this screen.
```

Some sample runs:

```
$ cargo run --bin tlsclient-mio -- --http mozilla-modern.badssl.com
HTTP/1.1 200 OK
Server: nginx/1.6.2 (Ubuntu)
Date: Wed, 01 Jun 2016 18:44:00 GMT
Content-Type: text/html
Content-Length: 644
(...)
```

or

```
$ cargo run --bin tlsclient-mio -- --http expired.badssl.com
TLS error: InvalidCertificate(Expired)
Connection closed
```

## Server example program
The server example program is named `tlsserver-mio`.  The interface looks like:

```tlsserver-mio
Runs a TLS server on :PORT.  The default PORT is 443.

`echo' mode means the server echoes received data on each connection.

`http' mode means the server blindly sends a HTTP response on each
connection.

`forward' means the server forwards plaintext to a connection made to
localhost:fport.

`--certs' names the full certificate chain, `--key' provides the
RSA private key.

Usage:
  tlsserver-mio --certs CERTFILE --key KEYFILE [--suite SUITE ...] [--proto PROTO ...] [--protover PROTOVER ...] [options] echo
  tlsserver-mio --certs CERTFILE --key KEYFILE [--suite SUITE ...] [--proto PROTO ...] [--protover PROTOVER ...] [options] http
  tlsserver-mio --certs CERTFILE --key KEYFILE [--suite SUITE ...] [--proto PROTO ...] [--protover PROTOVER ...] [options] forward <fport>
  tlsserver-mio (--version | -v)
  tlsserver-mio (--help | -h)

Options:
    -p, --port PORT     Listen on PORT [default: 443].
    --certs CERTFILE    Read server certificates from CERTFILE.
                        This should contain PEM-format certificates
                        in the right order (the first certificate should
                        certify KEYFILE, the last should be a root CA).
    --key KEYFILE       Read private key from KEYFILE.  This should be a RSA
                        private key or PKCS8-encoded private key, in PEM format.
    --ocsp OCSPFILE     Read DER-encoded OCSP response from OCSPFILE and staple
                        to certificate.  Optional.
    --auth CERTFILE     Enable client authentication, and accept certificates
                        signed by those roots provided in CERTFILE.
    --require-auth      Send a fatal alert if the client does not complete client
                        authentication.
    --resumption        Support session resumption.
    --tickets           Support tickets.
    --protover VERSION  Disable default TLS version list, and use
                        VERSION instead.  May be used multiple times.
    --suite SUITE       Disable default cipher suite list, and use
                        SUITE instead.  May be used multiple times.
    --proto PROTOCOL    Negotiate PROTOCOL using ALPN.
                        May be used multiple times.
    --verbose           Emit log output.
    --version, -v       Show tool version.
    --help, -h          Show this screen.
```

Here's a sample run; we start a TLS echo server, then connect to it with
`openssl` and `tlsclient-mio`:

```
$ cargo run --bin tlsserver-mio -- --certs test-ca/rsa/end.fullchain --key test-ca/rsa/end.rsa -p 8443 echo &
$ echo hello world | openssl s_client -ign_eof -quiet -connect localhost:8443
depth=2 CN = ponytown RSA CA
verify error:num=19:self signed certificate in certificate chain
hello world
^C
$ echo hello world | cargo run --bin tlsclient-mio -- --cafile test-ca/rsa/ca.cert -p 8443 localhost
hello world
^C
```

# License

Rustls is distributed under the following three licenses:

- Apache License version 2.0.
- MIT license.
- ISC license.

These are included as LICENSE-APACHE, LICENSE-MIT and LICENSE-ISC
respectively.  You may use this software under the terms of any
of these licenses, at your option.

# Code of conduct

This project adopts the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).
Please email rustls-mod@googlegroups.com to report any instance of misconduct, or if you
have any comments or questions on the Code of Conduct.
