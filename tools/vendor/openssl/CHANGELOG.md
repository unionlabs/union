# Change Log

## [Unreleased]

## [v0.10.41] - 2022-06-09

### Fixed

* Fixed a use-after-free in `Error::function` and `Error::file` with OpenSSL 3.x.

### Added

* Added `MessageDigest::block_size` and `MdRef::block_size`.
* Implemented `Ord` and `Eq` for `X509` and `X509Ref`.
* Added `X509Extension::add_alias`.
* Added SM4 support.
* Added `EcGroup::from_components` `EcGropuRef::set_generator`, and `EcPointRef::set_affine_coordinates_gfp`.

## [v0.10.40] - 2022-05-04

### Fixed

* Fixed the openssl-sys dependency version.

## [v0.10.39] - 2022-05-02

### Deprecated

* Deprecated `SslContextBuilder::set_tmp_ecdh_callback` and `SslRef::set_tmp_ecdh_callback`.

### Added

* Added `SslRef::extms_support`.
* Added `Nid::create`.
* Added `CipherCtx`, which exposes a more direct interface to `EVP_CIPHER_CTX`.
* Added `PkeyCtx`, which exposes a more direct interface to `EVP_PKEY_CTX`.
* Added `MdCtx`, which exposes a more direct interface to `EVP_MD_CTX`.
* Added `Pkcs12Builder::mac_md`.
* Added `Provider`.
* Added `X509Ref::issuer_name_hash`.
* Added `Decrypter::set_rsa_oaep_label`.
* Added `X509Ref::to_text`.

## [v0.10.38] - 2021-10-31

### Added

* Added `Pkey::ec_gen`.

## [v0.10.37] - 2021-10-27

### Fixed

* Fixed linkage against OpenSSL distributions built with `no-chacha`.

### Added

* Added `BigNumRef::to_vec_padded`.
* Added `X509Name::from_der` and `X509NameRef::to_der`.
* Added `BigNum::new_secure`, `BigNumReef::set_const_time`, `BigNumref::is_const_time`, and `BigNumRef::is_secure`.

## [v0.10.36] - 2021-08-17

### Added

* Added `Asn1Object::as_slice`.
* Added `PKeyRef::{raw_public_key, raw_private_key, private_key_to_pkcs8_passphrase}` and
    `PKey::{private_key_from_raw_bytes, public_key_from_raw_bytes}`.
* Added `Cipher::{seed_cbc, seed_cfb128, seed_ecb, seed_ofb}`.

## [v0.10.35] - 2021-06-18

### Fixed

* Fixed a memory leak in `Deriver`.

### Added

* Added support for OpenSSL 3.x.x.
* Added `SslStream::peek`.

## [v0.10.34] - 2021-04-28

### Added

* Added `Dh::set_private_key` and `DhRef::private_key`.
* Added `EcPointRef::affine_coordinates`.
* Added `TryFrom` implementations to convert between `PKey` and specific key types.
* Added `X509StoreBuilderRef::set_flags`.

## [v0.10.33] - 2021-03-13

### Fixed

* `Dh::generate_params` now uses `DH_generate_params_ex` rather than the deprecated `DH_generated_params` function.

### Added

* Added `Asn1Type`.
* Added `CmsContentInfoRef::decrypt_without_cert_check`.
* Added `EcPointRef::{is_infinity, is_on_curve}`.
* Added `Encrypter::set_rsa_oaep_label`.
* Added `MessageDigest::sm3`.
* Added `Pkcs7Ref::signers`.
* Added `Cipher::nid`.
* Added `X509Ref::authority_info` and `AccessDescription::{method, location}`.
* Added `X509NameBuilder::{append_entry_by_text_with_type, append_entry_by_nid_with_type}`.

## [v0.10.32] - 2020-12-24

### Fixed

* Fixed `Ssl::new` to take a `&SslContextRef` rather than `&SslContext`.

### Added

* Added the `encrypt` module to support asymmetric encryption and decryption with `PKey`s.
* Added `MessageDigest::from_name`.
* Added `ConnectConfiguration::into_ssl`.
* Added the ability to create unconnected `SslStream`s directly from an `Ssl` and transport stream
    without performing any part of the handshake with `SslStream::new`.
* Added `SslStream::{read_early_data, write_early_data, connect, accept, do_handshake, stateless}`.
* Implemented `ToOwned` for `SslContextRef`.
* Added `SslRef::{set_connect_state, set_accept_state}`.

