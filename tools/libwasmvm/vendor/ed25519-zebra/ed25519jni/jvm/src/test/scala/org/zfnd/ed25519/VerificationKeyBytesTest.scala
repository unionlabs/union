package org.zfnd.ed25519

import java.security.SecureRandom
import org.scalatest.{ FlatSpec, MustMatchers }
import scala.collection.mutable.HashSet

class VerificationKeyBytesTest extends FlatSpec with MustMatchers {
  private val RANDOM = new SecureRandom()

  it must "properly compare VerificationKeyBytes objects" in {
    val vkb1 = new Array[Byte](VerificationKeyBytes.BYTE_LENGTH)
    do {
      RANDOM.nextBytes(vkb1)
    } while(!VerificationKeyBytes.bytesAreValid(vkb1))

    val vkb2 = new Array[Byte](VerificationKeyBytes.BYTE_LENGTH)
    do {
      RANDOM.nextBytes(vkb2)
    } while(!VerificationKeyBytes.bytesAreValid(vkb2))

    val vkbObj1 = new VerificationKeyBytes(vkb1)
    val vkbObj2 = new VerificationKeyBytes(vkb1)
    val vkbObj3 = new VerificationKeyBytes(vkb2)
    vkbObj1 == vkbObj2 mustBe true
    vkbObj2 == vkbObj3 mustBe false
  }

  it must "properly handle VerificationKeyBytes in hashed data structures" in {
    val vkb = new Array[Byte](VerificationKeyBytes.BYTE_LENGTH)
    do {
      RANDOM.nextBytes(vkb)
    } while(!VerificationKeyBytes.bytesAreValid(vkb))

    val vkbObj1 = new VerificationKeyBytes(vkb)
    val vkbObj2 = new VerificationKeyBytes(vkb)

    val vkbSet: HashSet[VerificationKeyBytes] = HashSet(vkbObj1, vkbObj2)
    vkbSet.size must be(1)
    vkbSet.contains(new VerificationKeyBytes(vkb)) mustBe true
  }

  it must "reject bad VerificationKeyBytes creation attempts via fromBytes()" in {
    val vkb1 = new Array[Byte](2 * VerificationKeyBytes.BYTE_LENGTH)
    RANDOM.nextBytes(vkb1)
    val vkbObj1 = VerificationKeyBytes.fromBytes(vkb1)
    vkbObj1.isPresent() mustBe false
  }
}
