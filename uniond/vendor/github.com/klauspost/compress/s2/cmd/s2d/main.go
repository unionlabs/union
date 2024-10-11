package main

import (
	"bufio"
	"bytes"
	"errors"
	"flag"
	"fmt"
	"io"
	"net/http"
	"os"
	"runtime"
	"runtime/debug"
	"strconv"
	"strings"
	"sync"
	"time"
	"unicode"

	"github.com/klauspost/compress/s2"
	"github.com/klauspost/compress/s2/cmd/internal/filepathx"
	"github.com/klauspost/compress/s2/cmd/internal/readahead"
)

var (
	safe   = flag.Bool("safe", false, "Do not overwrite output files")
	verify = flag.Bool("verify", false, "Verify files, but do not write output")
	stdout = flag.Bool("c", false, "Write all output to stdout. Multiple input files will be concatenated")
	remove = flag.Bool("rm", false, "Delete source file(s) after successful decompression")
	quiet  = flag.Bool("q", false, "Don't write any output to terminal, except errors")
	bench  = flag.Int("bench", 0, "Run benchmark n times. No output will be written")
	tail   = flag.String("tail", "", "Return last of compressed file. Examples: 92, 64K, 256K, 1M, 4M. Requires Index")
	offset = flag.String("offset", "", "Start at offset. Examples: 92, 64K, 256K, 1M, 4M. Requires Index")
	help   = flag.Bool("help", false, "Display help")
	out    = flag.String("o", "", "Write output to another file. Single input file only")
	block  = flag.Bool("block", false, "Decompress as a single block. Will load content into memory.")
	cpu    = flag.Int("cpu", runtime.NumCPU(), "Decompress streams using this amount of threads")

	version = "(dev)"
	date    = "(unknown)"
)

const (
	s2Ext     = ".s2"
	snappyExt = ".sz" // https://github.com/google/snappy/blob/main/framing_format.txt#L34
)

