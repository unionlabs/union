/*
 * Copyright Supranational LLC
 * Licensed under the Apache License, Version 2.0, see LICENSE for details.
 * SPDX-License-Identifier: Apache-2.0
 */

package blst

import (
    "crypto/rand"
    "fmt"
    "runtime"
    "testing"
)

// Min PK
type PublicKeyMinPk = P1Affine
type SignatureMinPk = P2Affine
type AggregateSignatureMinPk = P2Aggregate
type AggregatePublicKeyMinPk = P1Aggregate

// Names in this file must be unique to support min-sig so we can't use 'dst'
// here.
var dstMinPk = []byte("BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_")

func init() {
    // Use all cores when testing and benchmarking
    SetMaxProcs(runtime.GOMAXPROCS(0))
}

func TestInfinityMinPk(t *testing.T) {
    var infComp [48]byte
    infComp[0] |= 0xc0
    new(PublicKeyMinPk).Uncompress(infComp[:])
}

func TestSerdesMinPk(t *testing.T) {
    var ikm = [...]byte{
        0x93, 0xad, 0x7e, 0x65, 0xde, 0xad, 0x05, 0x2a,
        0x08, 0x3a, 0x91, 0x0c, 0x8b, 0x72, 0x85, 0x91,
        0x46, 0x4c, 0xca, 0x56, 0x60, 0x5b, 0xb0, 0x56,
        0xed, 0xfe, 0x2b, 0x60, 0xa6, 0x3c, 0x48, 0x99}

    sk := KeyGen(ikm[:])
    defer sk.Zeroize()

    // Serialize/deserialize sk
    sk2 := new(SecretKey).Deserialize(sk.Serialize())
    defer sk2.Zeroize()
    if !sk.Equals(sk2) {
        t.Errorf("sk2 != sk")
    }

    // Negative test equals
    sk.b[0]++
    if sk.Equals(sk2) {
        t.Errorf("sk2 == sk")
    }

    // pk
    pk := new(PublicKeyMinPk).From(sk)

    // Compress/decompress sk
    pk2 := new(PublicKeyMinPk).Uncompress(pk.Compress())
    if !pk.Equals(pk2) {
        t.Errorf("pk2 != pk")
    }

    // Serialize/deserialize sk
    pk3 := new(PublicKeyMinPk).Deserialize(pk.Serialize())
    if !pk.Equals(pk3) {
        t.Errorf("pk3 != pk")
    }

    // Negative test equals
    // pk.x.l[0] = pk.x.l[0] + 1
    // if pk.Equals(pk2) {
    //  t.Errorf("pk2 == pk")
    // }
}

func TestSignVerifyMinPk(t *testing.T) {
    var ikm = [...]byte{
        0x93, 0xad, 0x7e, 0x65, 0xde, 0xad, 0x05, 0x2a,
        0x08, 0x3a, 0x91, 0x0c, 0x8b, 0x72, 0x85, 0x91,
        0x46, 0x4c, 0xca, 0x56, 0x60, 0x5b, 0xb0, 0x56,
        0xed, 0xfe, 0x2b, 0x60, 0xa6, 0x3c, 0x48, 0x99}

    sk0 := KeyGen(ikm[:])
    ikm[0]++
    sk1 := KeyGen(ikm[:])

    // pk
    pk0 := new(PublicKeyMinPk).From(sk0)
    pk1 := new(PublicKeyMinPk).From(sk1)

    // Sign
    msg0 := []byte("hello foo")
    msg1 := []byte("hello bar!")
    sig0 := new(SignatureMinPk).Sign(sk0, msg0, dstMinPk)
    sig1 := new(SignatureMinPk).Sign(sk1, msg1, dstMinPk)

    // Verify
    if !sig0.Verify(true, pk0, false, msg0, dstMinPk) {
        t.Errorf("verify sig0")
    }
    if !sig1.Verify(true, pk1, false, msg1, dstMinPk) {
        t.Errorf("verify sig1")
    }
    if !new(SignatureMinPk).VerifyCompressed(sig1.Compress(), true,
                                             pk1.Compress(), false,
                                             msg1, dstMinPk) {
        t.Errorf("verify sig1")
    }
    // Batch verify
    if !sig0.AggregateVerify(true, []*PublicKeyMinPk{pk0}, false,
                             []Message{msg0}, dstMinPk) {
        t.Errorf("aggregate verify sig0")
    }
    // Verify compressed inputs
    if !new(SignatureMinPk).AggregateVerifyCompressed(sig0.Compress(), true,
                                                      [][]byte{pk0.Compress()},
                                                      false,
                                                      []Message{msg0}, dstMinPk) {
        t.Errorf("aggregate verify sig0 compressed")
    }

    // Verify serialized inputs
    if !new(SignatureMinPk).AggregateVerifyCompressed(sig0.Compress(), true,
                                                      [][]byte{pk0.Serialize()},
                                                      false,
                                                      []Message{msg0}, dstMinPk) {
        t.Errorf("aggregate verify sig0 serialized")
    }

    // Compressed with empty pk
    var emptyPk []byte
    if new(SignatureMinPk).VerifyCompressed(sig0.Compress(), true,
                                            emptyPk, false, msg0, dstMinPk) {
        t.Errorf("verify sig compressed inputs")
    }
    // Wrong message
    if sig0.Verify(true, pk0, false, msg1, dstMinPk) {
        t.Errorf("Expected Verify to return false")
    }
    // Wrong key
    if sig0.Verify(true, pk1, false, msg0, dstMinPk) {
        t.Errorf("Expected Verify to return false")
    }
    // Wrong sig
    if sig1.Verify(true, pk0, false, msg0, dstMinPk) {
        t.Errorf("Expected Verify to return false")
    }
}

