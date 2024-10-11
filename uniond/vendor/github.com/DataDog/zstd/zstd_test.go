package zstd

import (
	"bytes"
	b64 "encoding/base64"
	"errors"
	"fmt"
	"io/ioutil"
	"os"
	"strconv"
	"strings"
	"testing"
)

var raw []byte
var (
	ErrNoPayloadEnv = errors.New("PAYLOAD env was not set")
)

func init() {
	var err error
	payload := os.Getenv("PAYLOAD")
	if len(payload) > 0 {
		raw, err = ioutil.ReadFile(payload)
		if err != nil {
			fmt.Printf("Error opening payload: %s\n", err)
		}
	}
}

// Test our version of compress bound vs C implementation
func TestCompressBound(t *testing.T) {
	tests := []int{0, 1, 2, 10, 456, 15468, 1313, 512, 2147483632}
	for _, test := range tests {
		if CompressBound(test) != cCompressBound(test) {
			t.Fatalf("For %v, results are different: %v (actual) != %v (expected)", test,
				CompressBound(test), cCompressBound(test))
		}
	}
}

// Test error code
func TestErrorCode(t *testing.T) {
	tests := make([]int, 211)
	for i := 0; i < len(tests); i++ {
		tests[i] = i - 105
	}
	for _, test := range tests {
		err := getError(test)
		if err == nil && cIsError(test) {
			t.Fatalf("C function returned error for %v but ours did not", test)
		} else if err != nil && !cIsError(test) {
			t.Fatalf("Ours function returned error for %v but C one did not", test)
		}
	}

}

// Test compression
func TestCompressDecompress(t *testing.T) {
	input := []byte("Hello World!")
	out, err := Compress(nil, input)
	if err != nil {
		t.Fatalf("Error while compressing: %v", err)
	}
	out2 := make([]byte, 1000)
	out2, err = Compress(out2, input)
	if err != nil {
		t.Fatalf("Error while compressing: %v", err)
	}
	t.Logf("Compressed: %v", out)
	rein, err := Decompress(nil, out)
	if err != nil {
		t.Fatalf("Error while decompressing: %v", err)
	}
	rein2 := make([]byte, 10)
	rein2, err = Decompress(rein2, out2)
	if err != nil {
		t.Fatalf("Error while decompressing: %v", err)
	}

	if string(input) != string(rein) {
		t.Fatalf("Cannot compress and decompress: %s != %s", input, rein)
	}
	if string(input) != string(rein2) {
		t.Fatalf("Cannot compress and decompress: %s != %s", input, rein)
	}
}

func TestCompressDecompressInto(t *testing.T) {
	payload := []byte("Hello World!")
	compressed, err := Compress(make([]byte, CompressBound(len(payload))), payload)
	if err != nil {
		t.Fatalf("Error while compressing: %v", err)
	}
	t.Logf("Compressed: %v", compressed)

	// We know the size of the payload; construct a buffer that perfectly fits
	// the payload and use DecompressInto.
	decompressed := make([]byte, len(payload))
	if n, err := DecompressInto(decompressed, compressed); err != nil {
		t.Fatalf("error while decompressing into buffer of size %d: %v",
			len(decompressed), err)
	} else if n != len(decompressed) {
		t.Errorf("DecompressedInto = (%d, nil), want (%d, nil)", n, len(decompressed))
	}
	if !bytes.Equal(payload, decompressed) {
		t.Fatalf("DecompressInto(_, Compress(_, %q)) yielded %q, want %q", payload, decompressed, payload)
	}

	// Ensure that decompressing into a buffer too small errors appropriately.
	smallBuffer := make([]byte, len(payload)-1)
	if _, err := DecompressInto(smallBuffer, compressed); !IsDstSizeTooSmallError(err) {
		t.Fatalf("DecompressInto(<%d-sized buffer>, Compress(_, %q)) = %v, want 'Destination buffer is too small'",
			len(smallBuffer), payload, err)
	}
}

func TestCompressLevel(t *testing.T) {
	inputs := [][]byte{
		nil, {}, {0}, []byte("Hello World!"),
	}

	for _, input := range inputs {
		for level := BestSpeed; level <= BestCompression; level++ {
			out, err := CompressLevel(nil, input, level)
			if err != nil {
				t.Errorf("input=%#v level=%d CompressLevel failed err=%s", string(input), level, err.Error())
				continue
			}

			orig, err := Decompress(nil, out)
			if err != nil {
				t.Errorf("input=%#v level=%d Decompress failed err=%s", string(input), level, err.Error())
				continue
			}
			if !bytes.Equal(orig, input) {
				t.Errorf("input=%#v level=%d orig does not match: %#v", string(input), level, string(orig))
			}
		}
	}
}