### Deprecated

* Deprecated `SslStream::from_raw_parts` in favor of `Ssl::from_ptr` and `SslStream::new`.
* Deprecated `SslStreamBuilder` in favor of methods on `Ssl` and `SslStream`.

## [v0.10.31] - 2020-12-09

### Added

* Added `Asn1Object::from_str`.
* Added `Dh::from_pgq`, `DhRef::prime_p`, `DhRef::prime_q`, `DhRef::generator`, `DhRef::generate_params`,
    `DhRef::generate_key`, `DhRef::public_key`, and `DhRef::compute_key`.
* Added `Pkcs7::from_der` and `Pkcs7Ref::to_der`.
* Added `Id::X25519`, `Id::X448`, `PKey::generate_x25519`, and `PKey::generate_x448`.
* Added `SrtpProfileId::SRTP_AEAD_AES_128_GCM` and `SrtpProfileId::SRTP_AEAD_AES_256_GCM`.
* Added `SslContextBuilder::verify_param` and `SslContextBuilder::verify_param_mut`.
* Added `X509Ref::subject_name_hash` and `X509Ref::version`.
* Added `X509StoreBuilderRef::add_lookup`, and the `X509Lookup` type.
* Added `X509VerifyFlags`, `X509VerifyParamRef::set_flags`, `X509VerifyParamRef::clear_flags`
    `X509VerifyParamRef::get_flags`.

## [v0.10.30] - 2020-06-25

### Fixed

* `DsaRef::private_key_to_pem` can no longer be called without a private key.

### Changed

* Improved the `Debug` implementations of many types.

### Added

* Added `is_empty` implementations for `Asn1StringRef` and `Asn1BitStringRef`.
* Added `EcPointRef::{to_pem, to_dir}` and `EcKeyRef::{public_key_from_pem, public_key_from_der}`.
* Added `Default` implementations for many types.
* Added `Debug` implementations for many types.
* Added `SslStream::from_raw_parts`.
* Added `SslRef::set_mtu`.
* Added `Cipher::{aes_128_ocb, aes_192_ocb, aes_256_ocb}`.

### Deprecated

* Deprecated `SslStreamBuilder::set_dtls_mtu_size` in favor of `SslRef::set_mtu`.

## [v0.10.29] - 2020-04-07

### Fixed

* Fixed a memory leak in `X509Builder::append_extension`.

### Added

* Added `SslConnector::into_context` and `SslConnector::context`.
* Added `SslAcceptor::into_context` and `SslAcceptor::context`.
* Added `SslMethod::tls_client` and `SslMethod::tls_server`.
* Added `SslContextBuilder::set_cert_store`.
* Added `SslContextRef::verify_mode` and `SslRef::verify_mode`.
* Added `SslRef::is_init_finished`.
* Added `X509Object`.
* Added `X509StoreRef::objects`.

## [v0.10.28] - 2020-02-04

### Fixed

* Fixed the mutability of `Signer::sign_oneshot` and `Verifier::verify_oneshot`. This is unfortunately a breaking
    change, but a necessary soundness fix.

## [v0.10.27] - 2020-01-29

### Added

* Added `MessageDigest::null`.
* Added `PKey::private_key_from_pkcs8`.
* Added `SslOptions::NO_RENEGOTIATION`.
* Added `SslStreamBuilder::set_dtls_mtu_size`.

## [v0.10.26] - 2019-11-22

### Fixed

* Fixed improper handling of the IV buffer in `envelope::{Seal, Unseal}`.

### Added

* Added `Asn1TimeRef::{diff, compare}`.
* Added `Asn1Time::from_unix`.
* Added `PartialEq` and `PartialOrd` implementations for `Asn1Time` and `Asn1TimeRef`.
* Added `base64::{encode_block, decode_block}`.
* Added `EcGroupRef::order_bits`.
* Added `Clone` implementations for `Sha1`, `Sha224`, `Sha256`, `Sha384`, and `Sha512`.
* Added `SslContextBuilder::{set_sigalgs_list, set_groups_list}`.

## [v0.10.25] - 2019-10-02

### Fixed

* Fixed a memory leak in `EcdsaSig::from_private_components` when using OpenSSL 1.0.x.

### Added

* Added support for Ed25519 and Ed448 keys.
* Implemented `ToOwned` for `PKeyRef` and `Clone` for `PKey`.

## [v0.10.24] - 2019-07-19

### Fixed