func TestSignVerifyAugMinPk(t *testing.T) {
    sk := genRandomKeyMinPk()
    pk := new(PublicKeyMinPk).From(sk)
    msg := []byte("hello foo")
    aug := []byte("augmentation")
    sig := new(SignatureMinPk).Sign(sk, msg, dstMinPk, aug)
    if !sig.Verify(true, pk, false, msg, dstMinPk, aug) {
        t.Errorf("verify sig")
    }
    aug2 := []byte("augmentation2")
    if sig.Verify(true, pk, false, msg, dstMinPk, aug2) {
        t.Errorf("verify sig, wrong augmentation")
    }
    if sig.Verify(true, pk, false, msg, dstMinPk) {
        t.Errorf("verify sig, no augmentation")
    }
    // TODO: augmentation with aggregate verify
}

func TestSignVerifyEncodeMinPk(t *testing.T) {
    sk := genRandomKeyMinPk()
    pk := new(PublicKeyMinPk).From(sk)
    msg := []byte("hello foo")
    sig := new(SignatureMinPk).Sign(sk, msg, dstMinPk, false)
    if !sig.Verify(true, pk, false, msg, dstMinPk, false) {
        t.Errorf("verify sig")
    }
    if sig.Verify(true, pk, false, msg, dstMinPk) {
        t.Errorf("verify sig expected fail, wrong hashing engine")
    }
    if sig.Verify(true, pk, false, msg, dstMinPk, 0) {
        t.Errorf("verify sig expected fail, illegal argument")
    }
}

func TestSignVerifyAggregateMinPk(t *testing.T) {
    for size := 1; size < 20; size++ {
        sks, msgs, _, pubks, _, err :=
            generateBatchTestDataUncompressedMinPk(size)
        if err {
            t.Errorf("Error generating test data")
            return
        }

        // All signers sign the same message
        sigs := make([]*SignatureMinPk, 0)
        for i := 0; i < size; i++ {
            sigs = append(sigs, new(SignatureMinPk).Sign(sks[i], msgs[0],
                dstMinPk))
        }
        agProj := new(AggregateSignatureMinPk)
        if !agProj.Aggregate(sigs, false) {
            t.Errorf("Aggregate unexpectedly returned nil")
            return
        }
        agSig := agProj.ToAffine()

        if !agSig.FastAggregateVerify(false, pubks, msgs[0], dstMinPk) {
            t.Errorf("failed to verify size %d", size)
        }

        // Negative test
        if agSig.FastAggregateVerify(false, pubks, msgs[0][1:], dstMinPk) {
            t.Errorf("failed to not verify size %d", size)
        }

        // Test compressed signature aggregation
        compSigs := make([][]byte, size)
        for i := 0; i < size; i++ {
            compSigs[i] = sigs[i].Compress()
        }
        agProj = new(AggregateSignatureMinPk)
        if !agProj.AggregateCompressed(compSigs, false) {
            t.Errorf("AggregateCompressed unexpectedly returned nil")
            return
        }
        agSig = agProj.ToAffine()
        if !agSig.FastAggregateVerify(false, pubks, msgs[0], dstMinPk) {
            t.Errorf("failed to verify size %d", size)
        }

        // Negative test
        if agSig.FastAggregateVerify(false, pubks, msgs[0][1:], dstMinPk) {
            t.Errorf("failed to not verify size %d", size)
        }
    }
}

