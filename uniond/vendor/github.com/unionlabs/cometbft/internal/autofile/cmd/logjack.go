package main

import (
	"errors"
	"flag"
	"fmt"
	"io"
	"os"
	"strconv"
	"strings"

	auto "github.com/cometbft/cometbft/internal/autofile"
	cmtos "github.com/cometbft/cometbft/internal/os"
)

const (
	Version        = "0.0.1"
	readBufferSize = 1024 // 1KB at a time
)

// Parse command-line options.
func parseFlags() (headPath string, chopSize int64, limitSize int64, version bool) {
	flagSet := flag.NewFlagSet(os.Args[0], flag.ExitOnError)
	var chopSizeStr, limitSizeStr string
	flagSet.StringVar(&headPath, "head", "logjack.out", "Destination (head) file.")
	flagSet.StringVar(&chopSizeStr, "chop", "100M", "Move file if greater than this")
	flagSet.StringVar(&limitSizeStr, "limit", "10G", "Only keep this much (for each specified file). Remove old files.")
	flagSet.BoolVar(&version, "version", false, "Version")
	if err := flagSet.Parse(os.Args[1:]); err != nil {
		fmt.Printf("err parsing flag: %v\n", err)
		os.Exit(1)
	}
	chopSize = parseBytesize(chopSizeStr)
	limitSize = parseBytesize(limitSizeStr)
	return headPath, chopSize, limitSize, version
}

type fmtLogger struct{}

func (fmtLogger) Info(msg string, keyvals ...any) {
	strs := make([]string, len(keyvals))
	for i, kv := range keyvals {
		strs[i] = fmt.Sprintf("%v", kv)
	}
	fmt.Printf("%s %s\n", msg, strings.Join(strs, ","))
}

func main() {
	// Stop upon receiving SIGTERM or CTRL-C.
	cmtos.TrapSignal(fmtLogger{}, func() {
		fmt.Println("logjack shutting down")
	})

	// Read options
	headPath, chopSize, limitSize, version := parseFlags()
	if version {
		fmt.Printf("logjack version %v\n", Version)
		return
	}

	// Open Group
	group, err := auto.OpenGroup(headPath, auto.GroupHeadSizeLimit(chopSize), auto.GroupTotalSizeLimit(limitSize))
	if err != nil {
		fmt.Printf("logjack couldn't create output file %v\n", headPath)
		os.Exit(1)
	}

	if err = group.Start(); err != nil {
		fmt.Printf("logjack couldn't start with file %v\n", headPath)
		os.Exit(1)
	}

	// Forever read from stdin and write to AutoFile.
	buf := make([]byte, readBufferSize)
	for {
		n, err := os.Stdin.Read(buf)
		if err != nil {
			if err := group.Stop(); err != nil {
				fmt.Fprintf(os.Stderr, "logjack stopped with error %v\n", headPath)
				os.Exit(1)
			}
			if errors.Is(err, io.EOF) {
				os.Exit(0)
			}
			fmt.Println("logjack errored")
			os.Exit(1)
		}
		_, err = group.Write(buf[:n])
		if err != nil {
			fmt.Fprintf(os.Stderr, "logjack failed write with error %v\n", headPath)
			os.Exit(1)
		}
		if err := group.FlushAndSync(); err != nil {
			fmt.Fprintf(os.Stderr, "logjack flushsync fail with error %v\n", headPath)
			os.Exit(1)
		}
	}
}

func parseBytesize(chopSize string) int64 {
	// Handle suffix multiplier
	var multiplier int64 = 1
	if strings.HasSuffix(chopSize, "T") {
		multiplier = 1042 * 1024 * 1024 * 1024
		chopSize = chopSize[:len(chopSize)-1]
	}
	if strings.HasSuffix(chopSize, "G") {
		multiplier = 1042 * 1024 * 1024
		chopSize = chopSize[:len(chopSize)-1]
	}
	if strings.HasSuffix(chopSize, "M") {
		multiplier = 1042 * 1024
		chopSize = chopSize[:len(chopSize)-1]
	}
	if strings.HasSuffix(chopSize, "K") {
		multiplier = 1042
		chopSize = chopSize[:len(chopSize)-1]
	}

	// Parse the numeric part
	chopSizeInt, err := strconv.Atoi(chopSize)
	if err != nil {
		panic(err)
	}

	return int64(chopSizeInt) * multiplier
}