* Worked around an OpenSSL 1.0.x bug triggered by code calling `SSL_set_app_data`.

### Added

* Added `aes::{wrap_key, unwrap_key}`.
* Added `CmsContentInfoRef::to_pem` and `CmsContentInfo::from_pem`.
* Added `DsaRef::private_key_to_pem`.
* Added `EcGroupRef::{cofactor, generator}`.
* Added `EcPointRef::to_owned`.
* Added a `Debug` implementation for `EcKey`.
* Added `SslAcceptor::{mozilla_intermediate_v5, mozilla_modern_v5}`.
* Added `Cipher::{aes_128_ofb, aes_192_ecb, aes_192_cbc, aes_192_ctr, aes_192_cfb1, aes_192_cfb128, aes_192_cfb8,
    aes_192_gcm, aes_192_ccm, aes_192_ofb, aes_256_ofb}`.

## [v0.10.23] - 2019-05-18

### Fixed

* Fixed session callbacks when an `Ssl`'s context is replaced.

### Added

* Added `SslContextBuilder::add_client_ca`.

## [v0.10.22] - 2019-05-08

### Added

* Added support for the LibreSSL 2.9.x series.

## [v0.10.21] - 2019-04-30

### Fixed

* Fixed overly conservatifve buffer size checks in `Crypter` when using stream ciphers.

### Added

* Added bindings to envelope encryption APIs.
* Added `PkeyRef::size`.

## [v0.10.20] - 2019-03-20

### Added

* Added `CmsContentInfo::from_der` and `CmsContentInfo::encrypt`.
* Added `X509Ref::verify` and `X509ReqRef::verify`.
* Implemented `PartialEq` and `Eq` for `MessageDigest`.
* Added `MessageDigest::type_` and `EcGroupRef::curve_name`.

## [v0.10.19] - 2019-03-01

### Added

* The openssl-sys build script now logs the values of environment variables.
* Added `ERR_PACK` to openssl-sys.
* The `ERR_*` functions in openssl-sys are const functions when building against newer Rust versions.
* Implemented `Clone` for `Dsa`.
* Added `SslContextRef::add_session` and `SslContextRef::remove_session`.
* Added `SslSessionRef::time`, `SslSessionRef::timeout`, and `SslSessionRef::protocol_version`.
* Added `SslContextBuilder::set_session_cache_size` and `SslContextRef::session_cache_size`.

## [v0.10.18] - 2019-02-22

### Fixed

* Fixed the return type of `ssl::cipher_name`.

## [v0.10.17] - 2019-02-22

### Added

* Implemented `AsRef<str>` and `AsRef<[u8]>` for `OpenSslString`.
* Added `Asn1Integer::from_bn`.
* Added `RsaRef::check_key`.
* Added `Asn1Time::from_str` and `Asn1Time::from_str_x509`.
* Added `Rsa::generate_with_e`.
* Added `Cipher::des_ede3_cfb64`.
* Added `SslCipherRef::standard_name` and `ssl::cipher_name`.

## [v0.10.16] - 2018-12-16

### Added

* Added SHA3 and SHAKE to `MessageDigest`.
* Added `rand::keep_random_devices_open`.
* Added support for LibreSSL 2.9.0.

## [v0.10.15] - 2018-10-22

### Added

* Implemented `DoubleEndedIterator` for stack iterators.

## [v0.10.14] - 2018-10-18

### Fixed

* Made some accidentally exposed internal functions private.

### Added

* Added support for LibreSSL 2.8.

### Changed

* The OpenSSL version used with the `vendored` feature has been upgraded from 1.1.0 to 1.1.1.

## [v0.10.13] - 2018-10-14

### Fixed

* Fixed a double-free in the `SslContextBuilder::set_get_session_callback` API.

### Added

* Added `SslContextBuilder::set_client_hello_callback`.
* Added support for LibreSSL 2.8.1.
* Added `EcdsaSig::from_der` and `EcdsaSig::to_der`.
* Added PKCS#7 support.

## [v0.10.12] - 2018-09-13

### Fixed

* Fixed handling of SNI callbacks during renegotiation.

### Added

* Added `SslRef::get_shutdown` and `SslRef::set_shutdown`.
* Added support for SRTP in DTLS sessions.
* Added support for LibreSSL 2.8.0.

## [v0.10.11] - 2018-08-04

### Added

* The new `vendored` cargo feature will cause openssl-sys to compile and statically link to a
    vendored copy of OpenSSL.
