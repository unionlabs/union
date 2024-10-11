package compress

import (
	"crypto/rand"
	"encoding/base32"
	"testing"
)

func BenchmarkEstimate(b *testing.B) {
	b.ReportAllocs()
	// (predictable, low entropy distribution)
	b.Run("zeroes-5k", func(b *testing.B) {
		var testData = make([]byte, 5000)
		b.SetBytes(int64(len(testData)))
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			Estimate(testData)
		}
		b.Log(Estimate(testData))
	})

	// (predictable, high entropy distribution)
	b.Run("predictable-5k", func(b *testing.B) {
		var testData = make([]byte, 5000)
		for i := range testData {
			testData[i] = byte(float64(i) / float64(len(testData)) * 256)
		}
		b.SetBytes(int64(len(testData)))
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			Estimate(testData)
		}
		b.Log(Estimate(testData))
	})

	// (not predictable, high entropy distribution)
	b.Run("random-500b", func(b *testing.B) {
		var testData = make([]byte, 500)
		rand.Read(testData)
		b.SetBytes(int64(len(testData)))
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			Estimate(testData)
		}
		b.Log(Estimate(testData))
	})

	// (not predictable, high entropy distribution)
	b.Run("random-5k", func(b *testing.B) {
		var testData = make([]byte, 5000)
		rand.Read(testData)
		b.SetBytes(int64(len(testData)))
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			Estimate(testData)
		}
		b.Log(Estimate(testData))
	})

	// (not predictable, high entropy distribution)
	b.Run("random-50k", func(b *testing.B) {
		var testData = make([]byte, 50000)
		rand.Read(testData)
		b.SetBytes(int64(len(testData)))
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			Estimate(testData)
		}
		b.Log(Estimate(testData))
	})

	// (not predictable, high entropy distribution)
	b.Run("random-500k", func(b *testing.B) {
		var testData = make([]byte, 500000)
		rand.Read(testData)
		b.SetBytes(int64(len(testData)))
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			Estimate(testData)
		}
		b.Log(Estimate(testData))
	})

	// (not predictable, medium entropy distribution)
	b.Run("base-32-5k", func(b *testing.B) {
		var testData = make([]byte, 5000)
		rand.Read(testData)
		s := base32.StdEncoding.EncodeToString(testData)
		testData = []byte(s)
		testData = testData[:5000]
		b.SetBytes(int64(len(testData)))
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			Estimate(testData)
		}
		b.Log(Estimate(testData))
	})
	// (medium predictable, medium entropy distribution)
	b.Run("text", func(b *testing.B) {
		var testData = []byte(`If compression is done per-chunk, care should be taken that it doesn't leave restic backups open to watermarking/fingerprinting attacks.
This is essentially the same problem we discussed related to fingerprinting the CDC deduplication process:
With "naive" CDC, a "known plaintext" file can be verified to exist within the backup if the size of individual blocks can be observed by an attacker, by using CDC on the file in parallel and comparing the resulting amount of chunks and individual chunk lengths.
As discussed earlier, this can be somewhat mitigated by salting the CDC algorithm with a secret value, as done in attic.
With salted CDC, I assume compression would happen on each individual chunk, after splitting the problematic file into chunks. Restic chunks are in the range of 512 KB to 8 MB (but not evenly distributed - right?).
Attacker knows that the CDC algorithm uses a secret salt, so the attacker generates a range of chunks consisting of the first 512 KB to 8 MB of the file, one for each valid chunk length. The attacker is also able to determine the lengths of compressed chunks.
The attacker then compresses that chunk using the compression algorithm.
The attacker compares the lengths of the resulting chunks to the first chunk in the restic backup sets.
IF a matching block length is found, the attacker repeats the exercise with the next chunk, and the next chunk, and the next chunk, ... and the next chunk.
It is my belief that with sufficiently large files, and considering the fact that the CDC algorithm is "biased" (in lack of better of words) towards generating blocks of about 1 MB, this would be sufficient to ascertain whether or not a certain large file exists in the backup.
AS always, a paranoid and highly unscientific stream of consciousness.
Thoughts?`)
		testData = append(testData, testData...)
		testData = append(testData, testData...)
		b.SetBytes(int64(len(testData)))
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			Estimate(testData)
		}
		b.Log(Estimate(testData))
	})
}

