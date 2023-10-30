# Change Log

## [Unreleased]

## [v0.9.75] - 2022-07-09

### Added

* Added SM4 bindings.
* Added `EC_GROUP_set_generator` and `EC_POINT_set_affine_coordinates_GFp`.

## [v0.9.74] - 2022-06-01

### Added

* Added `EVP_MD_block_size`.
* Added `X509V3_EXT_add_alias`.
* Added `X509_V_ERR_INVALID_CA` back when building against OpenSSL 3.0.

## [v0.9.73] - 2022-05-02

### Added

* Added support for installations that place libraries in `$OPENSSL_DIR/lib64` in addition to `$OPENSSL_DIR/lib`.
* Added `X509_issuer_name_hash`.
* Added `ASN1_string_set`.
* Added `X509_CRL_dup`, `X509_REQ_dup`, `X509_NAME_dup`, and `X509_dup`.
* Added `X509_print`.
* Added support for LibreSSL 3.5.x.

## [v0.9.72] - 2021-12-11

### Changed

* Temporarily downgraded the vendored OpenSSL back to 1.1.1 due to significant performance regressions. We will move
    back to 3.0.0 when a future release resolves those issues.

### Added

* Added `PKCS12_set_mac`.
* Added `EVP_PKEY_sign_init`, `EVP_PKEY_sign`, `EVP_PKEY_verify_init`, and `EVP_PKEY_verify`.
* Added support for LibreSSL 3.4.x.

## [v0.9.71]

### Fixed

* Fixed linkage to static OpenSSL 3.0.0 libraries on some 32 bit Android targets.

### Added

* Added support for LibreSSL 3.4.1.
* Added `SSL_get_extms_support` and `SSL_CTRL_GET_EXTMS_SUPPORT`.
* Added `OBJ_create`.
* Added `EVP_CIPHER_CTX_get0_cipher`, `EVP_CIPHER_CTX_get_block_size`, `EVP_CIPHER_CTX_get_key_length`,
    `EVP_CIPHER_CTX_get_iv_length`, and `EVP_CIPHER_CTX_get_tag_length`.
* Added `EVP_CIPHER_free`.
* Added `EVP_CIPHER_CTX_rand_key`.
* Added `OSSL_LIB_CTX_new` and `OSSL_LIB_CTX_free`.
* Added `EVP_CIPHER_fetch`.
* Added `EVP_MD_fetch` and `EVP_MD_free`.
* Added `OPENSSL_malloc` and `OPENSSL_free`.
* Added `EVP_DigestSignUpdate` and `EVP_DigestVerifyUpdate`.

## [v0.9.70] - 2021-10-31

### Fixed

* Fixed linkage to static 3.0.0 OpenSSL libraries on some 32 bit architectures.

## [v0.9.69] - 2021-10-31

### Changed

* Upgraded the vendored OpenSSL to 3.0.0.

### Added

* Added support for automatic detection of Homebrew `openssl@3` installs.
* Added `EVP_PKEY_Q_keygen` and `EVP_EC_gen`.

## [v0.9.68] - 2021-10-27

### Added

* Added `BN_bn2binpad`.
* Added `i2d_X509_NAME` and `d2i_X509_NAME`.
* Added `BN_FLG_MALLOCED`, `BN_FLG_STATIC_DATA`, `BN_FLG_CONSTTIME`, and `BN_FLG_SECURE`.
* Added `BN_CTX_secure_new`, `BN_secure_new`, `BN_set_flags`, and `BN_get_flags`.

## [v0.9.67] - 2021-09-21

### Added

* Added support for LibreSSL 3.4.0

## [v0.9.66] - 2021-08-17

### Added

* Added `EVP_seed_cbc`, `EVP_seed_cfb128`, `EVP_seed_ecb`, and `EVP_seed_ofb`.
* Added `OBJ_length` and `OBJ_get0_data`.
* Added `i2d_PKCS8PrivateKey_bio`.

## [v0.9.65] - 2021-06-21

### Fixed

* Restored the accidentally deleted `PEM_read_bio_X509_CRL` function.

## [v0.9.64] - 2021-06-18

### Added

* Added support for OpenSSL 3.x.x.
* Added `SSL_peek`.
* Added `ERR_LIB_ASN1` and `ASN1_R_HEADER_TOO_LONG`.
* Added `d2i_X509_bio`.
* Added `OBJ_nid2obj`.
* Added `RAND_add`.
* Added `SSL_CTX_set_post_handshake_auth`.
* Added `COMP_get_type`.
* Added `X509_get_default_cert_file_env`, `X509_get_default_cert_file`, `X509_get_default_cert_dir_env`, and
    `X509_get_default_cirt_dir`.

## [v0.9.63] - 2021-05-06

### Added

* Added support for LibreSSL 3.3.x.

