package zstd

import (
	"bytes"
	"errors"
	"fmt"
	"io"
	"io/ioutil"
	"log"
	"os"
	"runtime/debug"
	"strings"
	"testing"
)

func failOnError(t *testing.T, msg string, err error) {
	if err != nil {
		debug.PrintStack()
		t.Fatalf("%s: %s", msg, err)
	}
}

func testCompressionDecompression(t *testing.T, dict []byte, payload []byte, nbWorkers int) {
	var w bytes.Buffer
	writer := NewWriterLevelDict(&w, DefaultCompression, dict)

	if nbWorkers > 1 {
		if err := writer.SetNbWorkers(nbWorkers); err == ErrNoParallelSupport {
			t.Skip()
		}
	}

	_, err := writer.Write(payload)
	failOnError(t, "Failed writing to compress object", err)
	failOnError(t, "Failed to close compress object", writer.Close())
	out := w.Bytes()
	t.Logf("Compressed %v -> %v bytes", len(payload), len(out))
	failOnError(t, "Failed compressing", err)
	rr := bytes.NewReader(out)
	// Check that we can decompress with Decompress()
	decompressed, err := Decompress(nil, out)
	failOnError(t, "Failed to decompress with Decompress()", err)
	if string(payload) != string(decompressed) {
		t.Fatalf("Payload did not match, lengths: %v & %v", len(payload), len(decompressed))
	}

	// Decompress
	r := NewReaderDict(rr, dict)
	dst := make([]byte, len(payload))
	n, err := io.ReadFull(r, dst)
	if err != nil {
		failOnError(t, "Failed to read for decompression", err)
	}
	dst = dst[:n]
	if string(payload) != string(dst) { // Only print if we can print
		if len(payload) < 100 && len(dst) < 100 {
			t.Fatalf("Cannot compress and decompress: %s != %s", payload, dst)
		} else {
			t.Fatalf("Cannot compress and decompress (lengths: %v bytes & %v bytes)", len(payload), len(dst))
		}
	}
	// Check EOF
	n, err = r.Read(dst)
	if err != io.EOF && len(dst) > 0 { // If we want 0 bytes, that should work
		t.Fatalf("Error should have been EOF, was %s instead: (%v bytes read: %s)", err, n, dst[:n])
	}
	failOnError(t, "Failed to close decompress object", r.Close())
}

func TestResize(t *testing.T) {
	if len(resize(nil, 129)) != 129 {
		t.Fatalf("Cannot allocate new slice")
	}
	a := make([]byte, 1, 200)
	b := resize(a, 129)
	if &a[0] != &b[0] {
		t.Fatalf("Address changed")
	}
	if len(b) != 129 {
		t.Fatalf("Wrong size")
	}
	c := make([]byte, 5, 10)
	d := resize(c, 129)
	if len(d) != 129 {
		t.Fatalf("Cannot allocate a new slice")
	}
}

func TestStreamSimpleCompressionDecompression(t *testing.T) {
	testCompressionDecompression(t, nil, []byte("Hello world!"), 1)
}

func TestStreamEmptySlice(t *testing.T) {
	testCompressionDecompression(t, nil, []byte{}, 1)
}

func TestZstdReaderLong(t *testing.T) {
	var long bytes.Buffer
	for i := 0; i < 10000; i++ {
		long.Write([]byte("Hellow World!"))
	}
	testCompressionDecompression(t, nil, long.Bytes(), 1)
}

func doStreamCompressionDecompression() error {
	payload := []byte("Hello World!")
	repeat := 10000
	var intermediate bytes.Buffer
	w := NewWriterLevel(&intermediate, 4)
	for i := 0; i < repeat; i++ {
		_, err := w.Write(payload)
		if err != nil {
			return fmt.Errorf("failed writing to compress object: %w", err)
		}
	}
	err := w.Close()
	if err != nil {
		return fmt.Errorf("failed to close compressor: %w", err)
	}

	// Decompress
	r := NewReader(&intermediate)
	dst := make([]byte, len(payload))
	for i := 0; i < repeat; i++ {
		n, err := r.Read(dst)
		if err != nil {
			return fmt.Errorf("failed to decompress: %w", err)
		}
		if n != len(payload) {
			return fmt.Errorf("did not read enough bytes: %d != %d", n, len(payload))
		}
		if string(dst) != string(payload) {
			return fmt.Errorf("Did not read the same %s != %s", string(dst), string(payload))
		}
	}
	// Check EOF
	n, err := r.Read(dst)
	if err != io.EOF {
		return fmt.Errorf("Error should have been EOF (%v bytes read: %s): %w",
			n, string(dst[:n]), err)
	}
	err = r.Close()
	if err != nil {
		return fmt.Errorf("failed to close decompress object: %w", err)
	}
	return nil
}