// structWithGoPointers contains a byte buffer and a pointer to Go objects (slice). This means
// Cgo checks can fail when passing a pointer to buffer:
// "panic: runtime error: cgo argument has Go pointer to Go pointer"
// https://github.com/golang/go/issues/14210#issuecomment-346402945
type structWithGoPointers struct {
	buffer [1]byte
	slice  []byte
}

// testCompressDecompressByte ensures that functions use the correct unsafe.Pointer assignment
// to avoid "Go pointer to Go pointer" panics.
func testCompressNoGoPointers(t *testing.T, compressFunc func(input []byte) ([]byte, error)) {
	t.Helper()

	s := structWithGoPointers{}
	s.buffer[0] = 0x42
	s.slice = s.buffer[:1]

	compressed, err := compressFunc(s.slice)
	if err != nil {
		t.Fatal(err)
	}
	decompressed, err := Decompress(nil, compressed)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(decompressed, s.slice) {
		t.Errorf("decompressed=%#v input=%#v", decompressed, s.slice)
	}
}

func TestCompressLevelNoGoPointers(t *testing.T) {
	testCompressNoGoPointers(t, func(input []byte) ([]byte, error) {
		return CompressLevel(nil, input, BestSpeed)
	})
}

func doCompressLevel(payload []byte, out []byte) error {
	out, err := CompressLevel(out, payload, DefaultCompression)
	if err != nil {
		return fmt.Errorf("failed calling CompressLevel: %w", err)
	}
	if len(out) == 0 {
		return errors.New("CompressLevel must return non-empty bytes")
	}
	return nil
}

func useStackSpaceCompressLevel(payload []byte, out []byte, level int) error {
	if level == 0 {
		return doCompressLevel(payload, out)
	}
	return useStackSpaceCompressLevel(payload, out, level-1)
}

func TestCompressLevelStackCgoBug(t *testing.T) {
	// CompressLevel previously had a bug where it would access the wrong pointer
	// This test would crash when run with CGODEBUG=efence=1 go test .
	const maxStackLevels = 100

	payload := []byte("Hello World!")
	// allocate the output buffer so CompressLevel does not allocate it
	out := make([]byte, CompressBound(len(payload)))

	for level := 0; level < maxStackLevels; level++ {
		err := useStackSpaceCompressLevel(payload, out, level)
		if err != nil {
			t.Fatal("CompressLevel failed:", err)
		}
	}
}

func TestEmptySliceCompress(t *testing.T) {
	compressed, err := Compress(nil, []byte{})
	if err != nil {
		t.Fatalf("Error while compressing: %v", err)
	}
	t.Logf("Compressing empty slice gives 0x%x", compressed)
	decompressed, err := Decompress(nil, compressed)
	if err != nil {
		t.Fatalf("Error while compressing: %v", err)
	}
	if string(decompressed) != "" {
		t.Fatalf("Expected empty slice as decompressed, got %v instead", decompressed)
	}
}

func TestEmptySliceDecompress(t *testing.T) {
	_, err := Decompress(nil, []byte{})
	if err != ErrEmptySlice {
		t.Fatalf("Did not get the correct error: %s", err)
	}
}

func TestDecompressZeroLengthBuf(t *testing.T) {
	input := []byte("Hello World!")
	out, err := Compress(nil, input)
	if err != nil {
		t.Fatalf("Error while compressing: %v", err)
	}

	buf := make([]byte, 0)
	decompressed, err := Decompress(buf, out)
	if err != nil {
		t.Fatalf("Error while decompressing: %v", err)
	}

	if res, exp := string(input), string(decompressed); res != exp {
		t.Fatalf("expected %s but decompressed to %s", exp, res)
	}
}