## [v0.9.62] - 2021-04-28

### Added

* Added support for LibreSSL 3.3.2.
* Added `DH_set0_key`.
* Added `EC_POINT_get_affine_coordinates`.

## [v0.9.61] - 2021-03-13

### Added

* Added support for automatic detection of OpenSSL installations via pkgsrc and MacPorts on macOS.
* Added various `V_ASN1_*` constants.
* Added `DH_generate_parameters_ex`.
* Added `EC_POINT_is_at_infinity` and `EC_POINT_is_on_curve`.
* Added `EVP_CIPHER_nid`.
* Added `EVP_sm3`.
* Added `NID_*` constants related to SM3.
* Added `PKCS7_get0_signers`.
* Added `EVP_PKEY_CTX_set0_rsa_oaep_label`.
* Added `ACCESS_DESCRIPTION` and `ACCESS_DESCRIPTION_free`.

## [v0.9.60] - 2020-12-24

### Added

* Added support for the default Homebrew install directory on ARM.
* Added `EVP_PKEY_CTX_set_rsa_oaep_md` and `EVP_PKEY_CTRL_RSA_OAEP_MD`.

## [v0.9.59] - 2020-12-09

### Added

* Added support for LibreSSL 3.2.x, 3.3.0, and 3.3.1.
* Added `DH_generate_parameters`, `DH_generate_key`, `DH_compute_key`, and `DH_size`.
* Added `NID_X25519`, `NID_X448`, `EVP_PKEY_x25519` and `EVP_PKEY_x448`.
* Added `OBJ_txt2obj`.
* Added `d2i_PKCS7` and `i2d_PKCS7`.
* Added `SRTP_AEAD_AES_128_GCM` and `SRTP_AEAD_AES_256_GCM`.

## [v0.9.58] - 2020-06-05

### Added

* Added `SSL_set_mtu`.
* Added support for LibreSSL 3.2.0.
* Added `PEM_read_bio_EC_PUBKEY`, `PEM_write_bio_EC_PUBKEY`, `d2i_EC_PUBKEY`, and `i2d_EC_PUBKEY`.
* Added `EVP_PKEY_encrypt_init`, `EVP_PKEY_encrypt`, `EVP_PKEY_decrypt_init`, `EVP_PKEY_decrypt`,
    `EVP_PKEY_get_raw_public_key`, `EVP_PKEY_new_raw_public_key`, `EVP_PKEY_get_raw_private_key`,
    and `EVP_PKEY_new_raw_private_key`.
* Added `OBJ_sn2nid`.

## [v0.9.57] - 2020-05-24

### Added

* Added support for LibreSSL 3.1.x.

## [v0.9.56] - 2020-05-07

### Fixed

* Fixed vendored builds on windows-gnu targets.

### Added

* Added support for LibreSSL 3.0.0.

## [v0.9.55] - 2020-04-07

### Fixed

* Fixed windows-msvc library names when using OpenSSL from vcpkg.

### Added

* If the `OPENSSL_NO_VENDOR` environment variable is set, vendoring will not be used even if enabled.
* Added `SSL_CTX_get_verify_mode` and `SSL_get_verify_mode`.
* Added `SSL_is_init_finished`.
* Added `SSL_CTX_set_cert_store`.
* Added `TLS_server_method` and `TLS_client_method`.
* Added `X509_STORE_get0_objects`.
* Added `X509_OBJECT_free`, `X509_OBJECT_get_type`, and `X509_OBJECT_get0_X509`.

## [v0.9.54] - 2020-01-29

### Added

* Added `BIO_CTRL_DGRAM_QUERY_MTU`.
* Added `EVP_EncryptInit_ex`, `EVP_EncryptFinal_ex`, `EVP_DecryptInit_ex`, and `EVP_DecryptFinal_ex`.
* Added `EVP_md_null`.
* Added `EVP_PKCS82PKEY`.
* Added `PKCS8_PRIV_KEY_INFO`, `d2i_PKCS8_PRIV_KEY_INFO`, and `PKCS8_PRIV_KEY_INFO_free`.
* Added `SSL_OP_NO_RENEGOTIATION`.

## [v0.9.53] - 2019-11-22

### Added

* Added `ASN1_TIME_diff`.
* Added `EC_GROUP_order_bits`.
* Added `EVP_EncodeBlock` and `EVP_DecodeBlock`.
* Added `SSL_CTRL_SET_GROUPS_LIST`, `SSL_CTRL_SET_SIGALGS_LIST`, `SSL_CTX_set1_groups_list`, and
    `SSL_CTX_set1_sigalgs_list`.
* Added `Clone` implementations to `SHA_CTX`, `SHA256_CTX`, and `SHA512_CTX`.

## [v0.9.52] - 2019-10-19

### Added

* Added support for LibreSSL 3.0.x.

