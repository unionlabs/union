package org.zfnd.ed25519

import java.math.BigInteger
import java.security.SecureRandom
import org.scalatest.{ FlatSpec, MustMatchers }

class Ed25519InterfaceTest extends FlatSpec with MustMatchers {
  private val RANDOM = new SecureRandom

  private def convertBytesToHex(bytes: Seq[Byte]): String = {
    val sb = new StringBuilder
    for (b <- bytes) {
      sb.append(String.format("%02x", Byte.box(b)))
    }
    sb.toString
  }

  it must "initialize the Ed25519 interface" in {
    Ed25519Interface.isEnabled mustBe true
  }

  it must "get a private key" in {
    val sks = Ed25519Interface.genSigningKeySeed(RANDOM)
    val sksValue = BigInt(convertBytesToHex(sks.getSigningKeySeed), 16)
    sksValue must not be BigInteger.ZERO
  }

  it must "sign and verify data" in {
    val sks = Ed25519Interface.genSigningKeySeed(RANDOM)
    val vkb = Ed25519Interface.getVerificationKeyBytes(sks)

    val m = new Array[Byte](32)
    RANDOM.nextBytes(m)
    val rustSig = Ed25519Interface.sign(sks, m)
    Ed25519Interface.verify(vkb, rustSig, m) mustBe (true)
  }

  it must "reject bad signing key seeds" in {
    val m = new Array[Byte](32) // 0x0000....
    val sks = SigningKeySeed.fromBytes(m)
    sks.isPresent mustBe false
  }

  it must "reject bad verification key bytes" in {
    val vkbValue = BigInt("9000000000000000000000000000000000000000000000000000000000000000", 16)
    var vkb = VerificationKeyBytes.fromBytes(vkbValue.toByteArray)
    vkb.isPresent mustBe false
  }

  // Included to deterministically confirm that JNI usage still leads to correct
  // results. See Sect. 7.1 of RFC 8032.
  it must "match RFC 8032 test vector data" in {
    val sksValue = BigInt("4ccd089b28ff96da9db6c346ec114e0f5b8a319f35aba624da8cf6ed4fb8a6fb", 16)
    val sks = new SigningKeySeed(sksValue.toByteArray)
    val vkb = Ed25519Interface.getVerificationKeyBytes(sks)
    convertBytesToHex(vkb.getVerificationKeyBytes) mustBe("3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c")

    val msg: Array[Byte] = Array(114.toByte) // 0x72
    val sig = Ed25519Interface.sign(sks, msg)
    convertBytesToHex(sig.getSignatureBytes) mustBe("92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00")

    // fromBytesOrThrow() sanity checks.
    val sks2 = SigningKeySeed.fromBytesOrThrow(sks.getSigningKeySeed)
    convertBytesToHex(sks2.getSigningKeySeed) mustBe("4ccd089b28ff96da9db6c346ec114e0f5b8a319f35aba624da8cf6ed4fb8a6fb")
    val vkb2 = VerificationKeyBytes.fromBytesOrThrow(vkb.getVerificationKeyBytes)
    convertBytesToHex(vkb2.getVerificationKeyBytes) mustBe("3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c")
  }
}
