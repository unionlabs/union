package xerial

import (
	"bytes"
	"math/rand"
	"testing"

	"github.com/klauspost/compress/s2"
)

const largeString = `Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt explicabo. Nemo enim ipsam voluptatem quia voluptas sit aspernatur aut odit aut fugit, sed quia consequuntur magni dolores eos qui ratione voluptatem sequi nesciunt. Neque porro quisquam est, qui dolorem ipsum quia dolor sit amet, consectetur, adipisci velit, sed quia non numquam eius modi tempora incidunt ut labore et dolore magnam aliquam quaerat voluptatem. Ut enim ad minima veniam, quis nostrum exercitationem ullam corporis suscipit laboriosam, nisi ut aliquid ex ea commodi consequatur? Quis autem vel eum iure reprehenderit qui in ea voluptate velit esse quam nihil molestiae consequatur, vel illum qui dolorem eum fugiat quo voluptas nulla pariatur? At vero eos et accusamus et iusto odio dignissimos ducimus qui blanditiis praesentium voluptatum deleniti atque corrupti quos dolores et quas molestias except`

var snappyStreamTestCases = map[string][]byte{
	"PLAINDATA":                         {130, 83, 78, 65, 80, 80, 89, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 11, 9, 32, 80, 76, 65, 73, 78, 68, 65, 84, 65},
	`{"a":"UtaitILHMDAAAAfU","b":"日本"}`: {130, 83, 78, 65, 80, 80, 89, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 39, 37, 144, 123, 34, 97, 34, 58, 34, 85, 116, 97, 105, 116, 73, 76, 72, 77, 68, 65, 65, 65, 65, 102, 85, 34, 44, 34, 98, 34, 58, 34, 230, 151, 165, 230, 156, 172, 34, 125},
	largeString:                         {130, 83, 78, 65, 80, 80, 89, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 3, 89, 128, 8, 240, 90, 83, 101, 100, 32, 117, 116, 32, 112, 101, 114, 115, 112, 105, 99, 105, 97, 116, 105, 115, 32, 117, 110, 100, 101, 32, 111, 109, 110, 105, 115, 32, 105, 115, 116, 101, 32, 110, 97, 116, 117, 115, 32, 101, 114, 114, 111, 114, 32, 115, 105, 116, 32, 118, 111, 108, 117, 112, 116, 97, 116, 101, 109, 32, 97, 99, 99, 117, 115, 97, 110, 116, 105, 117, 109, 32, 100, 111, 108, 111, 114, 101, 109, 113, 117, 101, 32, 108, 97, 117, 100, 97, 5, 22, 240, 60, 44, 32, 116, 111, 116, 97, 109, 32, 114, 101, 109, 32, 97, 112, 101, 114, 105, 97, 109, 44, 32, 101, 97, 113, 117, 101, 32, 105, 112, 115, 97, 32, 113, 117, 97, 101, 32, 97, 98, 32, 105, 108, 108, 111, 32, 105, 110, 118, 101, 110, 116, 111, 114, 101, 32, 118, 101, 114, 105, 116, 97, 1, 141, 4, 101, 116, 1, 36, 88, 115, 105, 32, 97, 114, 99, 104, 105, 116, 101, 99, 116, 111, 32, 98, 101, 97, 116, 97, 101, 32, 118, 105, 1, 6, 120, 100, 105, 99, 116, 97, 32, 115, 117, 110, 116, 32, 101, 120, 112, 108, 105, 99, 97, 98, 111, 46, 32, 78, 101, 109, 111, 32, 101, 110, 105, 109, 5, 103, 0, 109, 46, 180, 0, 12, 113, 117, 105, 97, 17, 16, 0, 115, 5, 209, 72, 97, 115, 112, 101, 114, 110, 97, 116, 117, 114, 32, 97, 117, 116, 32, 111, 100, 105, 116, 5, 9, 36, 102, 117, 103, 105, 116, 44, 32, 115, 101, 100, 9, 53, 32, 99, 111, 110, 115, 101, 113, 117, 117, 110, 1, 42, 20, 109, 97, 103, 110, 105, 32, 9, 245, 16, 115, 32, 101, 111, 115, 1, 36, 28, 32, 114, 97, 116, 105, 111, 110, 101, 17, 96, 33, 36, 1, 51, 36, 105, 32, 110, 101, 115, 99, 105, 117, 110, 116, 1, 155, 1, 254, 16, 112, 111, 114, 114, 111, 1, 51, 36, 115, 113, 117, 97, 109, 32, 101, 115, 116, 44, 1, 14, 13, 81, 5, 183, 4, 117, 109, 1, 18, 0, 97, 9, 19, 4, 32, 115, 1, 149, 12, 109, 101, 116, 44, 9, 135, 76, 99, 116, 101, 116, 117, 114, 44, 32, 97, 100, 105, 112, 105, 115, 99, 105, 32, 118, 101, 108, 50, 173, 0, 24, 110, 111, 110, 32, 110, 117, 109, 9, 94, 84, 105, 117, 115, 32, 109, 111, 100, 105, 32, 116, 101, 109, 112, 111, 114, 97, 32, 105, 110, 99, 105, 100, 33, 52, 20, 117, 116, 32, 108, 97, 98, 33, 116, 4, 101, 116, 9, 106, 0, 101, 5, 219, 20, 97, 109, 32, 97, 108, 105, 5, 62, 33, 164, 8, 114, 97, 116, 29, 212, 12, 46, 32, 85, 116, 41, 94, 52, 97, 100, 32, 109, 105, 110, 105, 109, 97, 32, 118, 101, 110, 105, 33, 221, 72, 113, 117, 105, 115, 32, 110, 111, 115, 116, 114, 117, 109, 32, 101, 120, 101, 114, 99, 105, 33, 202, 104, 111, 110, 101, 109, 32, 117, 108, 108, 97, 109, 32, 99, 111, 114, 112, 111, 114, 105, 115, 32, 115, 117, 115, 99, 105, 112, 105, 13, 130, 8, 105, 111, 115, 1, 64, 12, 110, 105, 115, 105, 1, 150, 5, 126, 44, 105, 100, 32, 101, 120, 32, 101, 97, 32, 99, 111, 109, 5, 192, 0, 99, 41, 131, 33, 172, 8, 63, 32, 81, 1, 107, 4, 97, 117, 33, 101, 96, 118, 101, 108, 32, 101, 117, 109, 32, 105, 117, 114, 101, 32, 114, 101, 112, 114, 101, 104, 101, 110, 100, 101, 114, 105, 65, 63, 12, 105, 32, 105, 110, 1, 69, 16, 118, 111, 108, 117, 112, 65, 185, 1, 47, 24, 105, 116, 32, 101, 115, 115, 101, 1, 222, 64, 109, 32, 110, 105, 104, 105, 108, 32, 109, 111, 108, 101, 115, 116, 105, 97, 101, 46, 103, 0, 0, 44, 1, 45, 16, 32, 105, 108, 108, 117, 37, 143, 45, 36, 0, 109, 5, 110, 65, 33, 20, 97, 116, 32, 113, 117, 111, 17, 92, 44, 115, 32, 110, 117, 108, 108, 97, 32, 112, 97, 114, 105, 9, 165, 24, 65, 116, 32, 118, 101, 114, 111, 69, 34, 44, 101, 116, 32, 97, 99, 99, 117, 115, 97, 109, 117, 115, 1, 13, 104, 105, 117, 115, 116, 111, 32, 111, 100, 105, 111, 32, 100, 105, 103, 110, 105, 115, 115, 105, 109, 111, 115, 32, 100, 117, 99, 105, 1, 34, 80, 113, 117, 105, 32, 98, 108, 97, 110, 100, 105, 116, 105, 105, 115, 32, 112, 114, 97, 101, 115, 101, 101, 87, 17, 111, 56, 116, 117, 109, 32, 100, 101, 108, 101, 110, 105, 116, 105, 32, 97, 116, 65, 89, 28, 99, 111, 114, 114, 117, 112, 116, 105, 1, 150, 0, 115, 13, 174, 5, 109, 8, 113, 117, 97, 65, 5, 52, 108, 101, 115, 116, 105, 97, 115, 32, 101, 120, 99, 101, 112, 116, 0, 0, 0, 1, 0},
}