* Added `SslContextBuilder::set_psk_server_callback`.
* Added `DsaRef::pub_key` and `DsaRef::priv_key`.
* Added `Dsa::from_private_components` and `Dsa::from_public_components`.
* Added `X509NameRef::entries`.

### Deprecated

* `SslContextBuilder::set_psk_callback` has been renamed to
    `SslContextBuilder::set_psk_client_callback` and deprecated.

## [v0.10.10] - 2018-06-06

### Added

* Added `SslRef::set_alpn_protos`.
* Added `SslContextBuilder::set_ciphersuites`.

## [v0.10.9] - 2018-06-01

### Fixed

* Fixed a use-after-free in `CmsContentInfo::sign`.
* `SslRef::servername` now returns `None` rather than panicking on a non-UTF8 name.

### Added

* Added `MessageDigest::from_nid`.
* Added `Nid::signature_algorithms`, `Nid::long_name`, and `Nid::short_name`.
* Added early data and early keying material export support for TLS 1.3.
* Added `SslRef::verified_chain`.
* Added `SslRef::servername_raw` which returns a `&[u8]` rather than `&str`.
* Added `SslRef::finished` and `SslRef::peer_finished`.
* Added `X509Ref::digest` to replace `X509Ref::fingerprint`.
* `X509StoreBuilder` and `X509Store` now implement `Sync` and `Send`.

### Deprecated

* `X509Ref::fingerprint` has been deprecated in favor of `X509Ref::digest`.

## [v0.10.8] - 2018-05-20

### Fixed

* `openssl-sys` will now detect Homebrew-installed OpenSSL when installed to a non-default
    directory.
* The `X509_V_ERR_INVALID_CALL`, `X509_V_ERR_STORE_LOOKUP`, and
    `X509_V_ERR_PROXY_SUBJECT_NAME_VIOLATION` constants in `openssl-sys` are now only present when
    building against 1.1.0g and up rather than 1.1.0.
* `SslContextBuilder::max_proto_version` and `SslContextBuilder::min_proto_version` are only present
    when building against 1.1.0g and up rather than 1.1.0.

### Added

* Added `CmsContentInfo::sign`.
* Added `Clone` and `ToOwned` implementations to `Rsa` and `RsaRef` respectively.
* The `min_proto_version` and `max_proto_version` methods are available when linking against
    LibreSSL 2.6.1 and up in addition to OpenSSL.
* `X509VerifyParam` is available when linking against LibreSSL 2.6.1 and up in addition to OpenSSL.
* ALPN support is available when linking against LibreSSL 2.6.1 and up in addition to OpenSSL.
* `Stack` and `StackRef` are now `Sync` and `Send`.

## [v0.10.7] - 2018-04-30

### Added

* Added `X509Req::public_key` and `X509Req::extensions`.
* Added `RsaPrivateKeyBuilder` to allow control over initialization of optional components of an RSA
    private key.
* Added DER encode/decode support to `SslSession`.
* openssl-sys now provides the `DEP_OPENSSL_VERSION_NUMBER` and
    `DEP_OPENSSL_LIBRESSL_VERSION_NUMBER` environment variables to downstream build scripts which
    contains the hex-encoded version number of the OpenSSL or LibreSSL distribution being built
    against. The other variables are deprecated.

## [v0.10.6] - 2018-03-05

### Added

* Added `SslOptions::ENABLE_MIDDLEBOX_COMPAT`.
* Added more `Sync` and `Send` implementations.
* Added `PKeyRef::id`.
* Added `Padding::PKCS1_PSS`.
* Added `Signer::set_rsa_pss_saltlen`, `Signer::set_rsa_mgf1_md`, `Signer::set_rsa_pss_saltlen`, and
    `Signer::set_rsa_mgf1_md`
* Added `X509StoreContextRef::verify` to directly verify certificates.
* Added low level ECDSA support.
* Added support for TLSv1.3 custom extensions. (OpenSSL 1.1.1 only)
* Added AES-CCM support.
* Added `EcKey::from_private_components`.
* Added CMAC support.
* Added support for LibreSSL 2.7.
* Added `X509Ref::serial_number`.
* Added `Asn1IntegerRef::to_bn`.
* Added support for TLSv1.3 stateless handshakes. (OpenSSL 1.1.1 only)

### Changed

* The Cargo features previously used to gate access to version-specific OpenSSL APIs have been
    removed. Those APIs will be available automatically when building against an appropriate OpenSSL
    version.
