package org.zfnd.ed25519

import java.security.SecureRandom
import org.scalatest.{ FlatSpec, MustMatchers }
import scala.collection.mutable.HashSet

class SignatureTest extends FlatSpec with MustMatchers {
    private val RANDOM = new SecureRandom()

    it must "properly compare Signature objects" in {
        val sig1 = new Array[Byte](Signature.SIGNATURE_LENGTH);
        do {
            RANDOM.nextBytes(sig1);
        } while(!Signature.bytesAreValid(sig1));

        val sig2 = new Array[Byte](Signature.SIGNATURE_LENGTH);
        do {
            RANDOM.nextBytes(sig2);
        } while(!Signature.bytesAreValid(sig2));

        val sigObj1 = Signature.fromBytesOrThrow(sig1);
        val sigObj2 = Signature.fromBytesOrThrow(sig1);
        val sigObj3 = Signature.fromBytesOrThrow(sig2);
        sigObj1 == sigObj2 mustBe true
        sigObj2 == sigObj3 mustBe false
    }

    it must "reject illegal Signature bytes" in {
        val sig = new Array[Byte](Signature.COMPONENT_LENGTH);
        RANDOM.nextBytes(sig);

        val sigObj = Signature.fromBytes(sig)
        sigObj.isPresent() mustBe false
    }

    it must "properly handle Signatures in hashed data structures" in {
        val sig = new Array[Byte](Signature.SIGNATURE_LENGTH);
        do {
            RANDOM.nextBytes(sig);
        } while(!Signature.bytesAreValid(sig));

        val sigObj1 = Signature.fromBytesOrThrow(sig);
        val sigObj2 = Signature.fromBytesOrThrow(sig);

        val sigSet: HashSet[Signature] = HashSet(sigObj1, sigObj2);
        sigSet.size must be(1);
        sigSet.contains(Signature.fromBytesOrThrow(sig)) mustBe true
    }
}