func makeMassive(input string, numCopies int) string {
	outBuff := make([]byte, len(input)*numCopies)

	for i := 0; i < numCopies; i++ {
		copy(outBuff[len(outBuff):], input)
	}

	return string(outBuff)
}

func TestSnappyEncodeStream(t *testing.T) {
	for src := range snappyStreamTestCases {
		dst := Encode(nil, []byte(src))

		// Block size can change the bytes generated, so let's just decode and make sure in matches out
		dec, err := Decode(dst)
		if err != nil {
			t.Error(err)
		}
		if src != string(dec) {
			t.Errorf("Expected decode to match encode orig = %s, decoded = %s", src, string(dec))
		}
	}
}

func TestSnappyLargeStringEncodeStream(t *testing.T) {
	massiveString := makeMassive(largeString, 10000)
	dst := Encode(nil, []byte(massiveString))
	dec, err := Decode(dst)
	if err != nil {
		t.Error(err)
	}
	if massiveString != string(dec) {
		t.Errorf("Decoded string didn't match original input (not printing due to size)")
	}
}

func TestSnappyDecodeStreams(t *testing.T) {
	for exp, src := range snappyStreamTestCases {
		dst, err := Decode(src)
		if err != nil {
			t.Error("Encoding error: ", err)
		} else if !bytes.Equal(dst, []byte(exp)) {
			t.Errorf("Expected %s to be generated from [%d]byte, but was %s", exp, len(src), string(dst))
		}
	}
}

