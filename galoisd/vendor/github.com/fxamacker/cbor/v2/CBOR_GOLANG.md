👉  [Comparisons](https://github.com/fxamacker/cbor#comparisons) • [Status](https://github.com/fxamacker/cbor#current-status) • [Design Goals](https://github.com/fxamacker/cbor#design-goals) • [Features](https://github.com/fxamacker/cbor#features) • [Standards](https://github.com/fxamacker/cbor#standards) • [Fuzzing](https://github.com/fxamacker/cbor#fuzzing-and-code-coverage) • [Usage](https://github.com/fxamacker/cbor#usage) • [Security Policy](https://github.com/fxamacker/cbor#security-policy) • [License](https://github.com/fxamacker/cbor#license)

# CBOR
[CBOR](https://en.wikipedia.org/wiki/CBOR) is a data format designed to allow small code size and small message size. CBOR is defined in [RFC 8949 Concise Binary Object Representation](https://tools.ietf.org/html/rfc8949) (previously [RFC 7049](https://tools.ietf.org/html/rfc7049)), an [IETF](http://ietf.org/) Internet Standards Document.

CBOR is also designed to be stable for decades, be extensible without need for version negotiation, and not require a schema.

While JSON uses text, CBOR uses binary. CDDL can be used to express CBOR (and JSON) in an easy and unambiguous way.  CDDL is defined in (RFC 8610 Concise Data Definition Language).

## CBOR in Golang (Go)
[Golang](https://golang.org/) is a nickname for the Go programming language.  Go is specified in [The Go Programming Language Specification](https://golang.org/ref/spec).

__[fxamacker/cbor](https://github.com/fxamacker/cbor)__ is a library (written in Go) that encodes and decodes CBOR. The API design of fxamacker/cbor is based on Go's [`encoding/json`](https://golang.org/pkg/encoding/json/).  The design and reliability of fxamacker/cbor makes it ideal for encoding and decoding COSE.

## COSE
COSE is a protocol using CBOR for basic security services. COSE is defined in ([RFC 8152 CBOR Object Signing and Encryption](https://tools.ietf.org/html/rfc8152)).

COSE describes how to create and process signatures, message authentication codes, and encryption using CBOR for serialization.  COSE specification also describes how to represent cryptographic keys using CBOR.  COSE is used by WebAuthn.

## CWT
CBOR Web Token (CWT) is defined in [RFC 8392](http://tools.ietf.org/html/rfc8392).  CWT is based on COSE and was derived in part from JSON Web Token (JWT).  CWT is a compact way to securely represent claims to be transferred between two parties.

## WebAuthn
[WebAuthn](https://en.wikipedia.org/wiki/WebAuthn) (Web Authentication) is a web standard for authenticating users to web-based apps and services. It's a core component of FIDO2, the successor of FIDO U2F legacy protocol.

__[fxamacker/webauthn](https://github.com/fxamacker/webauthn)__ is a library (written in Go) that performs server-side authentication for clients using FIDO2 keys, legacy FIDO U2F keys, tpm, and etc.

Copyright (c) Faye Amacker and contributors.

<hr>

👉  [Comparisons](https://github.com/fxamacker/cbor#comparisons) • [Status](https://github.com/fxamacker/cbor#current-status) • [Design Goals](https://github.com/fxamacker/cbor#design-goals) • [Features](https://github.com/fxamacker/cbor#features) • [Standards](https://github.com/fxamacker/cbor#standards) • [Fuzzing](https://github.com/fxamacker/cbor#fuzzing-and-code-coverage) • [Usage](https://github.com/fxamacker/cbor#usage) • [Security Policy](https://github.com/fxamacker/cbor#security-policy) • [License](https://github.com/fxamacker/cbor#license)
