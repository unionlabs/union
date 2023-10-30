package org.zfnd.ed25519;

import java.util.Arrays;
import java.util.Optional;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Java wrapper class for signatures that performs some sanity checking.
 */
public class Signature {
    public static final int COMPONENT_LENGTH = 32;
    public static final int SIGNATURE_LENGTH = 2 * COMPONENT_LENGTH;
    private static final Logger logger = LoggerFactory.getLogger(Signature.class);

    private byte[] rBytes;
    private byte[] sBytes;
    private byte[] completeSignature;

    // Don't bother with an expensive, literal check. Just ensure the format's correct.
    static boolean bytesAreValid(final byte[] signature) {
        return (signature.length == (SIGNATURE_LENGTH));
    }

    Signature(final byte[] sig) {
        // package protected constructor
        // assumes valid values from us or underlying library and that the caller will not mutate them
        rBytes = Arrays.copyOfRange(sig, 0, COMPONENT_LENGTH);
        sBytes = Arrays.copyOfRange(sig, COMPONENT_LENGTH, SIGNATURE_LENGTH);

        // Cache the complete signature array instead of rebuilding when requested.
        completeSignature = new byte[SIGNATURE_LENGTH];
        System.arraycopy(rBytes, 0, completeSignature, 0, COMPONENT_LENGTH);
        System.arraycopy(sBytes, 0, completeSignature, COMPONENT_LENGTH, COMPONENT_LENGTH);
    }

    /**
     * @return a copy of the complete signature
     */
    public byte[] getSignatureBytesCopy() {
        return completeSignature.clone();
    }

    byte[] getSignatureBytes() {
        return completeSignature;
    }

    /**
     * Optionally convert bytes into a verification key wrapper.
     *
     * @param bytes untrusted, unvalidated bytes that may be an encoding of a verification key
     * @return optionally a verification key wrapper, if bytes are valid
     */
    public static Optional<Signature> fromBytes(final byte[] bytes) {
        if (bytesAreValid(bytes)) {
            return Optional.of(new Signature(bytes));
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
    public static Signature fromBytesOrThrow(final byte[] bytes) {
        return fromBytes(bytes)
            .orElseThrow(() -> new IllegalArgumentException("Expected " + (SIGNATURE_LENGTH) + " bytes that encode a signature!"));
    }

    @Override
    public boolean equals(final Object other) {
        if (other == this) {
            return true;
        } else if (other instanceof Signature) {
            final Signature that = (Signature) other;
            return Arrays.equals(that.rBytes, this.rBytes) &&
                Arrays.equals(that.sBytes, this.sBytes);
        } else {
            return false;
        }
    }

    @Override
    public int hashCode() {
        int h = 23 * Arrays.hashCode(rBytes);
        h = 23 * (h + Arrays.hashCode(sBytes));
        return h;
    }
}