func TestSnappyDecodeMalformedTruncatedHeader(t *testing.T) {
	// Truncated headers should not cause a panic.
	for i := 0; i < len(xerialHeader); i++ {
		buf := make([]byte, i)
		copy(buf, xerialHeader[:i])
		if _, err := Decode(buf); err != ErrMalformed {
			t.Errorf("expected ErrMalformed got %v", err)
		}
	}
}

func TestSnappyDecodeMalformedTruncatedSize(t *testing.T) {
	// Inputs with valid Xerial header but truncated "size" field
	sizes := []int{sizeOffset + 1, sizeOffset + 2, sizeOffset + 3}
	for _, size := range sizes {
		buf := make([]byte, size)
		copy(buf, xerialHeader)
		if _, err := Decode(buf); err != ErrMalformed {
			t.Errorf("expected ErrMalformed got %v", err)
		}
	}
}

func TestSnappyDecodeMalformedBNoData(t *testing.T) {
	// No data after the size field
	buf := make([]byte, 20)
	copy(buf, xerialHeader)
	// indicate that there's one byte of data to be read
	buf[len(buf)-1] = 1
	if _, err := Decode(buf); err != ErrMalformed {
		t.Errorf("expected ErrMalformed got %v", err)
	}
}

func TestSnappyMasterDecodeFailed(t *testing.T) {
	buf := make([]byte, 21)
	copy(buf, xerialHeader)
	// indicate that there's one byte of data to be read
	buf[len(buf)-2] = 1
	// A payload which will not decode
	buf[len(buf)-1] = 1
	if _, err := Decode(buf); err == ErrMalformed || err == nil {
		t.Errorf("unexpected err: %v", err)
	}
}

func BenchmarkSnappyStreamDecode(b *testing.B) {
	b.ReportAllocs()
	b.ResetTimer()

	for n := 0; n < b.N; n++ {
		bytes := 0
		for _, test := range snappyStreamTestCases {
			dst, err := Decode(test)
			if err != nil {
				b.Error("Decoding error: ", err)
			}
			bytes += len(dst)
		}
		b.SetBytes(int64(bytes))
	}
}

