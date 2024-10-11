package main

import (
	"bufio"
	"bytes"
	"errors"
	"flag"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"os/signal"
	"runtime"
	"runtime/debug"
	"runtime/pprof"
	"runtime/trace"
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
	faster    = flag.Bool("faster", false, "Compress faster, but with a minor compression loss")
	slower    = flag.Bool("slower", false, "Compress more, but a lot slower")
	snappy    = flag.Bool("snappy", false, "Generate Snappy compatible output stream")
	recomp    = flag.Bool("recomp", false, "Recompress Snappy or S2 input")
	cpu       = flag.Int("cpu", runtime.GOMAXPROCS(0), "Compress using this amount of threads")
	blockSize = flag.String("blocksize", "4M", "Max  block size. Examples: 64K, 256K, 1M, 4M. Must be power of two and <= 4MB")
	block     = flag.Bool("block", false, "Compress as a single block. Will load content into memory.")
	safe      = flag.Bool("safe", false, "Do not overwrite output files")
	index     = flag.Bool("index", true, "Add seek index")
	padding   = flag.String("pad", "1", "Pad size to a multiple of this value, Examples: 500, 64K, 256K, 1M, 4M, etc")
	stdout    = flag.Bool("c", false, "Write all output to stdout. Multiple input files will be concatenated")
	out       = flag.String("o", "", "Write output to another file. Single input file only")
	remove    = flag.Bool("rm", false, "Delete source file(s) after successful compression")
	quiet     = flag.Bool("q", false, "Don't write any output to terminal, except errors")
	bench     = flag.Int("bench", 0, "Run benchmark n times. No output will be written")
	verify    = flag.Bool("verify", false, "Verify written files")
	help      = flag.Bool("help", false, "Display help")

	cpuprofile, memprofile, traceprofile string

	version = "(dev)"
	date    = "(unknown)"
)

const (
	s2Ext     = ".s2"
	snappyExt = ".sz" // https://github.com/google/snappy/blob/main/framing_format.txt#L34
)

