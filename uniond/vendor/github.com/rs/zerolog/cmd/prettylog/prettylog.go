package main

import (
	"bufio"
	"errors"
	"flag"
	"fmt"
	"io"
	"os"
	"time"

	"github.com/rs/zerolog"
)

func isInputFromPipe() bool {
	fileInfo, _ := os.Stdin.Stat()
	return fileInfo.Mode()&os.ModeCharDevice == 0
}

func processInput(reader io.Reader, writer io.Writer) error {
	scanner := bufio.NewScanner(reader)
	for scanner.Scan() {
		bytesToWrite := scanner.Bytes()
		_, err := writer.Write(bytesToWrite)
		if err != nil {
			if errors.Is(err, io.EOF) {
				break
			}

			fmt.Printf("%s\n", bytesToWrite)
		}
	}

	return scanner.Err()
}

func main() {
	timeFormats := map[string]string{
		"default": time.Kitchen,
		"full":    time.RFC1123,
	}

	timeFormatFlag := flag.String(
		"time-format",
		"default",
		"Time format, either 'default' or 'full'",
	)

	flag.Parse()

	timeFormat, ok := timeFormats[*timeFormatFlag]
	if !ok {
		panic("Invalid time-format provided")
	}

	writer := zerolog.NewConsoleWriter()
	writer.TimeFormat = timeFormat

	if isInputFromPipe() {
		_ = processInput(os.Stdin, writer)
	} else if flag.NArg() >= 1 {
		for _, filename := range flag.Args() {
			// Scan each line from filename and write it into writer
			reader, err := os.Open(filename)
			if err != nil {
				fmt.Printf("%s open: %v", filename, err)
				os.Exit(1)
			}

			if err := processInput(reader, writer); err != nil {
				fmt.Printf("%s scan: %v", filename, err)
				os.Exit(1)
			}
		}
	} else {
		fmt.Println("Usage:")
		fmt.Println("  app_with_zerolog | 2> >(prettylog)")
		fmt.Println("  prettylog zerolog_output.jsonl")
		os.Exit(1)
		return
	}
}