func main() {
	flag.Parse()
	r := s2.NewReader(nil)

	// No args, use stdin/stdout
	args := flag.Args()
	if len(args) == 0 || *help {
		_, _ = fmt.Fprintf(os.Stderr, "s2 decompress v%v, built at %v.\n\n", version, date)
		_, _ = fmt.Fprintf(os.Stderr, "Copyright (c) 2011 The Snappy-Go Authors. All rights reserved.\n"+
			"Copyright (c) 2019+ Klaus Post. All rights reserved.\n\n")
		_, _ = fmt.Fprintln(os.Stderr, `Usage: s2d [options] file1 file2

Decompresses all files supplied as input. Input files must end with '`+s2Ext+`' or '`+snappyExt+`'.
Output file names have the extension removed. By default output files will be overwritten.
Use - as the only file name to read from stdin and write to stdout.

Wildcards are accepted: testdir/*.txt will compress all files in testdir ending with .txt
Directories can be wildcards as well. testdir/*/*.txt will match testdir/subdir/b.txt

File names beginning with 'http://' and 'https://' will be downloaded and decompressed.
Extensions on downloaded files are ignored. Only http response code 200 is accepted.

Options:`)
		flag.PrintDefaults()
		os.Exit(0)
	}
	tailBytes, err := toSize(*tail)
	exitErr(err)
	offset, err := toSize(*offset)
	exitErr(err)
	if tailBytes > 0 && offset > 0 {
		exitErr(errors.New("--offset and --tail cannot be used together"))
	}
	if len(args) == 1 && args[0] == "-" {
		r.Reset(os.Stdin)
		if *verify {
			_, err := io.Copy(io.Discard, r)
			exitErr(err)
			return
		}
		if *out == "" {
			_, err := io.Copy(os.Stdout, r)
			exitErr(err)
			return
		}
		dstFilename := *out
		if *safe {
			_, err := os.Stat(dstFilename)
			if !os.IsNotExist(err) {
				exitErr(errors.New("destination files exists"))
			}
		}
		dstFile, err := os.OpenFile(dstFilename, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, os.ModePerm)
		exitErr(err)
		defer dstFile.Close()
		bw := bufio.NewWriterSize(dstFile, 4<<20)
		defer bw.Flush()
		_, err = io.Copy(bw, r)
		exitErr(err)
		return
	}
	var files []string

	for _, pattern := range args {
		if isHTTP(pattern) {
			files = append(files, pattern)
			continue
		}

		found, err := filepathx.Glob(pattern)
		exitErr(err)
		if len(found) == 0 {
			exitErr(fmt.Errorf("unable to find file %v", pattern))
		}
		files = append(files, found...)
	}

	*quiet = *quiet || *stdout

	if *bench > 0 {
		debug.SetGCPercent(10)
		for _, filename := range files {
			block := *block
			dstFilename := cleanFileName(filename)
			if strings.HasSuffix(filename, ".block") {
				dstFilename = strings.TrimSuffix(dstFilename, ".block")
				block = true
			}
			switch {
			case strings.HasSuffix(dstFilename, s2Ext):
			case strings.HasSuffix(dstFilename, snappyExt):
			case strings.HasSuffix(dstFilename, ".snappy"):
			default:
				if !isHTTP(filename) {
					fmt.Println("Skipping", filename)
					continue
				}
			}

			func() {
				if !*quiet {
					fmt.Print("Reading ", filename, "...")
				}
				// Input file.
				file, size, _ := openFile(filename)
				b := make([]byte, size)
				_, err := io.ReadFull(file, b)
				exitErr(err)
				file.Close()

				for i := 0; i < *bench; i++ {
					if !*quiet {
						fmt.Print("\nDecompressing...")
					}
					start := time.Now()
					var output int64
					if block {
						dec, err := s2.Decode(nil, b)
						exitErr(err)
						output = int64(len(dec))
					} else {
						r.Reset(bytes.NewBuffer(b))
						if *cpu > 1 {
							output, err = r.DecodeConcurrent(io.Discard, *cpu)
						} else {
							output, err = io.Copy(io.Discard, r)
						}
						exitErr(err)
					}
					if !*quiet {
						elapsed := time.Since(start)
						ms := elapsed.Round(time.Millisecond)
						mbPerSec := (float64(output) / (1024 * 1024)) / (float64(elapsed) / (float64(time.Second)))
						pct := float64(output) * 100 / float64(len(b))
						fmt.Printf(" %d -> %d [%.02f%%]; %v, %.01fMB/s", len(b), output, pct, ms, mbPerSec)
					}
				}
				if !*quiet {
					fmt.Println("")
				}
			}()
		}
		os.Exit(0)
	}

	if *out != "" && len(files) > 1 {
		exitErr(errors.New("-out parameter can only be used with one input"))
	}

	for _, filename := range files {
		dstFilename := cleanFileName(filename)
		block := *block
		if strings.HasSuffix(dstFilename, ".block") {
			dstFilename = strings.TrimSuffix(dstFilename, ".block")
			block = true
		}
		switch {
		case *out != "":
			dstFilename = *out
		case strings.HasSuffix(dstFilename, s2Ext):
			dstFilename = strings.TrimSuffix(dstFilename, s2Ext)
		case strings.HasSuffix(dstFilename, snappyExt):
			dstFilename = strings.TrimSuffix(dstFilename, snappyExt)
		case strings.HasSuffix(dstFilename, ".snappy"):
			dstFilename = strings.TrimSuffix(dstFilename, ".snappy")
		default:
			if !isHTTP(filename) {
				fmt.Println("Skipping", filename)
				continue
			}
		}
		if *verify {
			dstFilename = "(verify)"
		}

		func() {
			var closeOnce sync.Once
			if !*quiet {
				fmt.Print("Decompressing ", filename, " -> ", dstFilename)
			}
			// Input file.
			file, _, mode := openFile(filename)
			defer closeOnce.Do(func() { file.Close() })
			var rc interface {
				io.Reader
				BytesRead() int64
			}
			if tailBytes > 0 || offset > 0 {
				rs, ok := file.(io.ReadSeeker)
				if !ok && tailBytes > 0 {
					exitErr(errors.New("cannot tail with non-seekable input"))
				}
				if ok {
					rc = &rCountSeeker{in: rs}
				} else {
					rc = &rCounter{in: file}
				}
			} else {
				rc = &rCounter{in: file}
			}
			var src io.Reader
			if !block && tailBytes == 0 && offset == 0 {
				ra, err := readahead.NewReaderSize(rc, 2, 4<<20)
				exitErr(err)
				defer ra.Close()
				src = ra
			} else {
				src = rc
			}
			if *safe {
				_, err := os.Stat(dstFilename)
				if !os.IsNotExist(err) {
					exitErr(errors.New("destination files exists"))
				}
			}
			var out io.Writer
			switch {
			case *verify:
				out = io.Discard
			case *stdout:
				out = os.Stdout
			default:
				dstFile, err := os.OpenFile(dstFilename, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, mode)
				exitErr(err)
				defer dstFile.Close()
				out = dstFile
				if !block {
					bw := bufio.NewWriterSize(dstFile, 4<<20)
					defer bw.Flush()
					out = bw
				}
			}
			var decoded io.Reader
			start := time.Now()
			if block {
				all, err := io.ReadAll(src)
				exitErr(err)
				b, err := s2.Decode(nil, all)
				exitErr(err)
				decoded = bytes.NewReader(b)
			} else {
				r.Reset(src)
				if tailBytes > 0 || offset > 0 {
					rs, err := r.ReadSeeker(tailBytes > 0, nil)
					exitErr(err)
					if tailBytes > 0 {
						_, err = rs.Seek(-tailBytes, io.SeekEnd)
					} else {
						_, err = rs.Seek(offset, io.SeekStart)
					}
					exitErr(err)
				}
				decoded = r
			}
			var err error
			var output int64
			if dec, ok := decoded.(*s2.Reader); ok && tailBytes == 0 && offset == 0 {
				output, err = dec.DecodeConcurrent(out, *cpu)
			} else {
				output, err = io.Copy(out, decoded)
			}
			exitErr(err)
			if !*quiet {
				elapsed := time.Since(start)
				mbPerSec := (float64(output) / (1024 * 1024)) / (float64(elapsed) / (float64(time.Second)))
				pct := float64(output) * 100 / float64(rc.BytesRead())
				fmt.Printf(" %d -> %d [%.02f%%]; %.01fMB/s\n", rc.BytesRead(), output, pct, mbPerSec)
			}
			if *remove && !*verify {
				closeOnce.Do(func() {
					file.Close()
					if !*quiet {
						fmt.Println("Removing", filename)
					}
					err := os.Remove(filename)
					exitErr(err)
				})
			}
		}()
	}
}

