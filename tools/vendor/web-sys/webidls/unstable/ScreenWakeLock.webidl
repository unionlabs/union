/* The origin of this IDL file is
 * https://www.w3.org/TR/screen-wake-lock/#idl-index
 */

[SecureContext]
partial interface Navigator {
  [SameObject] readonly attribute WakeLock wakeLock;
};

[SecureContext, Exposed=(Window)]
interface WakeLock {
  Promise<WakeLockSentinel> request(WakeLockType type);
};

[SecureContext, Exposed=(Window)]
interface WakeLockSentinel : EventTarget {
  readonly attribute boolean released;
  readonly attribute WakeLockType type;
  Promise<undefined> release();
  attribute EventHandler onrelease;
};

enum WakeLockType { "screen" };