func TestSignMultipleVerifyAggregateMinPk(t *testing.T) {
    msgCount := 5
    for size := 1; size < 20; size++ {
        msgs := make([]Message, 0)
        sks := make([]*SecretKey, 0)
        pks := make([]*PublicKeyMinPk, 0)

        // Generate messages
        for i := 0; i < msgCount; i++ {
            msg := Message(fmt.Sprintf("blst is a blast!! %d %d", i, size))
            msgs = append(msgs, msg)
        }

        // Generate keypairs
        for i := 0; i < size; i++ {
            priv := genRandomKeyMinPk()
            sks = append(sks, priv)
            pks = append(pks, new(PublicKeyMinPk).From(priv))
        }

        // All signers sign each message
        aggSigs := make([]*SignatureMinPk, 0)
        aggPks := make([]*PublicKeyMinPk, 0)
        for i := 0; i < msgCount; i++ {
            sigsToAgg := make([]*SignatureMinPk, 0)
            pksToAgg := make([]*PublicKeyMinPk, 0)
            for j := 0; j < size; j++ {
                sigsToAgg = append(sigsToAgg,
                                   new(SignatureMinPk).Sign(sks[j], msgs[i],
                                                            dstMinPk))
                pksToAgg = append(pksToAgg, pks[j])
            }

            agSig := new(AggregateSignatureMinPk)
            if !agSig.Aggregate(sigsToAgg, true) {
                t.Errorf("failed to aggregate")
            }
            afSig := agSig.ToAffine()
            agPk := new(AggregatePublicKeyMinPk)
            agPk.Aggregate(pksToAgg, false)
            afPk := agPk.ToAffine()
            aggSigs = append(aggSigs, afSig)
            aggPks = append(aggPks, afPk)

            // Verify aggregated signature and pk
            if !afSig.Verify(false, afPk, false, msgs[i], dstMinPk) {
                t.Errorf("failed to verify single aggregate size %d", size)
            }

        }

        randFn := func(s *Scalar) {
            var rbytes [BLST_SCALAR_BYTES]byte
            _, err := rand.Read(rbytes[:])
            if err != nil {
                t.Errorf(err.Error())
            }
            s.FromBEndian(rbytes[:])
        }

        // Verify
        randBits := 64
        if !new(SignatureMinPk).MultipleAggregateVerify(aggSigs, true,
                                                        aggPks, false,
                                                        msgs, dstMinPk,
                                                        randFn, randBits) {
            t.Errorf("failed to verify multiple aggregate size %d", size)
        }

        // Negative test
        if new(SignatureMinPk).MultipleAggregateVerify(aggSigs, true,
                                                       aggPks, false,
                                                       msgs, dstMinPk[1:],
                                                       randFn, randBits) {
            t.Errorf("failed to not verify multiple aggregate size %d", size)
        }
    }
}

func TestBatchUncompressMinPk(t *testing.T) {
    size := 128
    var points []*P2Affine
    var compPoints [][]byte

    for i := 0; i < size; i++ {
        msg := Message(fmt.Sprintf("blst is a blast!! %d", i))
        p2 := HashToG2(msg, dstMinPk).ToAffine()
        points = append(points, p2)
        compPoints = append(compPoints, p2.Compress())
    }
    uncompPoints := new(SignatureMinPk).BatchUncompress(compPoints)
    if uncompPoints == nil {
        t.Errorf("BatchUncompress returned nil size %d", size)
    }
    for i := 0; i < size; i++ {
        if !points[i].Equals(uncompPoints[i]) {
            t.Errorf("Uncompressed point does not equal initial point %d", i)
        }
    }
}

