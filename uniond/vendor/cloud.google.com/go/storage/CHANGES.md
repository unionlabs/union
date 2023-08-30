# Changes


## [1.30.1](https://github.com/googleapis/google-cloud-go/compare/storage/v1.30.0...storage/v1.30.1) (2023-03-21)


### Bug Fixes

* **storage:** Retract versions with Copier bug ([#7583](https://github.com/googleapis/google-cloud-go/issues/7583)) ([9c10b6f](https://github.com/googleapis/google-cloud-go/commit/9c10b6f8a54cb8447260148b5e4a9b5160281020))
  * Versions v1.25.0-v1.27.0 are retracted due to [#6857](https://github.com/googleapis/google-cloud-go/issues/6857).
* **storage:** SignedURL v4 allows headers with colons in value ([#7603](https://github.com/googleapis/google-cloud-go/issues/7603)) ([6b50f9b](https://github.com/googleapis/google-cloud-go/commit/6b50f9b368f5b271ade1706c342865cef46712e6))

## [1.30.0](https://github.com/googleapis/google-cloud-go/compare/storage/v1.29.0...storage/v1.30.0) (2023-03-15)


### Features

* **storage/internal:** Update routing annotation for CreateBucketRequest docs: Add support for end-to-end checksumming in the gRPC WriteObject flow feat!: BREAKING CHANGE - renaming Notification to NotificationConfig ([2fef56f](https://github.com/googleapis/google-cloud-go/commit/2fef56f75a63dc4ff6e0eea56c7b26d4831c8e27))
* **storage:** Json downloads ([#7158](https://github.com/googleapis/google-cloud-go/issues/7158)) ([574a86c](https://github.com/googleapis/google-cloud-go/commit/574a86c614445f8c3f5a54446820df774c31cd47))
* **storage:** Update iam and longrunning deps ([91a1f78](https://github.com/googleapis/google-cloud-go/commit/91a1f784a109da70f63b96414bba8a9b4254cddd))


### Bug Fixes

* **storage:** Specify credentials with STORAGE_EMULATOR_HOST ([#7271](https://github.com/googleapis/google-cloud-go/issues/7271)) ([940ae15](https://github.com/googleapis/google-cloud-go/commit/940ae15f725ff384e345e627feb03d22e1fd8db5))

## [1.29.0](https://github.com/googleapis/google-cloud-go/compare/storage/v1.28.1...storage/v1.29.0) (2023-01-19)


### Features

* **storage:** Add ComponentCount as part of ObjectAttrs ([#7230](https://github.com/googleapis/google-cloud-go/issues/7230)) ([a19bca6](https://github.com/googleapis/google-cloud-go/commit/a19bca60704b4fbb674cf51d828580aa653c8210))
* **storage:** Add REST client ([06a54a1](https://github.com/googleapis/google-cloud-go/commit/06a54a16a5866cce966547c51e203b9e09a25bc0))


### Documentation

* **storage/internal:** Corrected typos and spellings ([7357077](https://github.com/googleapis/google-cloud-go/commit/735707796d81d7f6f32fc3415800c512fe62297e))

## [1.28.1](https://github.com/googleapis/google-cloud-go/compare/storage/v1.28.0...storage/v1.28.1) (2022-12-02)


### Bug Fixes

* **storage:** downgrade some dependencies ([7540152](https://github.com/googleapis/google-cloud-go/commit/754015236d5af7c82a75da218b71a87b9ead6eb5))

## [1.28.0](https://github.com/googleapis/google-cloud-go/compare/storage/v1.27.0...storage/v1.28.0) (2022-11-03)


### Features

* **storage/internal:** Add routing annotations ([ce3f945](https://github.com/googleapis/google-cloud-go/commit/ce3f9458e511eca0910992763232abbcd64698f1))
* **storage:** Add Autoclass support ([#6828](https://github.com/googleapis/google-cloud-go/issues/6828)) ([f7c7f41](https://github.com/googleapis/google-cloud-go/commit/f7c7f41e4d7fcffe05860e1114cb20f40c869da8))


### Bug Fixes

* **storage:** Fix read-write race in Writer.Write ([#6817](https://github.com/googleapis/google-cloud-go/issues/6817)) ([4766d3e](https://github.com/googleapis/google-cloud-go/commit/4766d3e1004119b93c6bd352024b5bf3404252eb))
* **storage:** Fix request token passing for Copier.Run ([#6863](https://github.com/googleapis/google-cloud-go/issues/6863)) ([faaab06](https://github.com/googleapis/google-cloud-go/commit/faaab066d8e509dc440bcbc87391557ecee7dbf2)), refs [#6857](https://github.com/googleapis/google-cloud-go/issues/6857)


### Documentation

* **storage:** Update broken links for SignURL and PostPolicy ([#6779](https://github.com/googleapis/google-cloud-go/issues/6779)) ([776138b](https://github.com/googleapis/google-cloud-go/commit/776138bc06a1e5fd45acbf8f9d36e9dc6ce31dd3))

## [1.27.0](https://github.com/googleapis/google-cloud-go/compare/storage/v1.26.0...storage/v1.27.0) (2022-09-22)


### Features

* **storage:** Find GoogleAccessID when using impersonated creds ([#6591](https://github.com/googleapis/google-cloud-go/issues/6591)) ([a2d16a7](https://github.com/googleapis/google-cloud-go/commit/a2d16a7a778c85d13217fc67955ec5dac1da34e8))

## [1.26.0](https://github.com/googleapis/google-cloud-go/compare/storage/v1.25.0...storage/v1.26.0) (2022-08-29)


### Features

* **storage:** export ShouldRetry ([#6370](https://github.com/googleapis/google-cloud-go/issues/6370)) ([0da9ab0](https://github.com/googleapis/google-cloud-go/commit/0da9ab0831540569dc04c0a23437b084b1564e15)), refs [#6362](https://github.com/googleapis/google-cloud-go/issues/6362)


### Bug Fixes

* **storage:** allow to use age=0 in OLM conditions ([#6204](https://github.com/googleapis/google-cloud-go/issues/6204)) ([c85704f](https://github.com/googleapis/google-cloud-go/commit/c85704f4284626ce728cb48f3b130f2ce2a0165e))

## [1.25.0](https://github.com/googleapis/google-cloud-go/compare/storage/v1.24.0...storage/v1.25.0) (2022-08-11)


### Features

* **storage/internal:** Add routing annotations ([8a8ba85](https://github.com/googleapis/google-cloud-go/commit/8a8ba85311f85701c97fd7c10f1d88b738ce423f))
* **storage:** refactor to use transport-agnostic interface ([#6465](https://github.com/googleapis/google-cloud-go/issues/6465)) ([d03c3e1](https://github.com/googleapis/google-cloud-go/commit/d03c3e15a79fe9afa1232d9c8bd4c484a9bb927e))

## [1.24.0](https://github.com/googleapis/google-cloud-go/compare/storage/v1.23.0...storage/v1.24.0) (2022-07-20)


### Features

* **storage:** add Custom Placement Config Dual Region Support  ([#6294](https://github.com/googleapis/google-cloud-go/issues/6294)) ([5a8c607](https://github.com/googleapis/google-cloud-go/commit/5a8c607e3a9a3265887e27cb13f8943f3e3fa23d))

## [1.23.0](https://github.com/googleapis/google-cloud-go/compare/storage/v1.22.1...storage/v1.23.0) (2022-06-23)


### Features

* **storage:** add support for OLM Prefix/Suffix ([#5929](https://github.com/googleapis/google-cloud-go/issues/5929)) ([ec21d10](https://github.com/googleapis/google-cloud-go/commit/ec21d10d6d1b01aa97a52560319775041707690d))
* **storage:** support AbortIncompleteMultipartUpload LifecycleAction ([#5812](https://github.com/googleapis/google-cloud-go/issues/5812)) ([fdec929](https://github.com/googleapis/google-cloud-go/commit/fdec929b9da6e01dda0ab3c72544d44d6bd82bd4)), refs [#5795](https://github.com/googleapis/google-cloud-go/issues/5795)


### Bug Fixes

* **storage:** allow for  Age *int64 type and int64 type ([#6230](https://github.com/googleapis/google-cloud-go/issues/6230)) ([cc7acb8](https://github.com/googleapis/google-cloud-go/commit/cc7acb8bffb31828e9e96d4834a65f9728494473))

### [1.22.1](https://github.com/googleapis/google-cloud-go/compare/storage/v1.22.0...storage/v1.22.1) (2022-05-19)


### Bug Fixes

* **storage:** bump genproto, remove deadcode ([#6059](https://github.com/googleapis/google-cloud-go/issues/6059)) ([bb10f9f](https://github.com/googleapis/google-cloud-go/commit/bb10f9faca57dc3b987e0fb601090887b3507f07))
* **storage:** remove field that no longer exists ([#6061](https://github.com/googleapis/google-cloud-go/issues/6061)) ([ee150cf](https://github.com/googleapis/google-cloud-go/commit/ee150cfd194463ddfcb59898cfb0237e47777973))

## [1.22.0](https://github.com/googleapis/google-cloud-go/compare/storage/v1.21.0...storage/v1.22.0) (2022-03-31)


### Features

* **storage:** allow specifying includeTrailingDelimiter ([#5617](https://github.com/googleapis/google-cloud-go/issues/5617)) ([a34503b](https://github.com/googleapis/google-cloud-go/commit/a34503bc0f0b95399285e8db66976b227e3b0072))
* **storage:** set versionClient to module version ([55f0d92](https://github.com/googleapis/google-cloud-go/commit/55f0d92bf112f14b024b4ab0076c9875a17423c9))


### Bug Fixes

* **storage:** respect STORAGE_EMULATOR_HOST in signedURL ([#5673](https://github.com/googleapis/google-cloud-go/issues/5673)) ([1c249ae](https://github.com/googleapis/google-cloud-go/commit/1c249ae5b4980cf53fa74635943ca8bf6a96a341))

## [1.21.0](https://github.com/googleapis/google-cloud-go/compare/storage/v1.20.0...storage/v1.21.0) (2022-02-17)


### Features

* **storage:** add better version metadata to calls ([#5507](https://github.com/googleapis/google-cloud-go/issues/5507)) ([13fe0bc](https://github.com/googleapis/google-cloud-go/commit/13fe0bc0d8acbffd46b59ab69b25449f1cbd6a88)), refs [#2749](https://github.com/googleapis/google-cloud-go/issues/2749)
* **storage:** add Writer.ChunkRetryDeadline ([#5482](https://github.com/googleapis/google-cloud-go/issues/5482)) ([498a746](https://github.com/googleapis/google-cloud-go/commit/498a746769fa43958b92af8875b927879947128e))

## [1.20.0](https://www.github.com/googleapis/google-cloud-go/compare/storage/v1.19.0...storage/v1.20.0) (2022-02-04)


### Features

* **storage/internal:** Update definition of RewriteObjectRequest to bring to parity with JSON API support ([#5447](https://www.github.com/googleapis/google-cloud-go/issues/5447)) ([7d175ef](https://www.github.com/googleapis/google-cloud-go/commit/7d175ef12b7b3e75585427f5dd2aab4a175e92d6))

## [1.19.0](https://www.github.com/googleapis/google-cloud-go/compare/storage/v1.18.2...storage/v1.19.0) (2022-01-25)


### Features

* **storage:** add fully configurable and idempotency-aware retry strategy ([#5384](https://www.github.com/googleapis/google-cloud-go/issues/5384), [#5185](https://www.github.com/googleapis/google-cloud-go/issues/5185), [#5170](https://www.github.com/googleapis/google-cloud-go/issues/5170), [#5223](https://www.github.com/googleapis/google-cloud-go/issues/5223), [#5221](https://www.github.com/googleapis/google-cloud-go/issues/5221), [#5193](https://www.github.com/googleapis/google-cloud-go/issues/5193), [#5159](https://www.github.com/googleapis/google-cloud-go/issues/5159), [#5165](https://www.github.com/googleapis/google-cloud-go/issues/5165), [#5166](https://www.github.com/googleapis/google-cloud-go/issues/5166), [#5210](https://www.github.com/googleapis/google-cloud-go/issues/5210), [#5172](https://www.github.com/googleapis/google-cloud-go/issues/5172), [#5314](https://www.github.com/googleapis/google-cloud-go/issues/5314))
  * This release contains changes to fully align this library's retry strategy
    with best practices as described in the
    Cloud Storage [docs](https://cloud.google.com/storage/docs/retry-strategy).
  * The library will now retry only idempotent operations by default. This means
    that for certain operations, including object upload, compose, rewrite,
    update, and delete, requests will not be retried by default unless
    [idempotency conditions](https://cloud.google.com/storage/docs/retry-strategy#idempotency)
    for the request have been met.
  * The library now has methods to configure aspects of retry policy for
    API calls, including which errors are retried, the timing of the
    exponential backoff, and how idempotency is taken into account.
  * If you wish to re-enable retries for a non-idempotent request, use the
    [RetryAlways](https://pkg.go.dev/cloud.google.com/go/storage@main#RetryAlways)
    policy.
  * For full details on how to configure retries, see the
    [package docs](https://pkg.go.dev/cloud.google.com/go/storage@main#hdr-Retrying_failed_requests)
    and the
    [Cloud Storage docs](https://cloud.google.com/storage/docs/retry-strategy)
* **storage:** GenerateSignedPostPolicyV4 can use existing creds to authenticate ([#5105](https://www.github.com/googleapis/google-cloud-go/issues/5105)) ([46489f4](https://www.github.com/googleapis/google-cloud-go/commit/46489f4c8a634068a3e7cf2fd5e5ca11b555c0a8))
* **storage:** post policy can be signed with a fn that takes raw bytes ([#5079](https://www.github.com/googleapis/google-cloud-go/issues/5079)) ([25d1278](https://www.github.com/googleapis/google-cloud-go/commit/25d1278cab539fbfdd8563ed6b297e30d3fe555c))
* **storage:** add rpo (turbo replication) support ([#5003](https://www.github.com/googleapis/google-cloud-go/issues/5003)) ([3bd5995](https://www.github.com/googleapis/google-cloud-go/commit/3bd59958e0c06d2655b67fcb5410668db3c52af0))

### Bug Fixes

* **storage:** fix nil check in gRPC Reader ([#5376](https://www.github.com/googleapis/google-cloud-go/issues/5376)) ([5e7d722](https://www.github.com/googleapis/google-cloud-go/commit/5e7d722d18a62b28ba98169b3bdbb49401377264))

### [1.18.2](https://www.github.com/googleapis/google-cloud-go/compare/storage/v1.18.1...storage/v1.18.2) (2021-10-18)


### Bug Fixes

* **storage:** upgrade genproto ([#4993](https://www.github.com/googleapis/google-cloud-go/issues/4993)) ([5ca462d](https://www.github.com/googleapis/google-cloud-go/commit/5ca462d99fe851b7cddfd70108798e2fa959bdfd)), refs [#4991](https://www.github.com/googleapis/google-cloud-go/issues/4991)

### [1.18.1](https://www.github.com/googleapis/google-cloud-go/compare/storage/v1.18.0...storage/v1.18.1) (2021-10-14)


### Bug Fixes

* **storage:** don't assume auth from a client option ([#4982](https://www.github.com/googleapis/google-cloud-go/issues/4982)) ([e17334d](https://www.github.com/googleapis/google-cloud-go/commit/e17334d1fe7645d89d14ae7148313498b984dfbb))

## [1.18.0](https://www.github.com/googleapis/google-cloud-go/compare/storage/v1.17.0...storage/v1.18.0) (2021-10-11)


### Features

* **storage:** returned wrapped error for timeouts ([#4802](https://www.github.com/googleapis/google-cloud-go/issues/4802)) ([0e102a3](https://www.github.com/googleapis/google-cloud-go/commit/0e102a385dc67a06f6b444b3a93e6998428529be)), refs [#4197](https://www.github.com/googleapis/google-cloud-go/issues/4197)
* **storage:** SignedUrl can use existing creds to authenticate ([#4604](https://www.github.com/googleapis/google-cloud-go/issues/4604)) ([b824c89](https://www.github.com/googleapis/google-cloud-go/commit/b824c897e6941270747b612f6d36a8d6ae081315))


### Bug Fixes

* **storage:** update PAP to use inherited instead of unspecified ([#4909](https://www.github.com/googleapis/google-cloud-go/issues/4909)) ([dac26b1](https://www.github.com/googleapis/google-cloud-go/commit/dac26b1af2f2972f12775341173bcc5f982438b8))

## [1.17.0](https://www.github.com/googleapis/google-cloud-go/compare/storage/v1.16.1...storage/v1.17.0) (2021-09-28)


### Features

* **storage:** add projectNumber field to bucketAttrs. ([#4805](https://www.github.com/googleapis/google-cloud-go/issues/4805)) ([07343af](https://www.github.com/googleapis/google-cloud-go/commit/07343afc15085b164cc41d202d13f9d46f5c0d02))


### Bug Fixes

* **storage:** align retry idempotency (part 1) ([#4715](https://www.github.com/googleapis/google-cloud-go/issues/4715)) ([ffa903e](https://www.github.com/googleapis/google-cloud-go/commit/ffa903eeec61aa3869e5220e2f09371127b5c393))

### [1.16.1](https://www.github.com/googleapis/google-cloud-go/compare/storage/v1.16.0...storage/v1.16.1) (2021-08-30)


### Bug Fixes

* **storage/internal:** Update encryption_key fields to "bytes" type. fix: Improve date/times and field name clarity in lifecycle conditions. ([a52baa4](https://www.github.com/googleapis/google-cloud-go/commit/a52baa456ed8513ec492c4b573c191eb61468758))
* **storage:** accept emulator env var without scheme ([#4616](https://www.github.com/googleapis/google-cloud-go/issues/4616)) ([5f8cbb9](https://www.github.com/googleapis/google-cloud-go/commit/5f8cbb98070109e2a34409ac775ed63b94d37efd))
* **storage:** preserve supplied endpoint's scheme ([#4609](https://www.github.com/googleapis/google-cloud-go/issues/4609)) ([ee2756f](https://www.github.com/googleapis/google-cloud-go/commit/ee2756fb0a335d591464a770c9fa4f8fe0ba2e01))
* **storage:** remove unnecessary variable ([#4608](https://www.github.com/googleapis/google-cloud-go/issues/4608)) ([27fc784](https://www.github.com/googleapis/google-cloud-go/commit/27fc78456fb251652bdf5cdb493734a7e1e643e1))
* **storage:** retry LockRetentionPolicy ([#4439](https://www.github.com/googleapis/google-cloud-go/issues/4439)) ([09879ea](https://www.github.com/googleapis/google-cloud-go/commit/09879ea80cb67f9bfd8fc9384b0fda335567cba9)), refs [#4437](https://www.github.com/googleapis/google-cloud-go/issues/4437)
* **storage:** revise Reader to send XML preconditions ([#4479](https://www.github.com/googleapis/google-cloud-go/issues/4479)) ([e36b29a](https://www.github.com/googleapis/google-cloud-go/commit/e36b29a3d43bce5c1c044f7daf6e1db00b0a49e0)), refs [#4470](https://www.github.com/googleapis/google-cloud-go/issues/4470)

## [1.16.0](https://www.github.com/googleapis/google-cloud-go/compare/storage/v1.15.0...storage/v1.16.0) (2021-06-28)


### Features

* **storage:** support PublicAccessPrevention ([#3608](https://www.github.com/googleapis/google-cloud-go/issues/3608)) ([99bc782](https://www.github.com/googleapis/google-cloud-go/commit/99bc782fb50a47602b45278384ef5d5b5da9263b)), refs [#3203](https://www.github.com/googleapis/google-cloud-go/issues/3203)


### Bug Fixes

* **storage:** fix Writer.ChunkSize validation ([#4255](https://www.github.com/googleapis/google-cloud-go/issues/4255)) ([69c2e9d](https://www.github.com/googleapis/google-cloud-go/commit/69c2e9dc6303e1a004d3104a8178532fa738e742)), refs [#4167](https://www.github.com/googleapis/google-cloud-go/issues/4167)
* **storage:** try to reopen for failed Reads ([#4226](https://www.github.com/googleapis/google-cloud-go/issues/4226)) ([564102b](https://www.github.com/googleapis/google-cloud-go/commit/564102b335dbfb558bec8af883e5f898efb5dd10)), refs [#3040](https://www.github.com/googleapis/google-cloud-go/issues/3040)

## [1.15.0](https://www.github.com/googleapis/google-cloud-go/compare/storage/v1.13.0...storage/v1.15.0) (2021-04-21)


### Features

* **transport** Bump dependency on google.golang.org/api to pick up HTTP/2
  config updates (see [googleapis/google-api-go-client#882](https://github.com/googleapis/google-api-go-client/pull/882)).

### Bug Fixes

* **storage:** retry io.ErrUnexpectedEOF ([#3957](https://www.github.com/googleapis/google-cloud-go/issues/3957)) ([f6590cd](https://www.github.com/googleapis/google-cloud-go/commit/f6590cdc26c8479be5df48949fa59f879e0c24fc))


## v1.14.0

- Updates to various dependencies.

## [1.13.0](https://www.github.com/googleapis/google-cloud-go/compare/storage/v1.12.0...v1.13.0) (2021-02-03)


### Features

* **storage:** add missing StorageClass in BucketAttrsToUpdate ([#3038](https://www.github.com/googleapis/google-cloud-go/issues/3038)) ([2fa1b72](https://www.github.com/googleapis/google-cloud-go/commit/2fa1b727f8a7b20aa62fe0990530744f6c109be0))
* **storage:** add projection parameter for BucketHandle.Objects() ([#3549](https://www.github.com/googleapis/google-cloud-go/issues/3549)) ([9b9c3dc](https://www.github.com/googleapis/google-cloud-go/commit/9b9c3dce3ee10af5b6c4d070821bf47a861efd5b))


### Bug Fixes

* **storage:** fix endpoint selection logic ([#3172](https://www.github.com/googleapis/google-cloud-go/issues/3172)) ([99edf0d](https://www.github.com/googleapis/google-cloud-go/commit/99edf0d211a9e617f2586fbc83b6f9630da3c537))

## v1.12.0
- V4 signed URL fixes:
  - Fix encoding of spaces in query parameters.
  - Add fields that were missing from PostPolicyV4 policy conditions.
- Fix Query to correctly list prefixes as well as objects when SetAttrSelection
  is used.

## v1.11.0
- Add support for CustomTime and NoncurrentTime object lifecycle management
  features.

## v1.10.0
- Bump dependency on google.golang.org/api to capture changes to retry logic
  which will make retries on writes more resilient.
- Improve documentation for Writer.ChunkSize.
- Fix a bug in lifecycle to allow callers to clear lifecycle rules on a bucket.

## v1.9.0
- Add retry for transient network errors on most operations (with the exception
  of writes).
- Bump dependency for google.golang.org/api to capture a change in the default
  HTTP transport which will improve performance for reads under heavy load.
- Add CRC32C checksum validation option to Composer.

## v1.8.0
- Add support for V4 signed post policies.

## v1.7.0
- V4 signed URL support:
  - Add support for bucket-bound domains and virtual hosted style URLs.
  - Add support for query parameters in the signature.
  - Fix text encoding to align with standards.
- Add the object name to query parameters for write calls.
- Fix retry behavior when reading files with Content-Encoding gzip.
- Fix response header in reader.
- New code examples:
   - Error handling for `ObjectHandle` preconditions.
   - Existence checks for buckets and objects.

## v1.6.0

- Updated option handling:
  - Don't drop custom scopes (#1756)
  - Don't drop port in provided endpoint (#1737)

## v1.5.0

- Honor WithEndpoint client option for reads as well as writes.
- Add archive storage class to docs.
- Make fixes to storage benchwrapper.

## v1.4.0

- When listing objects in a bucket, allow callers to specify which attributes
  are queried. This allows for performance optimization.

## v1.3.0

- Use `storage.googleapis.com/storage/v1` by default for GCS requests
  instead of `www.googleapis.com/storage/v1`.

## v1.2.1

- Fixed a bug where UniformBucketLevelAccess and BucketPolicyOnly were not
  being sent in all cases.

## v1.2.0

- Add support for UniformBucketLevelAccess. This configures access checks
  to use only bucket-level IAM policies.
  See: https://godoc.org/cloud.google.com/go/storage#UniformBucketLevelAccess.
- Fix userAgent to use correct version.

## v1.1.2

- Fix memory leak in BucketIterator and ObjectIterator.

## v1.1.1

- Send BucketPolicyOnly even when it's disabled.

## v1.1.0

- Performance improvements for ObjectIterator and BucketIterator.
- Fix Bucket.ObjectIterator size calculation checks.
- Added HMACKeyOptions to all the methods which allows for options such as
  UserProject to be set per invocation and optionally be used.

## v1.0.0

This is the first tag to carve out storage as its own module. See:
https://github.com/golang/go/wiki/Modules#is-it-possible-to-add-a-module-to-a-multi-module-repository.