func TestStreamCompressionDecompressionParallel(t *testing.T) {
	// start many goroutines: triggered Cgo stack growth related bugs
	if os.Getenv("DISABLE_BIG_TESTS") != "" {
		t.Skip("Big (memory) tests are disabled")
	}
	const threads = 500
	errChan := make(chan error)

	for i := 0; i < threads; i++ {
		go func() {
			errChan <- doStreamCompressionDecompression()
		}()
	}

	for i := 0; i < threads; i++ {
		err := <-errChan
		if err != nil {
			t.Error("task failed:", err)
		}
	}
}

func doStreamCompressionStackDepth(stackDepth int) error {
	if stackDepth == 0 {
		return doStreamCompressionDecompression()
	}
	return doStreamCompressionStackDepth(stackDepth - 1)
}

func TestStreamCompressionDecompressionCgoStack(t *testing.T) {
	// this crashed with: GODEBUG=efence=1 go test .
	if os.Getenv("DISABLE_BIG_TESTS") != "" {
		t.Skip("Big (memory) tests are disabled")
	}
	const maxStackDepth = 200

	for i := 0; i < maxStackDepth; i++ {
		err := doStreamCompressionStackDepth(i)
		if err != nil {
			t.Error("task failed:", err)
		}
	}
}

func TestStreamRealPayload(t *testing.T) {
	if raw == nil {
		t.Skip(ErrNoPayloadEnv)
	}
	testCompressionDecompression(t, nil, raw, 1)
}

func TestStreamEmptyPayload(t *testing.T) {
	w := bytes.NewBuffer(nil)
	writer := NewWriter(w)
	_, err := writer.Write(nil)
	failOnError(t, "failed to write empty slice", err)
	err = writer.Close()
	failOnError(t, "failed to close", err)
	compressed := w.Bytes()
	t.Logf("compressed buffer: 0x%x", compressed)
	// Now recheck that if we decompress, we get empty slice
	r := bytes.NewBuffer(compressed)
	reader := NewReader(r)
	decompressed, err := ioutil.ReadAll(reader)
	failOnError(t, "failed to read", err)
	err = reader.Close()
	failOnError(t, "failed to close", err)
	if string(decompressed) != "" {
		t.Fatalf("Expected empty slice as decompressed, got %v instead", decompressed)
	}
}

func TestStreamFlush(t *testing.T) {
	// use an actual os pipe so that
	// - it's buffered and we don't get a 1-read = 1-write behaviour (io.Pipe)
	// - reading doesn't send EOF when we're done reading the buffer (bytes.Buffer)
	pr, pw, err := os.Pipe()
	failOnError(t, "Failed creating pipe", err)
	defer pw.Close()
	defer pr.Close()

	writer := NewWriter(pw)
	reader := NewReader(pr)

	payload := "cc" // keep the payload short to make sure it will not be automatically flushed by zstd
	buf := make([]byte, len(payload))

	for i := 0; i < 5; i++ {
		_, err := writer.Write([]byte(payload))
		failOnError(t, "Failed writing to compress object", err)

		err = writer.Flush()
		failOnError(t, "Failed flushing compress object", err)

		_, err = io.ReadFull(reader, buf)
		failOnError(t, "Failed reading uncompress object", err)

		if string(buf) != payload {
			debug.PrintStack()
			log.Fatal("Uncompressed object mismatch")
		}
	}

	failOnError(t, "Failed to close compress object", writer.Close())
	failOnError(t, "Failed to close uncompress object", reader.Close())
}

type closeableWriter struct {
	w      io.Writer
	closed bool
}

func (c *closeableWriter) Write(p []byte) (n int, err error) {
	if c.closed {
		return 0, errors.New("io: Write on a closed closeableWriter")
	}
	return c.w.Write(p)
}

func (c *closeableWriter) Close() error {
	c.closed = true
	return nil
}

func TestStreamFlushError(t *testing.T) {
	var bw bytes.Buffer
	w := closeableWriter{w: &bw}
	writer := NewWriter(&w)

	_, err := writer.Write([]byte("cc"))
	failOnError(t, "Failed writing to compress object", err)

	w.Close()
	if err = writer.Flush(); err == nil {
		debug.PrintStack()
		t.Fatal("Writer.Flush returned no error when writing to underlying io.Writer failed")
	}
}

func TestStreamCloseError(t *testing.T) {
	var bw bytes.Buffer
	w := closeableWriter{w: &bw}
	writer := NewWriter(&w)

	_, err := writer.Write([]byte("cc"))
	failOnError(t, "Failed writing to compress object", err)

	w.Close()
	if err = writer.Close(); err == nil {
		debug.PrintStack()
		t.Fatal("Writer.Close returned no error when writing to underlying io.Writer failed")
	}
}