func BenchmarkCoreSignMinPk(b *testing.B) {
    var ikm = [...]byte{
        0x93, 0xad, 0x7e, 0x65, 0xde, 0xad, 0x05, 0x2a,
        0x08, 0x3a, 0x91, 0x0c, 0x8b, 0x72, 0x85, 0x91,
        0x46, 0x4c, 0xca, 0x56, 0x60, 0x5b, 0xb0, 0x56,
        0xed, 0xfe, 0x2b, 0x60, 0xa6, 0x3c, 0x48, 0x99}

    sk := KeyGen(ikm[:])
    defer sk.Zeroize()
    msg := []byte("hello foo")
    for i := 0; i < b.N; i++ {
        new(SignatureMinPk).Sign(sk, msg, dstMinPk)
    }
}

func BenchmarkCoreVerifyMinPk(b *testing.B) {
    var ikm = [...]byte{
        0x93, 0xad, 0x7e, 0x65, 0xde, 0xad, 0x05, 0x2a,
        0x08, 0x3a, 0x91, 0x0c, 0x8b, 0x72, 0x85, 0x91,
        0x46, 0x4c, 0xca, 0x56, 0x60, 0x5b, 0xb0, 0x56,
        0xed, 0xfe, 0x2b, 0x60, 0xa6, 0x3c, 0x48, 0x99}

    sk := KeyGen(ikm[:])
    defer sk.Zeroize()
    pk := new(PublicKeyMinPk).From(sk)
    msg := []byte("hello foo")
    sig := new(SignatureMinPk).Sign(sk, msg, dstMinPk)

    // Verify
    for i := 0; i < b.N; i++ {
        if !sig.Verify(true, pk, false, msg, dstMinPk) {
            b.Fatal("verify sig")
        }
    }
}

func BenchmarkCoreVerifyAggregateMinPk(b *testing.B) {
    run := func(size int) func(b *testing.B) {
        return func(b *testing.B) {
            msgs, _, pubks, agsig, err := generateBatchTestDataMinPk(size)
            if err {
                b.Fatal("Error generating test data")
            }
            b.ResetTimer()
            for i := 0; i < b.N; i++ {
                if !new(SignatureMinPk).AggregateVerifyCompressed(agsig, true,
                                                                  pubks, false,
                                                                  msgs, dstMinPk) {
                    b.Fatal("failed to verify")
                }
            }
        }
    }

    b.Run("1", run(1))
    b.Run("10", run(10))
    b.Run("50", run(50))
    b.Run("100", run(100))
    b.Run("300", run(300))
    b.Run("1000", run(1000))
    b.Run("4000", run(4000))
}

func BenchmarkVerifyAggregateUncompressedMinPk(b *testing.B) {
    run := func(size int) func(b *testing.B) {
        return func(b *testing.B) {
            _, msgs, _, pubks, agsig, err :=
                generateBatchTestDataUncompressedMinPk(size)
            if err {
                b.Fatal("Error generating test data")
            }
            b.ResetTimer()
            for i := 0; i < b.N; i++ {
                if !agsig.AggregateVerify(true, pubks, false, msgs, dstMinPk) {
                    b.Fatal("failed to verify")
                }
            }
        }
    }

    b.Run("1", run(1))
    b.Run("10", run(10))
    b.Run("50", run(50))
    b.Run("100", run(100))
    b.Run("300", run(300))
    b.Run("1000", run(1000))
    b.Run("4000", run(4000))
}

func BenchmarkCoreAggregateMinPk(b *testing.B) {
    run := func(size int) func(b *testing.B) {
        return func(b *testing.B) {
            _, sigs, _, _, err := generateBatchTestDataMinPk(size)
            if err {
                b.Fatal("Error generating test data")
            }
            b.ResetTimer()
            for i := 0; i < b.N; i++ {
                var agg AggregateSignatureMinPk
                agg.AggregateCompressed(sigs, true)
            }
        }
    }

    b.Run("1", run(1))
    b.Run("10", run(10))
    b.Run("50", run(50))
    b.Run("100", run(100))
    b.Run("300", run(300))
    b.Run("1000", run(1000))
    b.Run("4000", run(4000))
}

func genRandomKeyMinPk() *SecretKey {
    // Generate 32 bytes of randomness
    var ikm [32]byte
    _, err := rand.Read(ikm[:])

    if err != nil {
        return nil
    }
    return KeyGen(ikm[:])
}