func main() {
	if false {
		flag.StringVar(&cpuprofile, "cpuprofile", "", "write cpu profile to file")
		flag.StringVar(&memprofile, "memprofile", "", "write mem profile to file")
		flag.StringVar(&traceprofile, "traceprofile", "", "write trace profile to file")
	}
	flag.Parse()
	sz, err := toSize(*blockSize)
	exitErr(err)
	pad, err := toSize(*padding)
	exitErr(err)

	args := flag.Args()
	if len(args) == 0 || *help || (*slower && *faster) {
		_, _ = fmt.Fprintf(os.Stderr, "s2 compress v%v, built at %v.\n\n", version, date)
		_, _ = fmt.Fprintf(os.Stderr, "Copyright (c) 2011 The Snappy-Go Authors. All rights reserved.\n"+
			"Copyright (c) 2019+ Klaus Post. All rights reserved.\n\n")
		_, _ = fmt.Fprintln(os.Stderr, `Usage: s2c [options] file1 file2

Compresses all files supplied as input separately.
Output files are written as 'filename.ext`+s2Ext+`' or 'filename.ext`+snappyExt+`'.
By default output files will be overwritten.
Use - as the only file name to read from stdin and write to stdout.

Wildcards are accepted: testdir/*.txt will compress all files in testdir ending with .txt
Directories can be wildcards as well. testdir/*/*.txt will match testdir/subdir/b.txt

File names beginning with 'http://' and 'https://' will be downloaded and compressed.
Only http response code 200 is accepted.

Options:`)
		flag.PrintDefaults()
		os.Exit(0)
	}
	opts := []s2.WriterOption{s2.WriterBlockSize(sz), s2.WriterConcurrency(*cpu), s2.WriterPadding(pad)}
	if *index {
		opts = append(opts, s2.WriterAddIndex())
	}
	if !*faster {
		opts = append(opts, s2.WriterBetterCompression())
	}
	if *slower {
		opts = append(opts, s2.WriterBestCompression())
	}
	if *snappy {
		opts = append(opts, s2.WriterSnappyCompat())
	}
	wr := s2.NewWriter(nil, opts...)

	// No args, use stdin/stdout
	if len(args) == 1 && args[0] == "-" {
		// Catch interrupt, so we don't exit at once.
		// os.Stdin will return EOF, so we should be able to get everything.
		signal.Notify(make(chan os.Signal, 1), os.Interrupt)
		if len(*out) == 0 {
			wr.Reset(os.Stdout)
		} else {
			if *safe {
				_, err := os.Stat(*out)
				if !os.IsNotExist(err) {
					exitErr(errors.New("destination file exists"))
				}
			}
			dstFile, err := os.OpenFile(*out, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, os.ModePerm)
			exitErr(err)
			defer dstFile.Close()
			bw := bufio.NewWriterSize(dstFile, sz*2)
			defer bw.Flush()
			wr.Reset(bw)
		}
		_, err = wr.ReadFrom(os.Stdin)
		printErr(err)
		printErr(wr.Close())
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
	if cpuprofile != "" {
		f, err := os.Create(cpuprofile)
		if err != nil {
			log.Fatal(err)
		}
		pprof.StartCPUProfile(f)
		defer pprof.StopCPUProfile()
	}
	if memprofile != "" {
		f, err := os.Create(memprofile)
		if err != nil {
			log.Fatal(err)
		}
		defer f.Close()
		defer pprof.WriteHeapProfile(f)
	}
	if traceprofile != "" {
		f, err := os.Create(traceprofile)
		if err != nil {
			log.Fatal(err)
		}
		defer f.Close()
		err = trace.Start(f)
		if err != nil {
			log.Fatal(err)
		}
		defer trace.Stop()
	}

	*quiet = *quiet || *stdout
	if *bench > 0 {
		debug.SetGCPercent(10)
		dec := s2.NewReader(nil)
		for _, filename := range files {
			if *block {
				func() {
					if !*quiet {
						fmt.Print("Reading ", filename, "...")
					}
					// Input file.
					file, size, _ := openFile(filename)
					b := make([]byte, size)
					_, err = io.ReadFull(file, b)
					exitErr(err)
					file.Close()
					for i := 0; i < *bench; i++ {
						if !*quiet {
							fmt.Print("\nCompressing...")
						}
						start := time.Now()
						var compressed []byte
						switch {
						case *faster:
							if *snappy {
								compressed = s2.EncodeSnappy(nil, b)
								break
							}
							compressed = s2.Encode(nil, b)
						case *slower:
							if *snappy {
								compressed = s2.EncodeSnappyBest(nil, b)
								break
							}
							compressed = s2.EncodeBest(nil, b)
						default:
							if *snappy {
								compressed = s2.EncodeSnappyBetter(nil, b)
								break
							}
							compressed = s2.EncodeBetter(nil, b)
						}
						exitErr(err)
						err = wr.Close()
						exitErr(err)
						if !*quiet {
							input := len(b)
							elapsed := time.Since(start)
							mbpersec := (float64(input) / (1024 * 1024)) / (float64(elapsed) / (float64(time.Second)))
							pct := float64(len(compressed)) * 100 / float64(input)
							ms := elapsed.Round(time.Millisecond)
							fmt.Printf(" %d -> %d [%.02f%%]; %v, %.01fMB/s", input, len(compressed), pct, ms, mbpersec)
						}
						if *verify {
							if !*quiet {
								fmt.Print("\nDecompressing.")
							}
							decomp := make([]byte, 0, len(b))
							start := time.Now()
							decomp, err = s2.Decode(decomp, compressed)
							exitErr(err)
							if len(decomp) != len(b) {
								exitErr(fmt.Errorf("unexpected size, want %d, got %d", len(b), len(decomp)))
							}
							if !*quiet {
								input := len(b)
								elapsed := time.Since(start)
								mbpersec := (float64(input) / (1024 * 1024)) / (float64(elapsed) / (float64(time.Second)))
								pct := float64(input) * 100 / float64(len(compressed))
								ms := elapsed.Round(time.Millisecond)
								fmt.Printf(" %d -> %d [%.02f%%]; %v, %.01fMB/s", len(compressed), len(decomp), pct, ms, mbpersec)
							}
							if !bytes.Equal(decomp, b) {
								exitErr(fmt.Errorf("decompressed data mismatch"))
							}
							if !*quiet {
								fmt.Print("... Verified ok.")
							}
						}
					}
					if !*quiet {
						fmt.Println("")
					}
					wr.Close()
				}()
				continue
			}
			func() {
				if !*quiet {
					fmt.Print("Reading ", filename, "...")
				}
				// Input file.
				file, size, _ := openFile(filename)
				b := make([]byte, size)
				_, err = io.ReadFull(file, b)
				exitErr(err)
				file.Close()
				var buf *bytes.Buffer
				for i := 0; i < *bench; i++ {
					w := io.Discard
					// Verify with this buffer...
					if *verify {
						if buf == nil {
							buf = bytes.NewBuffer(make([]byte, 0, len(b)+(len(b)>>8)))
						}
						buf.Reset()
						w = buf
					}
					wc := wCounter{out: w}
					if !*quiet {
						fmt.Print("\nCompressing...")
					}
					wr.Reset(&wc)
					start := time.Now()
					err := wr.EncodeBuffer(b)
					exitErr(err)
					err = wr.Close()
					exitErr(err)
					if !*quiet {
						input := len(b)
						elapsed := time.Since(start)
						mbpersec := (float64(input) / (1024 * 1024)) / (float64(elapsed) / (float64(time.Second)))
						pct := float64(wc.n) * 100 / float64(input)
						ms := elapsed.Round(time.Millisecond)
						fmt.Printf(" %d -> %d [%.02f%%]; %v, %.01fMB/s", input, wc.n, pct, ms, mbpersec)
					}
					if *verify {
						if !*quiet {
							fmt.Print("\nDecompressing.")
						}
						start := time.Now()
						dec.Reset(buf)
						n, err := dec.DecodeConcurrent(io.Discard, *cpu)
						exitErr(err)
						if int(n) != len(b) {
							exitErr(fmt.Errorf("unexpected size, want %d, got %d", len(b), n))
						}
						if !*quiet {
							input := len(b)
							elapsed := time.Since(start)
							mbpersec := (float64(input) / (1024 * 1024)) / (float64(elapsed) / (float64(time.Second)))
							pct := float64(input) * 100 / float64(wc.n)
							ms := elapsed.Round(time.Millisecond)
							fmt.Printf(" %d -> %d [%.02f%%]; %v, %.01fMB/s", wc.n, n, pct, ms, mbpersec)
						}
						dec.Reset(nil)
					}
				}
				if !*quiet {
					fmt.Println("")
				}
				wr.Close()
			}()
		}
		os.Exit(0)
	}
	ext := s2Ext
	if *snappy {
		ext = snappyExt
	}
	if *block {
		ext += ".block"
	}
	if *out != "" && len(files) > 1 {
		exitErr(errors.New("-out parameter can only be used with one input"))
	}
	for _, filename := range files {
		if *block {
			if *recomp {
				exitErr(errors.New("cannot recompress blocks (yet)"))
			}
			func() {
				var closeOnce sync.Once
				dstFilename := cleanFileName(fmt.Sprintf("%s%s", filename, ext))
				if *out != "" {
					dstFilename = *out
				}
				if !*quiet {
					fmt.Print("Compressing ", filename, " -> ", dstFilename)
				}
				// Input file.
				file, _, mode := openFile(filename)
				exitErr(err)
				defer closeOnce.Do(func() { file.Close() })
				inBytes, err := io.ReadAll(file)
				exitErr(err)

				var out io.Writer
				switch {
				case *stdout:
					out = os.Stdout
				default:
					if *safe {
						_, err := os.Stat(dstFilename)
						if !os.IsNotExist(err) {
							exitErr(errors.New("destination file exists"))
						}
					}
					dstFile, err := os.OpenFile(dstFilename, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, mode)
					exitErr(err)
					defer dstFile.Close()
					out = dstFile
				}
				start := time.Now()
				var compressed []byte
				switch {
				case *faster:
					if *snappy {
						compressed = s2.EncodeSnappy(nil, inBytes)
						break
					}
					compressed = s2.Encode(nil, inBytes)
				case *slower:
					if *snappy {
						compressed = s2.EncodeSnappyBest(nil, inBytes)
						break
					}
					compressed = s2.EncodeBest(nil, inBytes)
				default:
					if *snappy {
						compressed = s2.EncodeSnappyBetter(nil, inBytes)
						break
					}
					compressed = s2.EncodeBetter(nil, inBytes)
				}
				_, err = out.Write(compressed)
				exitErr(err)
				if !*quiet {
					elapsed := time.Since(start)
					mbpersec := (float64(len(inBytes)) / (1024 * 1024)) / (float64(elapsed) / (float64(time.Second)))
					pct := float64(len(compressed)) * 100 / float64(len(inBytes))
					fmt.Printf(" %d -> %d [%.02f%%]; %.01fMB/s\n", len(inBytes), len(compressed), pct, mbpersec)
				}
				if *verify {
					got, err := s2.Decode(make([]byte, 0, len(inBytes)), compressed)
					exitErr(err)
					if !bytes.Equal(got, inBytes) {
						exitErr(fmt.Errorf("decoded content mismatch"))
					}
					if !*quiet {
						fmt.Print("... Verified ok.")
					}
				}
				if *remove {
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
			continue
		}
		func() {
			var closeOnce sync.Once
			outFileBase := filename
			if *recomp {
				switch {
				case strings.HasSuffix(outFileBase, s2Ext):
					outFileBase = strings.TrimSuffix(outFileBase, s2Ext)
				case strings.HasSuffix(outFileBase, snappyExt):
					outFileBase = strings.TrimSuffix(outFileBase, snappyExt)
				case strings.HasSuffix(outFileBase, ".snappy"):
					outFileBase = strings.TrimSuffix(outFileBase, ".snappy")
				}
			}
			dstFilename := cleanFileName(fmt.Sprintf("%s%s", outFileBase, ext))
			if *out != "" {
				dstFilename = *out
			}
			if !*quiet {
				fmt.Print("Compressing ", filename, " -> ", dstFilename)
			}

			if dstFilename == filename && !*stdout {
				if *remove {
					exitErr(errors.New("cannot remove when input = output"))
				}
				renameDst := dstFilename
				dstFilename = fmt.Sprintf(".tmp-%s%s", time.Now().Format("2006-01-02T15-04-05Z07"), ext)
				defer func() {
					exitErr(os.Rename(dstFilename, renameDst))
				}()
			}

			// Input file.
			file, _, mode := openFile(filename)
			exitErr(err)
			defer closeOnce.Do(func() { file.Close() })
			src, err := readahead.NewReaderSize(file, *cpu+1, 1<<20)
			exitErr(err)
			defer src.Close()
			var rc = &rCounter{
				in: src,
			}
			if !*quiet {
				// We only need to count for printing
				src = rc
			}
			if *recomp {
				dec := s2.NewReader(src)
				pr, pw := io.Pipe()
				go func() {
					_, err := dec.DecodeConcurrent(pw, *cpu)
					pw.CloseWithError(err)
				}()
				src = pr
			}

			var out io.Writer
			switch {
			case *stdout:
				out = os.Stdout
			default:
				if *safe {
					_, err := os.Stat(dstFilename)
					if !os.IsNotExist(err) {
						exitErr(errors.New("destination file exists"))
					}
				}
				dstFile, err := os.OpenFile(dstFilename, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, mode)
				exitErr(err)
				defer dstFile.Close()
				bw := bufio.NewWriterSize(dstFile, int(sz)*2)
				defer bw.Flush()
				out = bw
			}
			out, errFn := verifyTo(out)
			wc := wCounter{out: out}
			wr.Reset(&wc)
			defer wr.Close()
			start := time.Now()
			_, err = wr.ReadFrom(src)
			exitErr(err)
			err = wr.Close()
			exitErr(err)
			if !*quiet {
				input := rc.n
				elapsed := time.Since(start)
				mbpersec := (float64(input) / (1024 * 1024)) / (float64(elapsed) / (float64(time.Second)))
				pct := float64(wc.n) * 100 / float64(input)
				fmt.Printf(" %d -> %d [%.02f%%]; %.01fMB/s\n", input, wc.n, pct, mbpersec)
			}
			exitErr(errFn())
			if *remove {
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

func isHTTP(name string) bool {
	return strings.HasPrefix(name, "http://") || strings.HasPrefix(name, "https://")
}

func openFile(name string) (rc io.ReadCloser, size int64, mode os.FileMode) {
	if isHTTP(name) {
		resp, err := http.Get(name)
		exitErr(err)
		if resp.StatusCode != http.StatusOK {
			exitErr(fmt.Errorf("unexpected response status code %v, want OK", resp.Status))
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

func verifyTo(w io.Writer) (io.Writer, func() error) {
	if !*verify {
		return w, func() error {
			return nil
		}
	}
	pr, pw := io.Pipe()
	writer := io.MultiWriter(w, pw)
	var wg sync.WaitGroup
	var err error
	wg.Add(1)
	go func() {
		defer wg.Done()
		r := s2.NewReader(pr)
		_, err = r.DecodeConcurrent(io.Discard, *cpu)
		pr.CloseWithError(fmt.Errorf("verify: %w", err))
	}()
	return writer, func() error {
		pw.Close()
		wg.Wait()
		if err == nil {
			if !*quiet {
				fmt.Print("... Verified ok.")
			}
		}
		return err
	}
}

func printErr(err error) {
	if err != nil {
		fmt.Fprintln(os.Stderr, "\nERROR:", err.Error())
	}
}

func exitErr(err error) {
	if err != nil {
		fmt.Fprintln(os.Stderr, "\nERROR:", err.Error())
		os.Exit(2)
	}
}

// toSize converts a size indication to bytes.
func toSize(size string) (int, error) {
	size = strings.ToUpper(strings.TrimSpace(size))
	firstLetter := strings.IndexFunc(size, unicode.IsLetter)
	if firstLetter == -1 {
		firstLetter = len(size)
	}

	bytesString, multiple := size[:firstLetter], size[firstLetter:]
	sz, err := strconv.Atoi(bytesString)
	if err != nil {
		return 0, fmt.Errorf("unable to parse size: %v", err)
	}

	switch multiple {
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

type wCounter struct {
	n   int
	out io.Writer
}

func (w *wCounter) Write(p []byte) (n int, err error) {
	n, err = w.out.Write(p)
	w.n += n
	return n, err

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

func (w *rCounter) Close() (err error) {
	return nil
}