* Fixed `PKey::private_key_from_der` to return a `PKey<Private>` rather than a `PKey<Public>`. This
    is technically a breaking change but the function was pretty useless previously.

### Deprecated

* `X509CheckFlags::FLAG_NO_WILDCARDS` has been renamed to `X509CheckFlags::NO_WILDCARDS` and the old
    name deprecated.

## [v0.10.5] - 2018-02-28

### Fixed

* `ErrorStack`'s `Display` implementation no longer writes an empty string if it contains no errors.

### Added

* Added `SslRef::version2`.
* Added `Cipher::des_ede3_cbc`.
* Added `SslRef::export_keying_material`.
* Added the ability to push an `Error` or `ErrorStack` back onto OpenSSL's error stack. Various
    callback bindings use this to propagate errors properly.
* Added `SslContextBuilder::set_cookie_generate_cb` and `SslContextBuilder::set_cookie_verify_cb`.
* Added `SslContextBuilder::set_max_proto_version`, `SslContextBuilder::set_min_proto_version`,
    `SslContextBuilder::max_proto_version`, and `SslContextBuilder::min_proto_version`.

### Changed

* Updated `SslConnector`'s default cipher list to match Python's.

### Deprecated

* `SslRef::version` has been deprecated. Use `SslRef::version_str` instead.

## [v0.10.4] - 2018-02-18

### Added

* Added OpenSSL 1.1.1 support.
* Added `Rsa::public_key_from_pem_pkcs1`.
* Added `SslOptions::NO_TLSV1_3`. (OpenSSL 1.1.1 only)
* Added `SslVersion`.
* Added `SslSessionCacheMode` and `SslContextBuilder::set_session_cache_mode`.
* Added `SslContextBuilder::set_new_session_callback`,
    `SslContextBuilder::set_remove_session_callback`, and
    `SslContextBuilder::set_get_session_callback`.
* Added `SslContextBuilder::set_keylog_callback`. (OpenSSL 1.1.1 only)
* Added `SslRef::client_random` and `SslRef::server_random`. (OpenSSL 1.1.0+ only)

### Fixed

* The `SslAcceptorBuilder::mozilla_modern` constructor now disables TLSv1.0 and TLSv1.1 in
    accordance with Mozilla's recommendations.

## [v0.10.3] - 2018-02-12

### Added

* OpenSSL is now automatically detected on FreeBSD systems.
* Added `GeneralName` accessors for `rfc822Name` and `uri` variants.
* Added DES-EDE3 support.

### Fixed

* Fixed a memory leak in `X509StoreBuilder::add_cert`.

## [v0.10.2] - 2018-01-11

### Added

* Added `ConnectConfiguration::set_use_server_name_indication` and
    `ConnectConfiguration::set_verify_hostname` for use in contexts where you don't have ownership
    of the `ConnectConfiguration`.

## [v0.10.1] - 2018-01-10

### Added

* Added a `From<ErrorStack> for ssl::Error` implementation.

## [v0.10.0] - 2018-01-10

### Compatibility

* openssl 0.10 still uses openssl-sys 0.9, so openssl 0.9 and 0.10 can coexist without issue.

### Added

* The `ssl::select_next_proto` function can be used to easily implement the ALPN selection callback
    in a "standard" way.
* FIPS mode support is available in the `fips` module.
* Accessors for the Issuer and Issuer Alternative Name fields of X509 certificates have been added.
* The `X509VerifyResult` can now be set in the certificate verification callback via
    `X509StoreContextRef::set_error`.

### Changed

* All constants have been moved to associated constants of their type. For example, `bn::MSB_ONE`
    is now `bn::MsbOption::ONE`.
* Asymmetric key types are now parameterized over what they contain. In OpenSSL, the same type is
    used for key parameters, public keys, and private keys. Unfortunately, some APIs simply assume
    that certain components are present and will segfault trying to use things that aren't there.

    The `pkey` module contains new tag types named `Params`, `Public`, and `Private`, and the
    `Dh`, `Dsa`, `EcKey`, `Rsa`, and `PKey` have a type parameter set to one of those values. This
    allows the `Signer` constructor to indicate that it requires a private key at compile time for
    example. Previously, `Signer` would simply segfault if provided a key without private
    components.
* ALPN support has been changed to more directly model OpenSSL's own APIs. Instead of a single
    method used for both the server and client sides which performed everything automatically, the
    `SslContextBuilder::set_alpn_protos` and `SslContextBuilder::set_alpn_select_callback` handle
    the client and server sides respectively.