func TestTooSmall(t *testing.T) {
	var long bytes.Buffer
	for i := 0; i < 10000; i++ {
		long.Write([]byte("Hellow World!"))
	}
	input := long.Bytes()
	out, err := Compress(nil, input)
	if err != nil {
		t.Fatalf("Error while compressing: %v", err)
	}
	rein := make([]byte, 1)
	// This should switch to the decompression stream to handle too small dst
	rein, err = Decompress(rein, out)
	if err != nil {
		t.Fatalf("Failed decompressing: %s", err)
	}
	if string(input) != string(rein) {
		t.Fatalf("Cannot compress and decompress: %s != %s", input, rein)
	}
}

func TestRealPayload(t *testing.T) {
	if raw == nil {
		t.Skip(ErrNoPayloadEnv)
	}
	dst, err := Compress(nil, raw)
	if err != nil {
		t.Fatalf("Failed to compress: %s", err)
	}
	rein, err := Decompress(nil, dst)
	if err != nil {
		t.Fatalf("Failed to decompress: %s", err)
	}
	if string(raw) != string(rein) {
		t.Fatalf("compressed/decompressed payloads are not the same (lengths: %v & %v)", len(raw), len(rein))
	}
}

func TestLegacy(t *testing.T) {
	// payloads compressed with zstd v0.5
	// needs ZSTD_LEGACY_SUPPORT=5 or less
	testCases := []struct {
		input    string
		expected string
	}{
		{"%\xb5/\xfd\x00@\x00\x1bcompressed with legacy zstd\xc0\x00\x00", "compressed with legacy zstd"},
		{"%\xb5/\xfd\x00\x00\x00A\x11\x007\x14\xb0\xb5\x01@\x1aR\xb6iI7[FH\x022u\xe0O-\x18\xe3G\x9e2\xab\xd9\xea\xca7؊\xee\x884\xbf\xe7\xdc\xe4@\xe1-\x9e\xac\xf0\xf2\x86\x0f\xf1r\xbb7\b\x81Z\x01\x00\x01\x00\xdf`\xfe\xc0\x00\x00", "compressed with legacy zstd"},
	}
	for i, testCase := range testCases {
		t.Run(strconv.Itoa(i), func(t *testing.T) {
			out, err := Decompress(nil, []byte(testCase.input))
			if err != nil {
				t.Fatal(err)
			}
			if !strings.Contains(string(out), testCase.expected) {
				t.Errorf("expected to find %#v; output=%#v", testCase.expected, string(out))
			}
		})
	}
}

func TestBadPayloadZipBomb(t *testing.T) {
	payload, _ := b64.StdEncoding.DecodeString("KLUv/dcwMDAwMDAwMDAwMAAA")
	_, err := Decompress(nil, payload)
	if err.Error() != "Src size is incorrect" {
		t.Fatal("zstd should detect that the size is incorrect")
	}
}

func TestSmallPayload(t *testing.T) {
	// Test that we can compress really small payloads and this doesn't generate a huge output buffer
	compressed, err := Compress(nil, []byte("a"))
	if err != nil {
		t.Fatalf("failed to compress: %s", err)
	}

	preAllocated := make([]byte, 1, 64) // Don't use more than that
	decompressed, err := Decompress(preAllocated, compressed)
	if err != nil {
		t.Fatalf("failed to compress: %s", err)
	}

	if &(preAllocated[0]) != &(decompressed[0]) { // They should point to the same spot (no realloc)
		t.Fatal("Compression buffer was changed")
	}

}

func BenchmarkCompression(b *testing.B) {
	if raw == nil {
		b.Fatal(ErrNoPayloadEnv)
	}
	dst := make([]byte, CompressBound(len(raw)))
	b.SetBytes(int64(len(raw)))
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, err := Compress(dst, raw)
		if err != nil {
			b.Fatalf("Failed compressing: %s", err)
		}
	}
}

func BenchmarkDecompression(b *testing.B) {
	if raw == nil {
		b.Fatal(ErrNoPayloadEnv)
	}
	src := make([]byte, len(raw))
	dst, err := Compress(nil, raw)
	if err != nil {
		b.Fatalf("Failed compressing: %s", err)
	}
	b.Logf("Reduced from %v to %v", len(raw), len(dst))
	b.SetBytes(int64(len(raw)))
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		src2, err := Decompress(src, dst)
		if err != nil {
			b.Fatalf("Failed decompressing: %s", err)
		}
		b.StopTimer()
		if !bytes.Equal(raw, src2) {
			b.Fatalf("Results are not the same: %v != %v", len(raw), len(src2))
		}
		b.StartTimer()
	}
}
