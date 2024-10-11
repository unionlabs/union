package s2_test

import (
	"bytes"
	"encoding/hex"
	"fmt"
	"io"
	"math/rand"
	"os"
	"sync"
	"testing"

	"github.com/klauspost/compress/s2"
)

func ExampleIndex_Load() {
	fatalErr := func(err error) {
		if err != nil {
			panic(err)
		}
	}

	// Create a test corpus
	tmp := make([]byte, 5<<20)
	rng := rand.New(rand.NewSource(0xbeefcafe))
	rng.Read(tmp)
	// Make it compressible...
	for i, v := range tmp {
		tmp[i] = '0' + v&3
	}
	// Compress it...
	var buf bytes.Buffer
	// We use smaller blocks just for the example...
	enc := s2.NewWriter(&buf, s2.WriterBlockSize(100<<10))
	err := enc.EncodeBuffer(tmp)
	fatalErr(err)

	// Close and get index...
	idxBytes, err := enc.CloseIndex()
	fatalErr(err)

	// This is our compressed stream...
	compressed := buf.Bytes()

	var once sync.Once
	for wantOffset := int64(0); wantOffset < int64(len(tmp)); wantOffset += 555555 {
		// Let's assume we want to read from uncompressed offset 'i'
		// and we cannot seek in input, but we have the index.
		want := tmp[wantOffset:]

		// Load the index.
		var index s2.Index
		_, err = index.Load(idxBytes)
		fatalErr(err)

		// Find offset in file:
		compressedOffset, uncompressedOffset, err := index.Find(wantOffset)
		fatalErr(err)

		// Offset the input to the compressed offset.
		// Notice how we do not provide any bytes before the offset.
		input := io.Reader(bytes.NewBuffer(compressed[compressedOffset:]))
		if _, ok := input.(io.Seeker); !ok {
			// Notice how the input cannot be seeked...
			once.Do(func() {
				fmt.Println("Input does not support seeking...")
			})
		} else {
			panic("did you implement seeking on bytes.Buffer?")
		}

		// When creating the decoder we must specify that it should not
		// expect a stream identifier at the beginning og the frame.
		dec := s2.NewReader(input, s2.ReaderIgnoreStreamIdentifier())

		// We now have a reader, but it will start outputting at uncompressedOffset,
		// and not the actual offset we want, so skip forward to that.
		toSkip := wantOffset - uncompressedOffset
		err = dec.Skip(toSkip)
		fatalErr(err)

		// Read the rest of the stream...
		got, err := io.ReadAll(dec)
		fatalErr(err)
		if bytes.Equal(got, want) {
			fmt.Println("Successfully skipped forward to", wantOffset)
		} else {
			fmt.Println("Failed to skip forward to", wantOffset)
		}
	}
	// OUTPUT:
	//Input does not support seeking...
	//Successfully skipped forward to 0
	//Successfully skipped forward to 555555
	//Successfully skipped forward to 1111110
	//Successfully skipped forward to 1666665
	//Successfully skipped forward to 2222220
	//Successfully skipped forward to 2777775
	//Successfully skipped forward to 3333330
	//Successfully skipped forward to 3888885
	//Successfully skipped forward to 4444440
	//Successfully skipped forward to 4999995
}

