package org.zfnd.ed25519;

import java.math.BigInteger;
import java.security.SecureRandom;
import org.scijava.nativelib.NativeLoader;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class Ed25519Interface {
  private static final Logger logger;
  private static final boolean enabled;

  static {
    logger = LoggerFactory.getLogger(Ed25519Interface.class);
    boolean isEnabled = true;

    try {
      NativeLoader.loadLibrary("ed25519jni");
    } catch (java.io.IOException | UnsatisfiedLinkError e) {
      logger.error("Could not find ed25519jni - Interface is not enabled - ", e);
      isEnabled = false;
    }
    enabled = isEnabled;
  }

  /**
   * Helper method to determine whether the Ed25519 Rust backend is loaded and
   * available.
   *
   * @return whether the Ed25519 Rust backend is enabled
   */
  public static boolean isEnabled() {
    return enabled;
  }

  /**
   * Generate a new Ed25519 signing key seed and check the results for validity. This
   * code is valid but not canonical. If the Rust code ever adds restrictions on which
   * values are allowed, this code will have to stay in sync.
   *
   * @param rng An initialized, secure RNG
   * @return sks 32 byte signing key seed
   */
  private static byte[] genSigningKeySeedFromJava(SecureRandom rng) {
    byte[] seedBytes = new byte[SigningKeySeed.BYTE_LENGTH];

    do {
      rng.nextBytes(seedBytes);
    } while(!SigningKeySeed.bytesAreValid(seedBytes));

    return seedBytes;
  }

  /**
   * Public frontend to use when generating a signing key seed.
   *
   * @param rng source of entropy for key material
   * @return instance of SigningKeySeed containing an EdDSA signing key seed
   */
  public static SigningKeySeed genSigningKeySeed(SecureRandom rng) {
    return new SigningKeySeed(genSigningKeySeedFromJava(rng));
  }

  /**
   * Check if verification key bytes for a verification key are valid.
   *
   * @param vk_bytes 32 byte verification key bytes to verify
   * @return true if valid, false if not
   */
  public static native boolean checkVerificationKeyBytes(byte[] vk_bytes);

  /**
   * Get verification key bytes from a signing key seed.
   *
   * @param sk_seed_bytes 32 byte signing key seed
   * @return 32 byte verification key
   * @throws RuntimeException on error in libed25519
   */
  private static native byte[] getVerificationKeyBytes(byte[] sk_seed_bytes);

  /**
   * Get verification key bytes from a signing key seed.
   *
   * @param seed signing key seed
   * @return verification key bytes
   */
  public static VerificationKeyBytes getVerificationKeyBytes(SigningKeySeed seed) {
    return new VerificationKeyBytes(getVerificationKeyBytes(seed.getSigningKeySeed()));
  }

  /**
   * Creates a signature on msg using the given signing key.
   *
   * @param sk_seed_bytes 32 byte signing key seed
   * @param msg Message of arbitrary length to be signed
   * @return signature data
   * @throws RuntimeException on error in libed25519
   */
  private static native byte[] sign(byte[] sk_seed_bytes, byte[] msg);

  /**
   * Creates a signature on message using the given signing key.
   *
   * @param seed signing key seed
   * @param message Message of arbitrary length to be signed
   * @return signature data
   * @throws RuntimeException on error in libed25519
   */
  public static Signature sign(SigningKeySeed seed, byte[] message) {
    return new Signature(sign(seed.getSigningKeySeed(), message));
  }

  /**
   * Verifies a purported `signature` on the given `msg`.
   *
   * @param vk_bytes 32 byte verification key bytes
   * @param sig 64 byte signature to be verified
   * @param msg Message of arbitrary length to be signed
   * @return true if verified, false if not
   * @throws RuntimeException on error in libed25519
   */
  private static native boolean verify(byte[] vk_bytes, byte[] sig, byte[] msg);

  /**
   * Verifies a purported `signature` on the given `message` with `verificationKey`.
   *
   * @param verificationKey verification key bytes
   * @param signature 64 byte signature to be verified
   * @param message message of arbitrary length to be signed
   * @return true if verified, false if not
   * @throws RuntimeException on error in libed25519
   */
  public static boolean verify(VerificationKeyBytes verificationKey, Signature signature, byte[] message) {
    return verify(verificationKey.getVerificationKeyBytes(), signature.getSignatureBytes(), message);
  }
}