## [v0.9.51] - 2019-10-02

### Added

* Added support for LibreSSL 3.0.1.

## [v0.9.50] - 2019-10-02

### Added

* Added `CRYPTO_LOCK_EVP_PKEY`.
* Added `EVP_PKEY_ED25519` and `EVP_PKEY_ED448`.
* Added `EVP_DigestSign` and `EVP_DigestVerify`.
* Added `EVP_PKEY_up_ref`.
* Added `NID_ED25519` and `NID_ED448`.

## [v0.9.49] - 2019-08-15

### Added

* Added support for LibreSSL 3.0.0.

## [v0.9.48] - 2019-07-19

### Added

* Added `AES_wrap_key` and `AES_unwrap_key`.
* Added `EC_GROUP_get_cofactor`, `EC_GROUP_get0_generator`, and `EC_POINT_dup`.
* Added `EVP_aes_128_ofb`, `EVP_aes_192_ecb`, `EVP_aes_192_cbc`, `EVP_aes_192_cfb1`, `EVP_aes_192_cfb8`,
    `EVP_aes_192_cfb_128`, `EVP_aes_192_ctr`, `EVP_aes_192_ccm`, `EVP_aes_192_gcm`, `EVP_aes_192_ofb`, and
    `EVP_aes_256_ofb`.
* Added `PEM_read_bio_CMS` and `PEM_write_bio_CMS`.

## [v0.9.47] - 2019-05-18

### Added

* Added `SSL_CTX_add_client_CA`.

## [v0.9.46] - 2019-05-08

### Added

* Added support for the LibreSSL 2.9.x series.

## [v0.9.45] - 2019-05-03

### Fixed

* Reverted a change to windows-gnu library names that caused regressions.

## [v0.9.44] - 2019-04-30

### Added

* The `DEP_OPENSSL_VENDORED` environment variable tells downstream build scripts if the vendored feature was enabled.
* Added `EVP_SealInit`, `EVP_SealFinal`, `EVP_EncryptUpdate`, `EVP_OpenInit`, `EVP_OpenFinal`, and `EVP_DecryptUpdate`.
* Added `EVP_PKEY_size`.

### Fixed

* Fixed library names when targeting windows-gnu and pkg-config fails.

## [v0.9.43] - 2019-03-20

### Added

* Added `d2i_CMS_ContentInfo` and `CMS_encrypt`.
* Added `X509_verify` and `X509_REQ_verify`.
* Added `EVP_MD_type` and `EVP_GROUP_get_curve_name`.

[Unreleased]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.75..master
[v0.9.75]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.74...openssl-sys-v0.9.75
[v0.9.74]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.73...openssl-sys-v0.9.74
[v0.9.73]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.72...openssl-sys-v0.9.73
[v0.9.72]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.71...openssl-sys-v0.9.72
[v0.9.71]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.70...openssl-sys-v0.9.71
[v0.9.70]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.69...openssl-sys-v0.9.70
[v0.9.69]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.68...openssl-sys-v0.9.69
[v0.9.68]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.67...openssl-sys-v0.9.68
[v0.9.67]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.66...openssl-sys-v0.9.67
[v0.9.66]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.65...openssl-sys-v0.9.66
[v0.9.65]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.64...openssl-sys-v0.9.65
[v0.9.64]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.63...openssl-sys-v0.9.64
[v0.9.63]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.62...openssl-sys-v0.9.63
[v0.9.62]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.61...openssl-sys-v0.9.62
[v0.9.61]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.60...openssl-sys-v0.9.61
[v0.9.60]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.59...openssl-sys-v0.9.60
[v0.9.59]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.58...openssl-sys-v0.9.59
[v0.9.58]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.57...openssl-sys-v0.9.58
[v0.9.57]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.56...openssl-sys-v0.9.57
[v0.9.56]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.55...openssl-sys-v0.9.56
[v0.9.55]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.54...openssl-sys-v0.9.55
[v0.9.54]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.53...openssl-sys-v0.9.54
[v0.9.53]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.52...openssl-sys-v0.9.53
[v0.9.52]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.51...openssl-sys-v0.9.52
[v0.9.51]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.50...openssl-sys-v0.9.51
[v0.9.50]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.49...openssl-sys-v0.9.50
[v0.9.49]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.48...openssl-sys-v0.9.49
[v0.9.48]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.47...openssl-sys-v0.9.48
[v0.9.47]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.46...openssl-sys-v0.9.47
[v0.9.46]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.45...openssl-sys-v0.9.46
[v0.9.45]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.44...openssl-sys-v0.9.45
[v0.9.44]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.43...openssl-sys-v0.9.44
[v0.9.43]: https://github.com/sfackler/rust-openssl/compare/openssl-sys-v0.9.42...openssl-sys-v0.9.43