func TestSeeking(t *testing.T) {
	compressed := bytes.Buffer{}

	// Use small blocks so there are plenty of them.
	enc := s2.NewWriter(&compressed, s2.WriterBlockSize(16<<10))
	var nElems = 1_000_000
	var testSizes = []int{100, 1_000, 10_000, 20_000, 100_000, 200_000, 400_000}
	if testing.Short() {
		nElems = 100_000
		testSizes = []int{100, 1_000, 10_000, 20_000}
	}
	testSizes = append(testSizes, nElems-1)
	//24 bytes per item plus \n = 25 bytes per record
	for i := 0; i < nElems; i++ {
		fmt.Fprintf(enc, "Item %019d\n", i)
	}

	index, err := enc.CloseIndex()
	if err != nil {
		t.Fatal(err)
	}

	// Test trimming
	slim := s2.RemoveIndexHeaders(index)
	if slim == nil {
		t.Error("Removing headers failed")
	}
	restored := s2.RestoreIndexHeaders(slim)
	if !bytes.Equal(restored, index) {
		t.Errorf("want %s, got %s", hex.EncodeToString(index), hex.EncodeToString(restored))
	}
	t.Logf("Saved %d bytes", len(index)-len(slim))

	for _, skip := range testSizes {
		t.Run(fmt.Sprintf("noSeekSkip=%d", skip), func(t *testing.T) {
			dec := s2.NewReader(io.NopCloser(bytes.NewReader(compressed.Bytes())))
			seeker, err := dec.ReadSeeker(false, nil)
			if err != nil {
				t.Fatal(err)
			}
			buf := make([]byte, 25)
			for rec := 0; rec < nElems; rec += skip {
				offset := int64(rec * 25)
				//t.Logf("Reading record %d", rec)
				_, err := seeker.Seek(offset, io.SeekStart)
				if err != nil {
					t.Fatalf("Failed to seek: %v", err)
				}
				_, err = io.ReadFull(dec, buf)
				if err != nil {
					t.Fatalf("Failed to seek: %v", err)
				}
				expected := fmt.Sprintf("Item %019d\n", rec)
				if string(buf) != expected {
					t.Fatalf("Expected %q, got %q", expected, buf)
				}
			}
		})
		t.Run(fmt.Sprintf("seekSkip=%d", skip), func(t *testing.T) {
			dec := s2.NewReader(io.ReadSeeker(bytes.NewReader(compressed.Bytes())))
			seeker, err := dec.ReadSeeker(false, nil)
			if err != nil {
				t.Fatal(err)
			}
			buf := make([]byte, 25)
			for rec := 0; rec < nElems; rec += skip {
				offset := int64(rec * 25)
				//t.Logf("Reading record %d", rec)
				_, err := seeker.Seek(offset, io.SeekStart)
				if err != nil {
					t.Fatalf("Failed to seek: %v", err)
				}
				_, err = io.ReadFull(dec, buf)
				if err != nil {
					t.Fatalf("Failed to seek: %v", err)
				}
				expected := fmt.Sprintf("Item %019d\n", rec)
				if string(buf) != expected {
					t.Fatalf("Expected %q, got %q", expected, buf)
				}
			}
		})
		t.Run(fmt.Sprintf("noSeekIndexSkip=%d", skip), func(t *testing.T) {
			dec := s2.NewReader(io.NopCloser(bytes.NewReader(compressed.Bytes())))
			seeker, err := dec.ReadSeeker(false, index)
			if err != nil {
				t.Fatal(err)
			}
			buf := make([]byte, 25)
			for rec := 0; rec < nElems; rec += skip {
				offset := int64(rec * 25)
				//t.Logf("Reading record %d", rec)
				_, err := seeker.Seek(offset, io.SeekStart)
				if err != nil {
					t.Fatalf("Failed to seek: %v", err)
				}
				_, err = io.ReadFull(dec, buf)
				if err != nil {
					t.Fatalf("Failed to seek: %v", err)
				}
				expected := fmt.Sprintf("Item %019d\n", rec)
				if string(buf) != expected {
					t.Fatalf("Expected %q, got %q", expected, buf)
				}
			}
		})
		t.Run(fmt.Sprintf("seekIndexSkip=%d", skip), func(t *testing.T) {
			dec := s2.NewReader(io.ReadSeeker(bytes.NewReader(compressed.Bytes())))

			seeker, err := dec.ReadSeeker(false, index)
			if err != nil {
				t.Fatal(err)
			}
			buf := make([]byte, 25)
			for rec := 0; rec < nElems; rec += skip {
				offset := int64(rec * 25)
				//t.Logf("Reading record %d", rec)
				_, err := seeker.Seek(offset, io.SeekStart)
				if err != nil {
					t.Fatalf("Failed to seek: %v", err)
				}
				_, err = io.ReadFull(dec, buf)
				if err != nil {
					t.Fatalf("Failed to seek: %v", err)
				}
				expected := fmt.Sprintf("Item %019d\n", rec)
				if string(buf) != expected {
					t.Fatalf("Expected %q, got %q", expected, buf)
				}
			}
		})
	}
	// Test seek current
	t.Run("seekCurrent", func(t *testing.T) {
		dec := s2.NewReader(io.ReadSeeker(bytes.NewReader(compressed.Bytes())))

		seeker, err := dec.ReadSeeker(true, index)
		if err != nil {
			t.Fatal(err)
		}
		buf := make([]byte, 25)
		rng := rand.New(rand.NewSource(0))
		var currentOff int64
		for i := 0; i < nElems/10; i++ {
			rec := rng.Intn(nElems)
			offset := int64(rec * 25)
			//t.Logf("Reading record %d", rec)
			absOff, err := seeker.Seek(offset-currentOff, io.SeekCurrent)
			if err != nil {
				t.Fatalf("Failed to seek: %v", err)
			}
			if absOff != offset {
				t.Fatalf("Unexpected seek offset: want %v, got %v", offset, absOff)
			}
			_, err = io.ReadFull(dec, buf)
			if err != nil {
				t.Fatalf("Failed to seek: %v", err)
			}
			expected := fmt.Sprintf("Item %019d\n", rec)
			if string(buf) != expected {
				t.Fatalf("Expected %q, got %q", expected, buf)
			}
			// Adjust offset
			currentOff = offset + int64(len(buf))
		}
	})
	// Test ReadAt
	t.Run("ReadAt", func(t *testing.T) {
		dec := s2.NewReader(io.ReadSeeker(bytes.NewReader(compressed.Bytes())))

		seeker, err := dec.ReadSeeker(true, index)
		if err != nil {
			t.Fatal(err)
		}
		buf := make([]byte, 25)
		rng := rand.New(rand.NewSource(0))
		for i := 0; i < nElems/10; i++ {
			rec := rng.Intn(nElems)
			offset := int64(rec * 25)
			n, err := seeker.ReadAt(buf, offset)
			if err != nil {
				t.Fatalf("Failed to seek: %v", err)
			}
			if n != len(buf) {
				t.Fatalf("Unexpected read length: want %v, got %v", len(buf), n)
			}
			expected := fmt.Sprintf("Item %019d\n", rec)
			if string(buf) != expected {
				t.Fatalf("Expected %q, got %q", expected, buf)
			}
		}
	})
}

