package ed25519consensus

import (
	"crypto/ed25519"
	"fmt"
	"testing"
)

func TestBatch(t *testing.T) {
	v := NewBatchVerifier()
	populateBatchVerifier(t, &v)

	if !v.Verify() {
		t.Error("failed batch verification")
	}
}

func TestBatchFailsOnShortSig(t *testing.T) {
	v := NewBatchVerifier()
	pub, _, _ := ed25519.GenerateKey(nil)
	v.Add(pub, []byte("message"), []byte{})
	if v.Verify() {
		t.Error("batch verification should fail due to short signature")
	}
}

func TestBatchFailsOnCorruptKey(t *testing.T) {
	v := NewBatchVerifier()
	populateBatchVerifier(t, &v)
	v.entries[1].pubkey[1] ^= 1
	if v.Verify() {
		t.Error("batch verification should fail due to corrupt key")
	}
}

func TestBatchFailsOnCorruptSignature(t *testing.T) {
	v := NewBatchVerifier()

	populateBatchVerifier(t, &v)
	// corrupt the R value of one of the signatures
	v.entries[4].signature[1] ^= 1
	if v.Verify() {
		t.Error("batch verification should fail due to corrupt signature")
	}

	populateBatchVerifier(t, &v)
	v.entries[1].digest[1] ^= 1
	if v.Verify() {
		t.Error("batch verification should fail due to corrupt signature")
	}
}

func TestEmptyBatchFails(t *testing.T) {
	v := NewBatchVerifier()

	if v.Verify() {
		t.Error("batch verification should fail on an empty batch")
	}
}

func BenchmarkBatch(b *testing.B) {
	for _, n := range []int{1, 8, 64, 1024, 4096, 16384} {
		b.Run(fmt.Sprint(n), func(b *testing.B) {
			var msg = []byte("ed25519consensus")
			b.ResetTimer()
			b.ReportAllocs()
			for i := 0; i < b.N; i++ {
				v := NewBatchVerifier()
				for j := 0; j < n; j++ {
					b.StopTimer()
					pub, priv, _ := ed25519.GenerateKey(nil)
					sig := ed25519.Sign(priv, msg)
					b.StartTimer()
					v.Add(pub, msg, sig)
				}
				if !v.Verify() {
					b.Fail()
				}
			}
			// Divide by n to get per-signature values.
			b.ReportMetric(float64(b.Elapsed().Nanoseconds())/float64(b.N)/float64(n), "ns/sig")
		})
	}
}

func BenchmarkPreallocatedBatch(b *testing.B) {
	for _, n := range []int{1, 8, 64, 1024, 4096, 16384} {
		b.Run(fmt.Sprint(n), func(b *testing.B) {
			var msg = []byte("ed25519consensus")
			b.ResetTimer()
			b.ReportAllocs()
			for i := 0; i < b.N; i++ {
				v := NewPreallocatedBatchVerifier(n)
				for j := 0; j < n; j++ {
					b.StopTimer()
					pub, priv, _ := ed25519.GenerateKey(nil)
					sig := ed25519.Sign(priv, msg)
					b.StartTimer()
					v.Add(pub, msg, sig)
				}
				if !v.Verify() {
					b.Fail()
				}
			}
			// Divide by n to get per-signature values.
			b.ReportMetric(float64(b.Elapsed().Nanoseconds())/float64(b.N)/float64(n), "ns/sig")
		})
	}
}

// populateBatchVerifier populates a verifier with multiple entries
func populateBatchVerifier(t *testing.T, v *BatchVerifier) {
	*v = NewBatchVerifier()
	for i := 0; i <= 38; i++ {

		pub, priv, _ := ed25519.GenerateKey(nil)

		var msg []byte
		if i%2 == 0 {
			msg = []byte("easter")
		} else {
			msg = []byte("egg")
		}

		sig := ed25519.Sign(priv, msg)

		v.Add(pub, msg, sig)
	}
}
