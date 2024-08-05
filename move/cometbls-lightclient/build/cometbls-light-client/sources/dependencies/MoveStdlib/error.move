/// This module defines a set of canonical error codes which are optional to use by applications for the
/// `abort` and `assert!` features.
///
/// Canonical error codes use the 3 lowest bytes of the u64 abort code range (the upper 5 bytes are free for other use).
/// Of those, the highest byte represents the *error category* and the lower two bytes the *error reason*.
/// Given an error category `0x1` and a reason `0x3`, a canonical abort code looks as `0x10003`.
///
/// A module can use a canonical code with a constant declaration of the following form:
///
/// ```
/// ///  An invalid ASCII character was encountered when creating a string.
/// const EINVALID_CHARACTER: u64 = 0x010003;
/// ```
///
/// This code is both valid in the worlds with and without canonical errors. It can be used as a plain module local
/// error reason understand by the existing error map tooling, or as a canonical code.
///
/// The actual canonical categories have been adopted from Google's canonical error codes, which in turn are derived
/// from Unix error codes [see here](https://cloud.google.com/apis/design/errors#handling_errors). Each code has an
/// associated HTTP error code which can be used in REST apis. The mapping from error code to http code is not 1:1;
/// error codes here are a bit richer than HTTP codes.
module std::error {

  /// Caller specified an invalid argument (http: 400)
  const INVALID_ARGUMENT: u64 = 0x1;

  /// An input or result of a computation is out of range (http: 400)
  const OUT_OF_RANGE: u64 = 0x2;

  /// The system is not in a state where the operation can be performed (http: 400)
  const INVALID_STATE: u64 = 0x3;

  /// Request not authenticated due to missing, invalid, or expired auth token (http: 401)
  const UNAUTHENTICATED: u64 = 0x4;

  /// client does not have sufficient permission (http: 403)
  const PERMISSION_DENIED: u64 = 0x5;

  /// A specified resource is not found (http: 404)
  const NOT_FOUND: u64 = 0x6;

  /// Concurrency conflict, such as read-modify-write conflict (http: 409)
  const ABORTED: u64 = 0x7;

  /// The resource that a client tried to create already exists (http: 409)
  const ALREADY_EXISTS: u64 = 0x8;

  /// Out of gas or other forms of quota (http: 429)
  const RESOURCE_EXHAUSTED: u64 = 0x9;

  /// Request cancelled by the client (http: 499)
  const CANCELLED: u64 = 0xA;

  /// Internal error (http: 500)
  const INTERNAL: u64 = 0xB;

  /// Feature not implemented (http: 501)
  const NOT_IMPLEMENTED: u64 = 0xC;

  /// The service is currently unavailable. Indicates that a retry could solve the issue (http: 503)
  const UNAVAILABLE: u64 = 0xD;

  /// Construct a canonical error code from a category and a reason.
  public fun canonical(category: u64, reason: u64): u64 {
    (category << 16) + reason
  }
  spec canonical {
    pragma opaque = true;
    let shl_res = category << 16;
    ensures [concrete] result == shl_res + reason;
    aborts_if [abstract] false;
    ensures [abstract] result == category;
  }

  /// Functions to construct a canonical error code of the given category.
  public fun invalid_argument(r: u64): u64 {  canonical(INVALID_ARGUMENT, r) }
  public fun out_of_range(r: u64): u64 {  canonical(OUT_OF_RANGE, r) }
  public fun invalid_state(r: u64): u64 {  canonical(INVALID_STATE, r) }
  public fun unauthenticated(r: u64): u64 { canonical(UNAUTHENTICATED, r) }
  public fun permission_denied(r: u64): u64 { canonical(PERMISSION_DENIED, r) }
  public fun not_found(r: u64): u64 { canonical(NOT_FOUND, r) }
  public fun aborted(r: u64): u64 { canonical(ABORTED, r) }
  public fun already_exists(r: u64): u64 { canonical(ALREADY_EXISTS, r) }
  public fun resource_exhausted(r: u64): u64 {  canonical(RESOURCE_EXHAUSTED, r) }
  public fun internal(r: u64): u64 {  canonical(INTERNAL, r) }
  public fun not_implemented(r: u64): u64 {  canonical(NOT_IMPLEMENTED, r) }
  public fun unavailable(r: u64): u64 { canonical(UNAVAILABLE, r) }
}
