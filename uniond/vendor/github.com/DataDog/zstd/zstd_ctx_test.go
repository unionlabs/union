package zstd

import (
	"bytes"
	"testing"
)

// Test compression
func TestCtxCompressDecompress(t *testing.T) {
	ctx := NewCtx()

	input := []byte("Hello World!")
	out, err := ctx.Compress(nil, input)
	if err != nil {
		t.Fatalf("Error while compressing: %v", err)
	}
	out2 := make([]byte, 1000)
	out2, err = ctx.Compress(out2, input)
	if err != nil {
		t.Fatalf("Error while compressing: %v", err)
	}
	t.Logf("Compressed: %v", out)

	rein, err := ctx.Decompress(nil, out)
	if err != nil {
		t.Fatalf("Error while decompressing: %v", err)
	}
	rein2 := make([]byte, 10)
	rein2, err = ctx.Decompress(rein2, out2)
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

func TestCtxCompressLevel(t *testing.T) {
	inputs := [][]byte{
		nil, {}, {0}, []byte("Hello World!"),
	}

	cctx := NewCtx()
	for _, input := range inputs {
		for level := BestSpeed; level <= BestCompression; level++ {
			out, err := cctx.CompressLevel(nil, input, level)
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

func TestCtxCompressLevelNoGoPointers(t *testing.T) {
	testCompressNoGoPointers(t, func(input []byte) ([]byte, error) {
		cctx := NewCtx()
		return cctx.CompressLevel(nil, input, BestSpeed)
	})
}

func TestCtxEmptySliceCompress(t *testing.T) {
	ctx := NewCtx()

	compressed, err := ctx.Compress(nil, []byte{})
	if err != nil {
		t.Fatalf("Error while compressing: %v", err)
	}
	t.Logf("Compressing empty slice gives 0x%x", compressed)
	decompressed, err := ctx.Decompress(nil, compressed)
	if err != nil {
		t.Fatalf("Error while compressing: %v", err)
	}
	if string(decompressed) != "" {
		t.Fatalf("Expected empty slice as decompressed, got %v instead", decompressed)
	}
}

func TestCtxEmptySliceDecompress(t *testing.T) {
	ctx := NewCtx()

	_, err := ctx.Decompress(nil, []byte{})
	if err != ErrEmptySlice {
		t.Fatalf("Did not get the correct error: %s", err)
	}
}

func TestCtxDecompressZeroLengthBuf(t *testing.T) {
	ctx := NewCtx()

	input := []byte("Hello World!")
	out, err := ctx.Compress(nil, input)
	if err != nil {
		t.Fatalf("Error while compressing: %v", err)
	}

	buf := make([]byte, 0)
	decompressed, err := ctx.Decompress(buf, out)
	if err != nil {
		t.Fatalf("Error while decompressing: %v", err)
	}

	if res, exp := string(input), string(decompressed); res != exp {
		t.Fatalf("expected %s but decompressed to %s", exp, res)
	}
}

func TestCtxTooSmall(t *testing.T) {
	ctx := NewCtx()

	var long bytes.Buffer
	for i := 0; i < 10000; i++ {
		long.Write([]byte("Hellow World!"))
	}
	input := long.Bytes()
	out, err := ctx.Compress(nil, input)
	if err != nil {
		t.Fatalf("Error while compressing: %v", err)
	}
	rein := make([]byte, 1)
	// This should switch to the decompression stream to handle too small dst
	rein, err = ctx.Decompress(rein, out)
	if err != nil {
		t.Fatalf("Failed decompressing: %s", err)
	}
	if string(input) != string(rein) {
		t.Fatalf("Cannot compress and decompress: %s != %s", input, rein)
	}
}

func TestCtxRealPayload(t *testing.T) {
	ctx := NewCtx()

	if raw == nil {
		t.Skip(ErrNoPayloadEnv)
	}
	dst, err := ctx.Compress(nil, raw)
	if err != nil {
		t.Fatalf("Failed to compress: %s", err)
	}
	rein, err := ctx.Decompress(nil, dst)
	if err != nil {
		t.Fatalf("Failed to decompress: %s", err)
	}
	if string(raw) != string(rein) {
		t.Fatalf("compressed/decompressed payloads are not the same (lengths: %v & %v)", len(raw), len(rein))
	}
}

func BenchmarkCtxCompression(b *testing.B) {
	ctx := NewCtx()

	if raw == nil {
		b.Fatal(ErrNoPayloadEnv)
	}
	dst := make([]byte, CompressBound(len(raw)))
	b.SetBytes(int64(len(raw)))
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, err := ctx.Compress(dst, raw)
		if err != nil {
			b.Fatalf("Failed compressing: %s", err)
		}
	}
}

func BenchmarkCtxDecompression(b *testing.B) {
	ctx := NewCtx()

	if raw == nil {
		b.Fatal(ErrNoPayloadEnv)
	}
	src := make([]byte, len(raw))
	dst, err := ctx.Compress(nil, raw)
	if err != nil {
		b.Fatalf("Failed compressing: %s", err)
	}
	b.Logf("Reduced from %v to %v", len(raw), len(dst))
	b.SetBytes(int64(len(raw)))
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		src2, err := ctx.Decompress(src, dst)
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