// ExampleIndexStream shows an example of indexing a stream
// and indexing it after it has been written.
// The index can either be appended.
func ExampleIndexStream() {
	fatalErr := func(err error) {
		if err != nil {
			panic(err)
		}
	}

	// Create a test stream without index
	var streamName = ""
	tmp := make([]byte, 5<<20)
	{
		rng := rand.New(rand.NewSource(0xbeefcafe))
		rng.Read(tmp)
		// Make it compressible...
		for i, v := range tmp {
			tmp[i] = '0' + v&3
		}
		// Compress it...
		output, err := os.CreateTemp("", "IndexStream")
		streamName = output.Name()
		fatalErr(err)

		// We use smaller blocks just for the example...
		enc := s2.NewWriter(output, s2.WriterSnappyCompat())
		err = enc.EncodeBuffer(tmp)
		fatalErr(err)

		// Close and get index...
		err = enc.Close()
		fatalErr(err)
		err = output.Close()
		fatalErr(err)
	}

	// Open our compressed stream without an index...
	stream, err := os.Open(streamName)
	fatalErr(err)
	defer stream.Close()

	var indexInput = io.Reader(stream)
	var indexOutput io.Writer
	var indexedName string

	// Should index be combined with stream by appending?
	// This could also be done by appending to an os.File
	// If not it will be written to a separate file.
	const combineOutput = false

	// Function to easier use defer.
	func() {
		if combineOutput {
			output, err := os.CreateTemp("", "IndexStream-Combined")
			fatalErr(err)
			defer func() {
				fatalErr(output.Close())
				if false {
					fi, err := os.Stat(output.Name())
					fatalErr(err)
					fmt.Println("Combined:", fi.Size(), "bytes")
				} else {
					fmt.Println("Index saved")
				}
			}()

			// Everything read from stream will also be written to output.
			indexedName = output.Name()
			indexInput = io.TeeReader(stream, output)
			indexOutput = output
		} else {
			output, err := os.CreateTemp("", "IndexStream-Index")
			fatalErr(err)
			defer func() {
				fatalErr(output.Close())
				fi, err := os.Stat(output.Name())
				fatalErr(err)
				if false {
					fmt.Println("Index:", fi.Size(), "bytes")
				} else {
					fmt.Println("Index saved")
				}
			}()
			indexedName = output.Name()
			indexOutput = output
		}

		// Index the input
		idx, err := s2.IndexStream(indexInput)
		fatalErr(err)

		// Write the index
		_, err = indexOutput.Write(idx)
		fatalErr(err)
	}()

	if combineOutput {
		// Read from combined stream only.
		stream, err := os.Open(indexedName)
		fatalErr(err)
		defer stream.Close()
		// Create a reader with the input.
		// We assert that the stream is an io.ReadSeeker.
		r := s2.NewReader(io.ReadSeeker(stream))

		// Request a ReadSeeker with random access.
		// This will load the index from the stream.
		rs, err := r.ReadSeeker(true, nil)
		fatalErr(err)

		_, err = rs.Seek(-10, io.SeekEnd)
		fatalErr(err)

		b, err := io.ReadAll(rs)
		fatalErr(err)
		if want := tmp[len(tmp)-10:]; !bytes.Equal(b, want) {
			fatalErr(fmt.Errorf("wanted %v, got %v", want, b))
		}
		fmt.Println("last 10 bytes read")

		_, err = rs.Seek(10, io.SeekStart)
		fatalErr(err)
		_, err = io.ReadFull(rs, b)
		fatalErr(err)
		if want := tmp[10:20]; !bytes.Equal(b, want) {
			fatalErr(fmt.Errorf("wanted %v, got %v", want, b))
		}
		fmt.Println("10 bytes at offset 10 read")
	} else {
		// Read from separate stream and index.
		stream, err := os.Open(streamName)
		fatalErr(err)
		defer stream.Close()
		// Create a reader with the input.
		// We assert that the stream is an io.ReadSeeker.
		r := s2.NewReader(io.ReadSeeker(stream))

		// Read the separate index.
		index, err := os.ReadFile(indexedName)
		fatalErr(err)

		// Request a ReadSeeker with random access.
		// The provided index will be used.
		rs, err := r.ReadSeeker(true, index)
		fatalErr(err)

		_, err = rs.Seek(-10, io.SeekEnd)
		fatalErr(err)

		b, err := io.ReadAll(rs)
		fatalErr(err)
		if want := tmp[len(tmp)-10:]; !bytes.Equal(b, want) {
			fatalErr(fmt.Errorf("wanted %v, got %v", want, b))
		}
		fmt.Println("last 10 bytes read")

		_, err = rs.Seek(10, io.SeekStart)
		fatalErr(err)
		_, err = io.ReadFull(rs, b)
		fatalErr(err)
		if want := tmp[10:20]; !bytes.Equal(b, want) {
			fatalErr(fmt.Errorf("wanted %v, got %v", want, b))
		}
		fmt.Println("10 bytes at offset 10 read")
	}

	// OUTPUT:
	// Index saved
	// last 10 bytes read
	// 10 bytes at offset 10 read
}