func generateBatchTestDataMinPk(size int) (msgs []Message,
    sigs [][]byte, pubks [][]byte, agsig []byte, err bool) {
    err = false
    for i := 0; i < size; i++ {
        msg := Message(fmt.Sprintf("blst is a blast!! %d", i))
        msgs = append(msgs, msg)
        priv := genRandomKeyMinPk()
        sigs = append(sigs, new(SignatureMinPk).Sign(priv, msg, dstMinPk).
            Compress())
        pubks = append(pubks, new(PublicKeyMinPk).From(priv).Compress())
    }
    agProj := new(AggregateSignatureMinPk)
    if !agProj.AggregateCompressed(sigs, true) {
        fmt.Println("AggregateCompressed unexpectedly returned nil")
        err = true
        return
    }
    agAff := agProj.ToAffine()
    if agAff == nil {
        fmt.Println("ToAffine unexpectedly returned nil")
        err = true
        return
    }
    agsig = agAff.Compress()
    return
}

func generateBatchTestDataUncompressedMinPk(size int) (sks []*SecretKey,
    msgs []Message, sigs []*SignatureMinPk, pubks []*PublicKeyMinPk,
    agsig *SignatureMinPk, err bool) {
    err = false
    for i := 0; i < size; i++ {
        msg := Message(fmt.Sprintf("blst is a blast!! %d", i))
        msgs = append(msgs, msg)
        priv := genRandomKeyMinPk()
        sks = append(sks, priv)
        sigs = append(sigs, new(SignatureMinPk).Sign(priv, msg, dstMinPk))
        pubks = append(pubks, new(PublicKeyMinPk).From(priv))
    }
    agProj := new(AggregateSignatureMinPk)
    if !agProj.Aggregate(sigs, true) {
        fmt.Println("Aggregate unexpectedly returned nil")
        err = true
        return
    }
    agsig = agProj.ToAffine()
    return
}

func BenchmarkBatchUncompressMinPk(b *testing.B) {
    size := 128
    var compPoints [][]byte

    for i := 0; i < size; i++ {
        msg := Message(fmt.Sprintf("blst is a blast!! %d", i))
        p2 := HashToG2(msg, dstMinPk).ToAffine()
        compPoints = append(compPoints, p2.Compress())
    }
    b.Run("Single", func(b *testing.B) {
        b.ResetTimer()
        b.ReportAllocs()
        var tmp SignatureMinPk
        for i := 0; i < b.N; i++ {
            for j := 0; j < size; j++ {
                if tmp.Uncompress(compPoints[j]) == nil {
                    b.Fatal("could not uncompress point")
                }
            }
        }
    })
    b.Run("Batch", func(b *testing.B) {
        b.ResetTimer()
        b.ReportAllocs()
        var tmp SignatureMinPk
        for i := 0; i < b.N; i++ {
            if tmp.BatchUncompress(compPoints) == nil {
                b.Fatal("could not batch uncompress points")
            }
        }
    })
}

func TestSignVerifyAggregateValidatesInfinitePubkeyMinPk(t *testing.T) {
    size := 20
    sks, msgs, _, pubks, _, err :=
      generateBatchTestDataUncompressedMinPk(size)
    if err {
        t.Errorf("Error generating test data")
        return
    }

    // All signers sign the same message
    sigs := make([]*SignatureMinPk, 0)
    for i := 0; i < size; i++ {
        sigs = append(sigs, new(SignatureMinPk).Sign(sks[i], msgs[i],
          dstMinPk))
    }

    // Single message: Infinite pubkeys and signature
    zeroKey := new(PublicKeyMinPk)
    zeroSig := new(SignatureMinPk)
    agProj := new(AggregateSignatureMinPk)
    if !agProj.Aggregate([]*SignatureMinPk{zeroSig}, false) {
        t.Errorf("Aggregate unexpectedly returned nil")
        return
    }
    agSig := agProj.ToAffine()

    if agSig.AggregateVerify(false, []*PublicKeyMinPk{zeroKey}, false,
                             [][]byte{msgs[0]}, dstMinPk) {
        t.Errorf("failed to NOT verify signature")
    }

    // Replace firstkey with infinite pubkey.
    pubks[0] = zeroKey
    sigs[0] = zeroSig
    agProj = new(AggregateSignatureMinPk)
    if !agProj.Aggregate(sigs, false) {
        t.Errorf("Aggregate unexpectedly returned nil")
        return
    }
    agSig = agProj.ToAffine()

    if agSig.AggregateVerify(false, pubks, false, msgs, dstMinPk) {
        t.Errorf("failed to NOT verify signature")
    }
}

