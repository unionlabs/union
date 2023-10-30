package org.zfnd.ed25519;

import java.util.Arrays;
import java.util.Optional;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Java wrapper class for signing key seeds that performs some sanity checking.
 */
public class SigningKeySeed {
    public static final int BYTE_LENGTH = 32;
    private static final Logger logger = LoggerFactory.getLogger(SigningKeySeed.class);

    private byte[] seed;

    // Determining if bytes are valid is pretty trivial. Rust code not needed.
    static boolean bytesAreValid(final byte[] seedBytes) {
        if(seedBytes.length == BYTE_LENGTH) {
            for (int b = 0; b < BYTE_LENGTH; b++) {
                if (seedBytes[b] != 0) {
                    return true;
                }
            }
        }

        return false;
    }

    SigningKeySeed(final byte[] seed) {
        // package protected constructor
        // assumes valid values from us or underlying library and that the caller will not mutate them
        this.seed = seed;
    }

    /**
     * @return a copy of the wrapped bytes
     */
    public byte[] getSigningKeySeedCopy() {
        return seed.clone();
    }

    byte[] getSigningKeySeed() {
        return seed;
    }

    /**
     * Optionally convert bytes into a signing key seed wrapper.
     *
     * @param bytes untrusted, unvalidated bytes that may be a valid signing key seed
     * @return optionally a signing key seed wrapper, if bytes are valid
     */
    public static Optional<SigningKeySeed> fromBytes(final byte[] bytes) {
        // input is mutable and from untrusted source, so take a copy
        final byte[] cloneBytes = bytes.clone();

        if (bytesAreValid(cloneBytes)) {
            return Optional.of(new SigningKeySeed(cloneBytes));
        }
        else {
            return Optional.empty();
        }
    }

    /**
     * Convert bytes into a signing key seed wrapper.
     *
     * @param bytes bytes that are expected be a valid signing key seed
     * @return a signing key seed wrapper, if bytes are valid
     * @throws IllegalArgumentException if bytes are invalid
     */
    public static SigningKeySeed fromBytesOrThrow(final byte[] bytes) {
        return fromBytes(bytes)
            .orElseThrow(() -> new IllegalArgumentException("Expected " + BYTE_LENGTH + " bytes where not all are zero!"));
    }
}