func openFile(name string) (rc io.ReadCloser, size int64, mode os.FileMode) {
	if isHTTP(name) {
		resp, err := http.Get(name)
		exitErr(err)
		if resp.StatusCode != http.StatusOK {
			exitErr(fmt.Errorf("unexpected response status code %v, want 200 OK", resp.Status))
		}
		return resp.Body, resp.ContentLength, os.ModePerm
	}
	file, err := os.Open(name)
	exitErr(err)
	st, err := file.Stat()
	exitErr(err)
	return file, st.Size(), st.Mode()
}

func cleanFileName(s string) string {
	if isHTTP(s) {
		s = strings.TrimPrefix(s, "http://")
		s = strings.TrimPrefix(s, "https://")
		s = strings.Map(func(r rune) rune {
			switch r {
			case '\\', '/', '*', '?', ':', '|', '<', '>', '~':
				return '_'
			}
			if r < 20 {
				return '_'
			}
			return r
		}, s)
	}
	return s
}

func isHTTP(name string) bool {
	return strings.HasPrefix(name, "http://") || strings.HasPrefix(name, "https://")
}

func exitErr(err error) {
	if err != nil {
		fmt.Fprintln(os.Stderr, "\nERROR:", err.Error())
		os.Exit(2)
	}
}

type rCounter struct {
	n  int64
	in io.Reader
}

func (w *rCounter) Read(p []byte) (n int, err error) {
	n, err = w.in.Read(p)
	w.n += int64(n)
	return n, err
}

func (w *rCounter) BytesRead() int64 {
	return w.n
}

type rCountSeeker struct {
	n  int64
	in io.ReadSeeker
}

func (w *rCountSeeker) Read(p []byte) (n int, err error) {
	n, err = w.in.Read(p)
	w.n += int64(n)
	return n, err
}

func (w *rCountSeeker) Seek(offset int64, whence int) (int64, error) {
	return w.in.Seek(offset, whence)
}

func (w *rCountSeeker) BytesRead() int64 {
	return w.n
}

// toSize converts a size indication to bytes.
func toSize(size string) (int64, error) {
	if len(size) == 0 {
		return 0, nil
	}
	size = strings.ToUpper(strings.TrimSpace(size))
	firstLetter := strings.IndexFunc(size, unicode.IsLetter)
	if firstLetter == -1 {
		firstLetter = len(size)
	}

	bytesString, multiple := size[:firstLetter], size[firstLetter:]
	sz, err := strconv.ParseInt(bytesString, 10, 64)
	if err != nil {
		return 0, fmt.Errorf("unable to parse size: %v", err)
	}

	if sz < 0 {
		return 0, errors.New("negative size given")
	}
	switch multiple {
	case "T", "TB", "TIB":
		return sz * 1 << 40, nil
	case "G", "GB", "GIB":
		return sz * 1 << 30, nil
	case "M", "MB", "MIB":
		return sz * 1 << 20, nil
	case "K", "KB", "KIB":
		return sz * 1 << 10, nil
	case "B", "":
		return sz, nil
	default:
		return 0, fmt.Errorf("unknown size suffix: %v", multiple)
	}
}