* `SslConnector::danger_connect_without_providing_domain_for_certificate_verification_and_server_name_indication`
    has been removed in favor of new methods which provide more control. The
    `ConnectConfiguration::use_server_name_indication` method controls the use of Server Name
    Indication (SNI), and the `ConnectConfiguration::verify_hostname` method controls the use of
    hostname verification. These can be controlled independently, and if both are disabled, the
    domain argument to `ConnectConfiguration::connect` is ignored.
* Shared secret derivation is now handled by the new `derive::Deriver` type rather than
    `pkey::PKeyContext`, which has been removed.
* `ssl::Error` is now no longer an enum, and provides more direct access to the relevant state.
* `SslConnectorBuilder::new` has been moved and renamed to `SslConnector::builder`.
* `SslAcceptorBuilder::mozilla_intermediate` and `SslAcceptorBuilder::mozilla_modern` have been
    moved to `SslAcceptor` and no longer take the private key and certificate chain. Install those
    manually after creating the builder.
* `X509VerifyError` is now `X509VerifyResult` and can now have the "ok" value in addition to error
    values.
* `x509::X509FileType` is now `ssl::SslFiletype`.
* Asymmetric key serialization and deserialization methods now document the formats that they
    correspond to, and some have been renamed to better indicate that.

### Removed

* All deprecated APIs have been removed.
* NPN support has been removed. It has been supersceded by ALPN, and is hopefully no longer being
    used in practice. If you still depend on it, please file an issue!
* `SslRef::compression` has been removed.
* Some `ssl::SslOptions` flags have been removed as they no longer do anything.

## Older

Look at the [release tags] for information about older releases.

[Unreleased]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.41...master
[v0.10.41]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.40...openssl-v0.10.41
[v0.10.40]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.39...openssl-v0.10.40
[v0.10.39]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.38...openssl-v0.10.39
[v0.10.38]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.37...openssl-v0.10.38
[v0.10.37]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.36...openssl-v0.10.37
[v0.10.36]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.35...openssl-v0.10.36
[v0.10.35]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.34...openssl-v0.10.35
[v0.10.34]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.33...openssl-v0.10.34
[v0.10.33]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.32...openssl-v0.10.33
[v0.10.32]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.31...openssl-v0.10.32
[v0.10.31]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.30...openssl-v0.10.31
[v0.10.30]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.29...openssl-v0.10.30
[v0.10.29]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.28...openssl-v0.10.29
[v0.10.28]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.27...openssl-v0.10.28
[v0.10.27]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.26...openssl-v0.10.27
[v0.10.26]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.25...openssl-v0.10.26
[v0.10.25]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.24...openssl-v0.10.25
[v0.10.24]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.23...openssl-v0.10.24
[v0.10.23]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.22...openssl-v0.10.23
[v0.10.22]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.21...openssl-v0.10.22
[v0.10.21]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.20...openssl-v0.10.21
[v0.10.20]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.19...openssl-v0.10.20
[v0.10.19]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.18...openssl-v0.10.19
[v0.10.18]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.17...openssl-v0.10.18
[v0.10.17]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.16...openssl-v0.10.17
[v0.10.16]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.15...openssl-v0.10.16
[v0.10.15]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.14...openssl-v0.10.15
[v0.10.14]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.13...openssl-v0.10.14
[v0.10.13]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.12...openssl-v0.10.13
[v0.10.12]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.11...openssl-v0.10.12
[v0.10.11]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.10...openssl-v0.10.11
[v0.10.10]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.9...openssl-v0.10.10
[v0.10.9]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.8...openssl-v0.10.9
[v0.10.8]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.7...openssl-v0.10.8
[v0.10.7]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.6...openssl-v0.10.7
[v0.10.6]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.5...openssl-v0.10.6
[v0.10.5]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.4...openssl-v0.10.5
[v0.10.4]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.3...openssl-v0.10.4
[v0.10.3]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.2...openssl-v0.10.3
[v0.10.2]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.1...openssl-v0.10.2
[v0.10.1]: https://github.com/sfackler/rust-openssl/compare/openssl-v0.10.0...openssl-v0.10.1
[v0.10.0]: https://github.com/sfackler/rust-openssl/compare/v0.9.23...openssl-v0.10.0
[release tags]: https://github.com/sfackler/rust-openssl/releases