type breakingReader struct{}

func (r *breakingReader) Read(p []byte) (int, error) {
	return len(p) - 1, io.ErrUnexpectedEOF
}

func TestStreamDecompressionUnexpectedEOFHandling(t *testing.T) {
	r := NewReader(&breakingReader{})
	_, err := r.Read(make([]byte, 1024))
	if err == nil {
		t.Error("Underlying error was handled silently")
	}
}

func TestStreamCompressionChunks(t *testing.T) {
	MB := 1024 * 1024
	totalSize := 100 * MB
	chunk := 1 * MB

	rawData := make([]byte, totalSize)
	r := NewRandBytes()
	r.Read(rawData)

	compressed, _ := Compress(nil, rawData)
	var streamCompressed bytes.Buffer
	w := NewWriter(&streamCompressed)
	for i := 0; i < totalSize; i += chunk {
		end := i + chunk
		if end >= len(rawData) {
			end = len(rawData)
		}
		n, err := w.Write(rawData[i:end])
		if err != nil {
			t.Fatalf("Error while writing: %s", err)
		}
		if n != (end - i) {
			t.Fatalf("Did not write the full ammount of data: %v != %v", n, end-i)
		}
	}
	err := w.Close()
	if err != nil {
		t.Fatalf("Failed to close writer: %s", err)
	}
	streamCompressedBytes := streamCompressed.Bytes()
	t.Logf("Compressed with single call=%v bytes, stream compressed=%v bytes", len(compressed), len(streamCompressedBytes))
	decompressed, err := Decompress(nil, streamCompressedBytes)
	if err != nil {
		t.Fatalf("Failed to decompress: %s", err)
	}
	if !bytes.Equal(rawData, decompressed) {
		t.Fatalf("Compression/Decompression data is not equal to original data")
	}
}

func TestStreamDecompressionChunks(t *testing.T) {
	MB := 1024 * 1024
	totalSize := 100 * MB
	chunk := 1 * MB

	rawData := make([]byte, totalSize)
	r := NewRandBytes()
	r.Read(rawData)

	compressed, _ := Compress(nil, rawData)
	streamDecompressed := bytes.NewReader(compressed)
	reader := NewReader(streamDecompressed)

	result := make([]byte, 0, totalSize)
	for {
		chunkBytes := make([]byte, chunk)
		n, err := reader.Read(chunkBytes)
		if err != nil && err != io.EOF {
			t.Fatalf("Got an error while reading: %s", err)
		}
		result = append(result, chunkBytes[:n]...)
		if err == io.EOF {
			break
		}
	}

	err := reader.Close()
	if err != nil {
		t.Fatalf("Failed to close writer: %s", err)
	}

	if !bytes.Equal(rawData, result) {
		t.Fatalf("Decompression data is not equal to original data")
	}
}

func TestStreamWriteNoGoPointers(t *testing.T) {
	testCompressNoGoPointers(t, func(input []byte) ([]byte, error) {
		buf := &bytes.Buffer{}
		zw := NewWriter(buf)
		_, err := zw.Write(input)
		if err != nil {
			return nil, err
		}
		err = zw.Close()
		if err != nil {
			return nil, err
		}
		return buf.Bytes(), nil
	})
}

func TestStreamSetNbWorkers(t *testing.T) {
	// Build a big string first
	s := strings.Repeat("foobaa", 1000*1000)

	nbWorkers := 4
	testCompressionDecompression(t, nil, []byte(s), nbWorkers)
}

func BenchmarkStreamCompression(b *testing.B) {
	if raw == nil {
		b.Fatal(ErrNoPayloadEnv)
	}
	var intermediate bytes.Buffer
	w := NewWriter(&intermediate)
	// w.SetNbWorkers(8)
	defer w.Close()
	b.SetBytes(int64(len(raw)))
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, err := w.Write(raw)
		if err != nil {
			b.Fatalf("Failed writing to compress object: %s", err)
		}
		// Prevent from unbound buffer growth.
		intermediate.Reset()
	}
}

func BenchmarkStreamDecompression(b *testing.B) {
	if raw == nil {
		b.Fatal(ErrNoPayloadEnv)
	}
	compressed, err := Compress(nil, raw)
	if err != nil {
		b.Fatalf("Failed to compress: %s", err)
	}
	_, err = Decompress(nil, compressed)
	if err != nil {
		b.Fatalf("Problem: %s", err)
	}

	dst := make([]byte, len(raw))
	b.SetBytes(int64(len(raw)))
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		rr := bytes.NewReader(compressed)
		r := NewReader(rr)
		_, err := io.ReadFull(r, dst)
		if err != nil {
			b.Fatalf("Failed to decompress: %s", err)
		}
		r.Close()
	}
}