func BenchmarkSnappyStreamDecodeInto(b *testing.B) {
	b.ReportAllocs()
	b.ResetTimer()

	var (
		dst = make([]byte, 1024, 1024)
		err error
	)

	for n := 0; n < b.N; n++ {
		bytes := 0
		for _, test := range snappyStreamTestCases {
			dst, err = DecodeInto(dst, test)
			if err != nil {
				b.Error("Decoding error: ", err)
			}
			bytes += len(dst)
		}
		b.SetBytes(int64(bytes))
	}
}
func BenchmarkSnappyStreamDecodeMassive(b *testing.B) {
	massiveString := makeMassive(largeString, 10000)
	enc := Encode(nil, []byte(massiveString))

	b.ReportAllocs()
	b.ResetTimer()
	b.SetBytes(int64(len(massiveString)))

	for n := 0; n < b.N; n++ {
		_, err := Decode(enc)
		if err != nil {
			b.Error("Decoding error: ", err)
		}
	}
}

func BenchmarkSnappyStreamDecodeIntoMassive(b *testing.B) {
	massiveString := makeMassive(largeString, 10000)
	enc := Encode(nil, []byte(massiveString))

	var (
		dst = make([]byte, 1024, len(massiveString))
		err error
	)

	b.ReportAllocs()
	b.ResetTimer()
	b.SetBytes(int64(len(massiveString)))

	for n := 0; n < b.N; n++ {
		dst, err = DecodeInto(dst, enc)
		if err != nil {
			b.Error("Decoding error: ", err)
		}
	}
}

func BenchmarkSnappyStreamEncode(b *testing.B) {
	test := []byte(largeString)

	var (
		dst = make([]byte, 0, 20+s2.MaxEncodedLen(len(test)))
		err error
	)

	b.ReportAllocs()
	b.ResetTimer()
	b.SetBytes(int64(len(test)))

	for n := 0; n < b.N; n++ {
		dst = Encode(dst[:0], test)
		if err != nil {
			b.Error("Encoding error: ", err)
		}
	}
}

func BenchmarkSnappyStreamEncodeBetter(b *testing.B) {
	test := []byte(largeString)

	var (
		dst = make([]byte, 0, 20+s2.MaxEncodedLen(len(test)))
		err error
	)

	b.ReportAllocs()
	b.ResetTimer()
	b.SetBytes(int64(len(test)))

	for n := 0; n < b.N; n++ {
		dst = EncodeBetter(dst[:0], test)
		if err != nil {
			b.Error("Encoding error: ", err)
		}
	}
}

func BenchmarkSnappyStreamEncodeMassive(b *testing.B) {
	massiveString := []byte(makeMassive(largeString, 10000))

	// Inject some randomness, so it isn't just all copies.
	rng := rand.New(rand.NewSource(0))
	for i := 0; i < len(massiveString)/10; i++ {
		massiveString[rng.Intn(len(massiveString))]++
	}
	var (
		dst = make([]byte, 0, s2.MaxEncodedLen(len(massiveString)))
		err error
	)

	b.ReportAllocs()
	b.ResetTimer()
	b.SetBytes(int64(len(massiveString)))

	for n := 0; n < b.N; n++ {
		dst = Encode(dst[:0], massiveString)
		if err != nil {
			b.Error("Encoding error: ", err)
		}
	}
}

func BenchmarkSnappyStreamEncodeBetterMassive(b *testing.B) {
	massiveString := []byte(makeMassive(largeString, 10000))

	// Inject some randomness, so it isn't just all copies.
	rng := rand.New(rand.NewSource(0))
	for i := 0; i < len(massiveString)/10; i++ {
		massiveString[rng.Intn(len(massiveString))]++
	}
	var (
		dst = make([]byte, 0, s2.MaxEncodedLen(len(massiveString)))
		err error
	)

	b.ReportAllocs()
	b.ResetTimer()
	b.SetBytes(int64(len(massiveString)))

	for n := 0; n < b.N; n++ {
		dst = EncodeBetter(dst[:0], massiveString)
		if err != nil {
			b.Error("Encoding error: ", err)
		}
	}
}