func BenchmarkSnannonEntropyBits(b *testing.B) {
	b.ReportAllocs()
	// (predictable, low entropy distribution)
	b.Run("zeroes-5k", func(b *testing.B) {
		var testData = make([]byte, 5000)
		b.SetBytes(int64(len(testData)))
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			ShannonEntropyBits(testData)
		}
		b.Log(ShannonEntropyBits(testData))
	})

	// (predictable, high entropy distribution)
	b.Run("predictable-5k", func(b *testing.B) {
		var testData = make([]byte, 5000)
		for i := range testData {
			testData[i] = byte(float64(i) / float64(len(testData)) * 256)
		}
		b.SetBytes(int64(len(testData)))
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			ShannonEntropyBits(testData)
		}
		b.Log(ShannonEntropyBits(testData))
	})

	// (not predictable, high entropy distribution)
	b.Run("random-500b", func(b *testing.B) {
		var testData = make([]byte, 500)
		rand.Read(testData)
		b.SetBytes(int64(len(testData)))
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			ShannonEntropyBits(testData)
		}
		b.Log(ShannonEntropyBits(testData))
	})

	// (not predictable, high entropy distribution)
	b.Run("random-5k", func(b *testing.B) {
		var testData = make([]byte, 5000)
		rand.Read(testData)
		b.SetBytes(int64(len(testData)))
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			ShannonEntropyBits(testData)
		}
		b.Log(ShannonEntropyBits(testData))
	})

	// (not predictable, high entropy distribution)
	b.Run("random-50k", func(b *testing.B) {
		var testData = make([]byte, 50000)
		rand.Read(testData)
		b.SetBytes(int64(len(testData)))
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			ShannonEntropyBits(testData)
		}
		b.Log(ShannonEntropyBits(testData))
	})

	// (not predictable, high entropy distribution)
	b.Run("random-500k", func(b *testing.B) {
		var testData = make([]byte, 500000)
		rand.Read(testData)
		b.SetBytes(int64(len(testData)))
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			ShannonEntropyBits(testData)
		}
		b.Log(ShannonEntropyBits(testData))
	})

	// (not predictable, medium entropy distribution)
	b.Run("base-32-5k", func(b *testing.B) {
		var testData = make([]byte, 5000)
		rand.Read(testData)
		s := base32.StdEncoding.EncodeToString(testData)
		testData = []byte(s)
		testData = testData[:5000]
		b.SetBytes(int64(len(testData)))
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			ShannonEntropyBits(testData)
		}
		b.Log(ShannonEntropyBits(testData))
	})
	// (medium predictable, medium entropy distribution)
	b.Run("text", func(b *testing.B) {
		var testData = []byte(`If compression is done per-chunk, care should be taken that it doesn't leave restic backups open to watermarking/fingerprinting attacks.
This is essentially the same problem we discussed related to fingerprinting the CDC deduplication process:
With "naive" CDC, a "known plaintext" file can be verified to exist within the backup if the size of individual blocks can be observed by an attacker, by using CDC on the file in parallel and comparing the resulting amount of chunks and individual chunk lengths.
As discussed earlier, this can be somewhat mitigated by salting the CDC algorithm with a secret value, as done in attic.
With salted CDC, I assume compression would happen on each individual chunk, after splitting the problematic file into chunks. Restic chunks are in the range of 512 KB to 8 MB (but not evenly distributed - right?).
Attacker knows that the CDC algorithm uses a secret salt, so the attacker generates a range of chunks consisting of the first 512 KB to 8 MB of the file, one for each valid chunk length. The attacker is also able to determine the lengths of compressed chunks.
The attacker then compresses that chunk using the compression algorithm.
The attacker compares the lengths of the resulting chunks to the first chunk in the restic backup sets.
IF a matching block length is found, the attacker repeats the exercise with the next chunk, and the next chunk, and the next chunk, ... and the next chunk.
It is my belief that with sufficiently large files, and considering the fact that the CDC algorithm is "biased" (in lack of better of words) towards generating blocks of about 1 MB, this would be sufficient to ascertain whether or not a certain large file exists in the backup.
AS always, a paranoid and highly unscientific stream of consciousness.
Thoughts?`)
		testData = append(testData, testData...)
		testData = append(testData, testData...)
		b.SetBytes(int64(len(testData)))
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			ShannonEntropyBits(testData)
		}
		b.Log(ShannonEntropyBits(testData))
	})
}