func TestEmptyMessageMinPk(t *testing.T) {
    msg := []byte("")
    var sk_bytes = []byte {99, 64, 58, 175, 15, 139, 113, 184, 37, 222, 127,
        204, 233, 209, 34, 8, 61, 27, 85, 251, 68, 31, 255, 214, 8, 189, 190, 71,
        198, 16, 210, 91};
    sk := new(SecretKey).Deserialize(sk_bytes)
    pk := new(PublicKeyMinPk).From(sk)
    sig := new(SignatureMinPk).Sign(sk, msg, dstMinPk)
    if !new(SignatureMinPk).VerifyCompressed(sig.Compress(), true,
        pk.Compress(), false, msg, dstMinPk) {
        t.Errorf("failed to verify empty message")
    }
}

func TestEmptySignatureMinPk(t *testing.T) {
    msg := []byte("message")
    var sk_bytes = []byte {99, 64, 58, 175, 15, 139, 113, 184, 37, 222, 127,
        204, 233, 209, 34, 8, 61, 27, 85, 251, 68, 31, 255, 214, 8, 189, 190, 71,
        198, 16, 210, 91};
    sk := new(SecretKey).Deserialize(sk_bytes)
    pk := new(PublicKeyMinPk).From(sk)
    var emptySig []byte
    if new(SignatureMinPk).VerifyCompressed(emptySig, true, pk.Compress(), false, msg, dstMinPk) {
        t.Errorf("failed to NOT verify empty signature")
    }
}

func TestMultiScalarP1(t *testing.T) {
    const npoints = 1027
    scalars := make([]byte, npoints*16)
    _, err := rand.Read(scalars)
    if err != nil {
        t.Errorf(err.Error())
	return
    }
    points := make([]P1, npoints)
    refs   := make([]P1, npoints)
    generator := P1Generator()
    for i := range points {
        points[i] = *generator.Mult(scalars[i*4:(i+1)*4])
        refs[i]   = *points[i].Mult(scalars[i*16:(i+1)*16], 128)
    }
    ref := P1s(refs).Add()
    ret := P1s(points).Mult(scalars, 128)
    if !ref.Equals(ret) {
        t.Errorf("failed self-consistency multi-scalar test")
    }
}

func BenchmarkMultiScalarP1(b *testing.B) {
    const npoints = 200000
    scalars := make([]byte, npoints*32)
    _, err := rand.Read(scalars)
    if err != nil {
        b.Fatal(err.Error())
    }
    temp := make([]P1, npoints)
    generator := P1Generator()
    for i := range temp {
        temp[i] = *generator.Mult(scalars[i*4:(i+1)*4])
    }
    points := P1s(temp).ToAffine()
    run := func(size int) func(b *testing.B) {
        return func(b *testing.B) {
            for i:=0; i<b.N; i++ {
                P1Affines(points[:size]).Mult(scalars, 255)
            }
        }
    }
    b.Run(fmt.Sprintf("%d",npoints/8), run(npoints/8))
    b.Run(fmt.Sprintf("%d",npoints/4), run(npoints/4))
    b.Run(fmt.Sprintf("%d",npoints/2), run(npoints/2))
    b.Run(fmt.Sprintf("%d",npoints), run(npoints))
}

func BenchmarkToP1Affines(b *testing.B) {
    const npoints = 32000
    scalars := make([]byte, npoints*32)
    _, err := rand.Read(scalars)
    if err != nil {
        b.Fatal(err.Error())
    }
    temp := make([]P1, npoints)
    generator := P1Generator()
    for i := range temp {
        temp[i] = *generator.Mult(scalars[i*4:(i+1)*4])
    }
    scratch := make([]P1Affine, npoints)
    run := func(size int) func(b *testing.B) {
        return func(b *testing.B) {
            b.ResetTimer()
            for i:=0; i<b.N; i++ {
                P1s(temp[:size]).ToAffine(scratch)
            }
        }
    }
    b.Run(fmt.Sprintf("%d",npoints/128), run(npoints/128))
    b.Run(fmt.Sprintf("%d",npoints/64), run(npoints/64))
    b.Run(fmt.Sprintf("%d",npoints/32), run(npoints/32))
    b.Run(fmt.Sprintf("%d",npoints/16), run(npoints/16))
    b.Run(fmt.Sprintf("%d",npoints/4), run(npoints/4))
    b.Run(fmt.Sprintf("%d",npoints), run(npoints))
}
