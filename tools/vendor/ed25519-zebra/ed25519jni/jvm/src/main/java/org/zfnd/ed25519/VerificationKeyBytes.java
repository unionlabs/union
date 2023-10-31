package org.zfnd.ed25519;

import java.util.Arrays;
import java.util.Optional;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Java wrapper class for verification key bytes that performs some sanity checking.
 */
public class VerificationKeyBytes {
    public static final int BYTE_LENGTH = 32;
    private static final Logger logger = LoggerFactory.getLogger(VerificationKeyBytes.class);

    private byte[] vkb;

    // Determining if bytes are valid is complicated. Call into Rust.
    static boolean bytesAreValid(final byte[] verificationKeyBytes) {
        return (verificationKeyBytes.length == BYTE_LENGTH) && Ed25519Interface.checkVerificationKeyBytes(verificationKeyBytes);
    }

    VerificationKeyBytes(final byte[] verificationKeyBytes) {
        // package protected constructor
        // assumes valid values from us or underlying library and that the caller will not mutate them
        this.vkb = verificationKeyBytes;
    }

    /**
     * @return a copy of the wrapped bytes
     */
    public byte[] getVerificationKeyBytesCopy() {
        return vkb.clone();
    }

    byte[] getVerificationKeyBytes() {
        return vkb;
    }

    /**
     * Optionally convert bytes into a verification key wrapper.
     *
     * @param bytes untrusted, unvalidated bytes that may be an encoding of a verification key
     * @return optionally a verification key wrapper, if bytes are valid
     */
    public static Optional<VerificationKeyBytes> fromBytes(final byte[] bytes) {
        // input is mutable and from untrusted source, so take a copy
        final byte[] cloneBytes = bytes.clone();

        if (bytesAreValid(cloneBytes)) {
            return Optional.of(new VerificationKeyBytes(cloneBytes));
        }
        else {
            return Optional.empty();
        }
    }

    /**
     * Convert bytes into a verification key wrapper.
     *
     * @param bytes bytes that are expected be an encoding of a verification key
     * @return a verification key wrapper, if bytes are valid
     * @throws IllegalArgumentException if bytes are invalid
     */
    public static VerificationKeyBytes fromBytesOrThrow(final byte[] bytes) {
        return fromBytes(bytes)
            .orElseThrow(() -> new IllegalArgumentException("Expected " + BYTE_LENGTH + " bytes that encode a verification key!"));
    }

    @Override
    public boolean equals(final Object other) {
        if (other == this) {
            return true;
        } else if (other instanceof VerificationKeyBytes) {
            final VerificationKeyBytes that = (VerificationKeyBytes) other;
            return Arrays.equals(that.vkb, this.vkb);
        } else {
            return false;
        }
    }

    @Override
    public int hashCode() {
        return 23 * Arrays.hashCode(this.vkb);
    }
}
