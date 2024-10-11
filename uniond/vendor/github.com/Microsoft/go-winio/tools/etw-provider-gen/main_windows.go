//go:build windows
// +build windows

package main

import (
	"flag"
	"fmt"
	"os"

	"github.com/Microsoft/go-winio/pkg/etw"
)

func main() {
	var pn = flag.String("provider-name", "", "The human readable ETW provider name to be converted into GUID format")
	flag.Parse()
	if pn == nil || *pn == "" {
		fmt.Fprint(os.Stderr, "--provider-name is required")
		os.Exit(1)
	}
	p, err := etw.NewProvider(*pn, nil)
	if err != nil {
		fmt.Fprintf(os.Stderr, "failed to convert provider-name: '%s' with err: '%s", *pn, err)
		os.Exit(1)
	}
	defer p.Close()
	fmt.Fprintf(os.Stdout, "%s", p)
}
