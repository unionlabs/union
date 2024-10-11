// Package main produces a test-binary used in tests.
package main

import (
	"flag"
	"fmt"
	"os"
	"time"
)

func main() {
	sleep := flag.Duration("sleep", 0, "Sleep")
	warn := flag.Bool("warn", false, "Warn")
	fail := flag.Int("fail", 0, "Fail with code")
	flag.Parse()

	if *sleep != 0 {
		time.Sleep(*sleep)
	}

	fmt.Println("this is stdout")
	if *warn {
		fmt.Fprintln(os.Stderr, "this is stderr")
	}

	os.Exit(*fail)
}
