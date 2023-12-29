# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.11.6 (2022-09-27)
### Added
- `ecdsa::recoverable::Signature::from_digest_bytes_trial_recovery` ([#660])

### Changed
- Make `ProjectivePoint` equality and `is_identity` faster ([#650])

[#650]: https://github.com/RustCrypto/elliptic-curves/pull/650
[#660]: https://github.com/RustCrypto/elliptic-curves/pull/660

## 0.11.5 (2022-09-14)
### Added
- Impl `PrehashSigner` and `PrehashVerifier` traits for ECDSA keys ([#653])
- Impl `Keypair` for `SigningKey` ([#654])

[#653]: https://github.com/RustCrypto/elliptic-curves/pull/653
[#654]: https://github.com/RustCrypto/elliptic-curves/pull/654

## 0.11.4 (2022-08-13)
### Added
- Impl `ZeroizeOnDrop` for `ecdsa::SigningKey` and `schnorr::SigningKey` ([#630])

### Changed
- Get rid of eager computation in `mul_shift_vartime()` ([#638])
- Bump the precision of precomputed division for the scalar decomposition ([#639])

[#630]: https://github.com/RustCrypto/elliptic-curves/pull/630
[#638]: https://github.com/RustCrypto/elliptic-curves/pull/638
[#639]: https://github.com/RustCrypto/elliptic-curves/pull/639

## 0.11.3 (2022-07-02)
### Changed
- Relax `DigestSigner` trait bounds ([#613])
- Bump `elliptic-curve` to v0.12.2 ([#616])

[#613]: https://github.com/RustCrypto/elliptic-curves/pull/613
[#616]: https://github.com/RustCrypto/elliptic-curves/pull/616

## 0.11.2 (2022-05-24)
### Changed
- Enable `schnorr` feature by default ([#561])

[#561]: https://github.com/RustCrypto/elliptic-curves/pull/561

## 0.11.1 (2022-05-24)
### Added
- Taproot Schnorr as defined in BIP 340 ([#554], [#557], [#558])
- Re-export low-level `diffie_hellman` function ([#556])

### Changed
- Use SHA-256 for computing RFC6979 for `recoverable::Signature` ([#552])

[#552]: https://github.com/RustCrypto/elliptic-curves/pull/552
[#554]: https://github.com/RustCrypto/elliptic-curves/pull/554
[#556]: https://github.com/RustCrypto/elliptic-curves/pull/556
[#557]: https://github.com/RustCrypto/elliptic-curves/pull/557
[#558]: https://github.com/RustCrypto/elliptic-curves/pull/558

## 0.11.0 (2022-05-09)
### Changed
- Bump `digest` to v0.10 ([#515])
- Make `AffinePoint` to `VerifyingKey` conversion fallible ([#535])
- Rename `recover_verify_key` => `recover_verifying_key` ([#537])
- Rename `recover_verify_key_from_digest` => `recover_verifying_key_from_digest` ([#537])
- Have `pkcs8` feature activate `ecdsa/pkcs8` ([#538])
- Bump `elliptic-curve` to v0.12 ([#544])
- Bump `ecdsa` to v0.14 ([#544])

### Fixed
- `hash2curve` crate feature ([#519])

[#515]: https://github.com/RustCrypto/elliptic-curves/pull/515
[#519]: https://github.com/RustCrypto/elliptic-curves/pull/519
[#535]: https://github.com/RustCrypto/elliptic-curves/pull/535
[#537]: https://github.com/RustCrypto/elliptic-curves/pull/537
[#538]: https://github.com/RustCrypto/elliptic-curves/pull/538
[#544]: https://github.com/RustCrypto/elliptic-curves/pull/544

## 0.10.4 (2022-03-15)
### Fixed
- Normalize before calling `is_odd()` in `sng0()` ([#533])

[#533]: https://github.com/RustCrypto/elliptic-curves/pull/533

## 0.10.3 (2022-03-14)
### Fixed
- Do not normalize the argument in `FieldElementImpl::is_odd()` ([#530])

[#530]: https://github.com/RustCrypto/elliptic-curves/pull/530

## 0.10.2 (2022-01-17)
### Added
- hash2curve support: impl `GroupDigest` for `Secp256k1` ([#503])
- `IDENTITY` and `GENERATOR` point constants ([#511])

[#503]: https://github.com/RustCrypto/elliptic-curves/pull/503
[#511]: https://github.com/RustCrypto/elliptic-curves/pull/511

## 0.10.1 (2022-01-04)
### Added
- Impl `ff::Field` trait for `FieldElement` ([#498])
- Impl `ReduceNonZero<U256>` for `Scalar` ([#501])

[#498]: https://github.com/RustCrypto/elliptic-curves/pull/498
[#501]: https://github.com/RustCrypto/elliptic-curves/pull/501

## 0.10.0 (2021-12-14)
### Added
- Implement `Scalar::sqrt` ([#400])
- Impl `PrimeCurveArithmetic` ([#415])
- Impl `Reduce<U256>` for `Scalar` ([#436])
- Impl `Drop` for `ecdsa::SigningKey` ([#449])
- `serde` feature ([#463], [#464])
- Impl `Reduce<U512>` for `Scalar` ([#472])
- Impl `ReduceNonZero<U512>` for `Scalar` ([#474])
- Impl `LinearCombination` trait ([#476])

### Changed
- Make `ecdsa::Signature::normalize_s` non-mutating ([#405])
- Use `PrimeCurve` trait ([#413])
- Use `sec1` crate for `EncodedPoint` type ([#435])
- Replace `ecdsa::hazmat::FromDigest` with `Reduce` ([#438])
- Make `FromEncodedPoint` return a `CtOption` ([#445])
- Rust 2021 edition upgrade; MSRV 1.56+ ([#453])
- Bump `elliptic-curve` crate dependency to v0.11 ([#466])
- Bump `ecdsa` crate dependency to v0.13 ([#467])

### Fixed
- Handle identity point in `GroupEncoding` ([#446])

### Removed
- `force-32-bit` feature ([#399])
- `field-montgomery` feature ([#404])
- `Scalar::conditional_add_bit` ([#431])
- Deprecated `SigningKey::verify_key` method ([#461])

[#399]: https://github.com/RustCrypto/elliptic-curves/pull/399
[#400]: https://github.com/RustCrypto/elliptic-curves/pull/400
[#404]: https://github.com/RustCrypto/elliptic-curves/pull/404
[#405]: https://github.com/RustCrypto/elliptic-curves/pull/405
[#413]: https://github.com/RustCrypto/elliptic-curves/pull/413
[#415]: https://github.com/RustCrypto/elliptic-curves/pull/415
[#431]: https://github.com/RustCrypto/elliptic-curves/pull/431
[#435]: https://github.com/RustCrypto/elliptic-curves/pull/435
[#436]: https://github.com/RustCrypto/elliptic-curves/pull/436
[#438]: https://github.com/RustCrypto/elliptic-curves/pull/438
[#445]: https://github.com/RustCrypto/elliptic-curves/pull/445
[#446]: https://github.com/RustCrypto/elliptic-curves/pull/446
[#449]: https://github.com/RustCrypto/elliptic-curves/pull/449
[#453]: https://github.com/RustCrypto/elliptic-curves/pull/453
[#461]: https://github.com/RustCrypto/elliptic-curves/pull/461
[#463]: https://github.com/RustCrypto/elliptic-curves/pull/463
[#464]: https://github.com/RustCrypto/elliptic-curves/pull/464
[#466]: https://github.com/RustCrypto/elliptic-curves/pull/466
[#467]: https://github.com/RustCrypto/elliptic-curves/pull/467
[#472]: https://github.com/RustCrypto/elliptic-curves/pull/472
[#474]: https://github.com/RustCrypto/elliptic-curves/pull/474
[#476]: https://github.com/RustCrypto/elliptic-curves/pull/476

## 0.9.6 (2021-07-22)
### Added
- Wycheproof test vectors ([#384])

### Fixed
- Edge case in `Scalar::is_high` ([#385])
- Bug in overflow check during 32-bit multiplication ([#388])

[#384]: https://github.com/RustCrypto/elliptic-curves/pull/384
[#385]: https://github.com/RustCrypto/elliptic-curves/pull/385
[#388]: https://github.com/RustCrypto/elliptic-curves/pull/388

## 0.9.5 (2021-07-18)
### Changed
- Optimize ECDSA using linear combination of points ([#380])

[#380]: https://github.com/RustCrypto/elliptic-curves/pull/380

## 0.9.4 (2021-06-23)
### Added
- Derive `Clone` for `ecdsa::SigningKey` ([#374])

[#374]: https://github.com/RustCrypto/elliptic-curves/pull/374

## 0.9.3 (2021-06-21)
### Added
- `ecdsa::SigningKey::verifying_key()` method ([#363])

### Changed
- Deprecate `SigningKey::verify_key()` - use `verifying_key` instead ([#363])
- Bump `elliptic-curve` dependency to v0.10.3 ([#371])

[#363]: https://github.com/RustCrypto/elliptic-curves/pull/363
[#371]: https://github.com/RustCrypto/elliptic-curves/pull/371

## 0.9.2 (2021-06-14) [YANKED]
### Added
- `Debug` impl for `ecdsa::SigningKey` ([#358])
- `ConstantTimeEq`/`Eq`/`PartialEq` impls for `ecdsa::SigningKey` ([#359])

[#358]: https://github.com/RustCrypto/elliptic-curves/pull/358
[#359]: https://github.com/RustCrypto/elliptic-curves/pull/359

## 0.9.1 (2021-06-09) [YANKED]
### Added
- `Copy` impl for `ecdsa::VerifyingKey` ([#355])

[#355]: https://github.com/RustCrypto/elliptic-curves/pull/355

## 0.9.0 (2021-06-08) [YANKED]
### Added
- Derive `Ord` on `ecdsa::VerifyingKey` ([#343])
- `AffineArithmetic` trait impl ([#347])
- `PrimeCurve` trait impls ([#350])

### Changed
- Bump `elliptic-curve` to v0.10; MSRV 1.51+ ([#349])
- Bump `ecdsa` to v0.12 ([#349])

[#343]: https://github.com/RustCrypto/elliptic-curves/pull/343
[#347]: https://github.com/RustCrypto/elliptic-curves/pull/347
[#349]: https://github.com/RustCrypto/elliptic-curves/pull/349
[#350]: https://github.com/RustCrypto/elliptic-curves/pull/350

## 0.8.1 (2021-05-10)
### Fixed
- Mixed coordinate addition with the point at infinity ([#337])

[#337]: https://github.com/RustCrypto/elliptic-curves/pull/337

## 0.8.0 (2021-04-29)
### Added
- `jwk` feature ([#295])
- `Order` constant ([#328])

### Changed
- Rename `ecdsa::Asn1Signature` to `::DerSignature` ([#288])
- Migrate to `FromDigest` trait from `ecdsa` crate ([#292])
- Bump `elliptic-curve` to v0.9.2 ([#296])
- Bump `pkcs8` to v0.6 ([#319])
- Bump `ecdsa` crate dependency to v0.11 ([#330])

### Fixed
- `DigestPrimitive` feature gating ([#324])

[#288]: https://github.com/RustCrypto/elliptic-curves/pull/288
[#292]: https://github.com/RustCrypto/elliptic-curves/pull/292
[#295]: https://github.com/RustCrypto/elliptic-curves/pull/295
[#296]: https://github.com/RustCrypto/elliptic-curves/pull/296
[#319]: https://github.com/RustCrypto/elliptic-curves/pull/319
[#324]: https://github.com/RustCrypto/elliptic-curves/pull/324
[#328]: https://github.com/RustCrypto/elliptic-curves/pull/328
[#330]: https://github.com/RustCrypto/elliptic-curves/pull/330

## 0.7.3 (2021-04-16)
### Changed
- Make `ecdsa` a default feature ([#325])

[#325]: https://github.com/RustCrypto/elliptic-curves/pull/325

## 0.7.2 (2021-01-13)
### Changed
- Have `std` feature activate `ecdsa-core/std` ([#273])

[#273]: https://github.com/RustCrypto/elliptic-curves/pull/273

## 0.7.1 (2020-12-16)
### Fixed
- Trigger docs.rs rebuild with nightly bugfix ([RustCrypto/traits#412])

[RustCrypto/traits#412]: https://github.com/RustCrypto/traits/pull/412

## 0.7.0 (2020-12-16)
### Changed
- Bump `elliptic-curve` dependency to v0.8 ([#260])
- Bump `ecdsa` to v0.10 ([#260])

[#260]: https://github.com/RustCrypto/elliptic-curves/pull/260

## 0.6.0 (2020-12-06)
### Added
- PKCS#8 support ([#243], [#244], [#251])
- `PublicKey` type ([#239])

### Changed
- Bump `elliptic-curve` crate dependency to v0.7; MSRV 1.46+ ([#247])
- Bump `ecdsa` crate dependency to v0.9 ([#247])
- Make `SigningKey` a newtype of `elliptic_curve::SecretKey` ([#242])

[#251]: https://github.com/RustCrypto/elliptic-curves/pull/251
[#247]: https://github.com/RustCrypto/elliptic-curves/pull/247
[#244]: https://github.com/RustCrypto/elliptic-curves/pull/244
[#243]: https://github.com/RustCrypto/elliptic-curves/pull/243
[#242]: https://github.com/RustCrypto/elliptic-curves/pull/242
[#239]: https://github.com/RustCrypto/elliptic-curves/pull/239

## 0.5.10 (2020-10-25)
### Changed
- Expand README.md ([#233])

[#233]: https://github.com/RustCrypto/elliptic-curves/pull/233

## 0.5.9 (2020-10-08)
### Changed
- Bump `cfg-if` from 0.1.10 to 1.0.0 ([#220])

[#220]: https://github.com/RustCrypto/elliptic-curves/pull/220

## 0.5.8 (2020-10-08)
### Fixed
- Regenerate `rustdoc` on https://docs.rs after nightly breakage

## 0.5.7 (2020-10-08)
### Added
- `SecretValue` impl when `arithmetic` feature is disabled ([#222])

[#222]: https://github.com/RustCrypto/elliptic-curves/pull/222

## 0.5.6 (2020-09-28)
### Added
- Enable `endomorphism-mul` optimizations by default ([#213])

[#213]: https://github.com/RustCrypto/elliptic-curves/pull/213

## 0.5.5 (2020-09-27)
### Added
- Impl `FromEncodedPoint` for `ProjectivePoint` ([#210])
- Impl `ToEncodedPoint` for `ecdsa::VerifyKey` ([#209])

[#210]: https://github.com/RustCrypto/elliptic-curves/pull/210
[#209]: https://github.com/RustCrypto/elliptic-curves/pull/209

## 0.5.4 (2020-09-27)
### Added
- Impl `RecoverableSignPrimtive` on `Scalar` ([#206])
- `recoverable::Signature::recover_verify_key_from_digest_bytes` ([#205])

[#206]: https://github.com/RustCrypto/elliptic-curves/pull/206
[#205]: https://github.com/RustCrypto/elliptic-curves/pull/205

## 0.5.3 (2020-09-23)
### Added
- Derive `Copy` on `VerifyKey` ([#202])

[#202]: https://github.com/RustCrypto/elliptic-curves/pull/202

## 0.5.2 (2020-09-22)
### Fixed
- Corrected imports when using `ecdsa` + `keccak256` features ([#199])

[#199]: https://github.com/RustCrypto/elliptic-curves/pull/199

## 0.5.1 (2020-09-21)
### Added
- Documentation for `sha256` feature ([#197])
- `sec1::EncodedPoint::decompress` test ([#194])
- Impl `RandomizedSigner` on `SigningKey` ([#193])

### Changed
- Gate ecdsa::{Signer, Verifier} impls on `sha256` feature ([#192])

[#197]: https://github.com/RustCrypto/elliptic-curves/pull/197
[#194]: https://github.com/RustCrypto/elliptic-curves/pull/194
[#193]: https://github.com/RustCrypto/elliptic-curves/pull/193
[#192]: https://github.com/RustCrypto/elliptic-curves/pull/192

## 0.5.0 (2020-09-17)
### Added
- `ecdsa::Asn1Signature` type alias ([#186])
- `ff` and `group` crate dependencies; MSRV 1.44+ ([#164], [#174])
- `AffinePoint::identity()` and `::is_identity()` ([#165])
- `expose-field` feature ([#161])
- `keccak256` feature ([#142])

### Changed
- Bump `elliptic-curve` crate to v0.6; `ecdsa` to v0.8 ([#180])
- Refactor ProjectiveArithmetic trait ([#179])
- Support generic inner type for `elliptic_curve::SecretKey<C>` ([#177])
- Rename `ElementBytes` => `FieldBytes` ([#176])
- Factor out a `from_digest_trial_recovery` method ([#168])
- Rename `ecdsa::{Signer, Verifier}` => `::{SigningKey, VerifyKey}` ([#153])
- Rename `Curve::ElementSize` => `FieldSize` ([#150])
- Implement RFC6979 deterministic ECDSA ([#146])
- Use `NonZeroScalar` for ECDSA signature components ([#144])
- Eagerly verify ECDSA scalars are in range ([#143])
- Rename `PublicKey` to `EncodedPoint` ([#141])

### Removed
- `rand` feature ([#162])

[#186]: https://github.com/RustCrypto/elliptic-curves/pull/186
[#180]: https://github.com/RustCrypto/elliptic-curves/pull/180
[#179]: https://github.com/RustCrypto/elliptic-curves/pull/179
[#177]: https://github.com/RustCrypto/elliptic-curves/pull/177
[#176]: https://github.com/RustCrypto/elliptic-curves/pull/176
[#174]: https://github.com/RustCrypto/elliptic-curves/pull/174
[#168]: https://github.com/RustCrypto/elliptic-curves/pull/168
[#165]: https://github.com/RustCrypto/elliptic-curves/pull/165
[#164]: https://github.com/RustCrypto/elliptic-curves/pull/164
[#162]: https://github.com/RustCrypto/elliptic-curves/pull/162
[#161]: https://github.com/RustCrypto/elliptic-curves/pull/161
[#153]: https://github.com/RustCrypto/elliptic-curves/pull/153
[#150]: https://github.com/RustCrypto/elliptic-curves/pull/150
[#146]: https://github.com/RustCrypto/elliptic-curves/pull/146
[#144]: https://github.com/RustCrypto/elliptic-curves/pull/144
[#143]: https://github.com/RustCrypto/elliptic-curves/pull/143
[#142]: https://github.com/RustCrypto/elliptic-curves/pull/142
[#141]: https://github.com/RustCrypto/elliptic-curves/pull/141

## 0.4.2 (2020-08-11)
### Fixed
- Builds with either `ecdsa-core` or `sha256` in isolation ([#133])

[#133]: https://github.com/RustCrypto/elliptic-curves/pull/133

## 0.4.1 (2020-08-10)
### Fixed
- secp256k1 rustdoc link ([#131])

[#131]: https://github.com/RustCrypto/elliptic-curves/pull/131

## 0.4.0 (2020-08-10)
### Added
- ECDSA support ([#73], [#101], [#104], [#105])
- ECDSA public key recovery support ([#110])
- OID support ([#103], [#113])
- Elliptic Curve Diffie-Hellman ([#120])
- `Zeroize` impl for `AffinePoint` and `FieldElement` types ([#124])

### Changed
- Optimized field arithmetic with 32-bit and 64-bit backends ([#59], [#82])
- Bump `elliptic-curve` crate dependency to v0.5 ([#126])

[#59]: https://github.com/RustCrypto/elliptic-curves/pull/59
[#73]: https://github.com/RustCrypto/elliptic-curves/pull/73
[#82]: https://github.com/RustCrypto/elliptic-curves/pull/82
[#101]: https://github.com/RustCrypto/elliptic-curves/pull/101
[#103]: https://github.com/RustCrypto/elliptic-curves/pull/103
[#104]: https://github.com/RustCrypto/elliptic-curves/pull/104
[#105]: https://github.com/RustCrypto/elliptic-curves/pull/105
[#110]: https://github.com/RustCrypto/elliptic-curves/pull/110
[#113]: https://github.com/RustCrypto/elliptic-curves/pull/113
[#120]: https://github.com/RustCrypto/elliptic-curves/pull/120
[#124]: https://github.com/RustCrypto/elliptic-curves/pull/124
[#126]: https://github.com/RustCrypto/elliptic-curves/pull/126

## 0.3.0 (2020-06-08)
### Changed
- Bump `elliptic-curve` crate dependency to v0.4 ([#39])

[#39]: https://github.com/RustCrypto/elliptic-curves/pull/39

## 0.2.0 (2020-04-30)
### Added
- Field arithmetic, point addition/doubling, and scalar multiplication ([#19])

[#19]: https://github.com/RustCrypto/elliptic-curves/pull/19

## 0.1.1 (2020-04-20)
### Fixed
- README.md: fix typo in crate name ([#16])

[#16]: https://github.com/RustCrypto/elliptic-curves/pull/16

## 0.1.0 (2020-01-15)
- Initial release
