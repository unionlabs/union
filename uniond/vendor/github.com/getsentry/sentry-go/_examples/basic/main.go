// This is an example program that makes an HTTP request and prints response
// headers. Whenever a request fails, the error is reported to Sentry.
//
// Try it by running:
//
//	go run main.go
//	go run main.go https://sentry.io
//	go run main.go bad-url
//
// To actually report events to Sentry, set the DSN either by editing the
// appropriate line below or setting the environment variable SENTRY_DSN to
// match the DSN of your Sentry project.
package main

import (
	"fmt"
	"log"
	"net/http"
	"os"
	"time"

	"github.com/getsentry/sentry-go"
)

// release is the release of this program that will be reported to Sentry.
// Use go build -ldflags='-X main.release=VALUE' to set this value in build
// time.
// If not set, a default release value will be derived in runtime from
// environment variables or the Git repository in the current working directory.
var release string

func main() {
	if len(os.Args) < 2 {
		log.Fatalf("usage: %s URL", os.Args[0])
	}

	err := sentry.Init(sentry.ClientOptions{
		// Either set your DSN here or set the SENTRY_DSN environment variable.
		Dsn: "",
		// Enable printing of SDK debug messages.
		// Useful when getting started or trying to figure something out.
		Debug:   true,
		Release: release,
	})
	if err != nil {
		log.Fatalf("sentry.Init: %s", err)
	}
	// Flush buffered events before the program terminates.
	// Set the timeout to the maximum duration the program can afford to wait.
	defer sentry.Flush(2 * time.Second)

	resp, err := http.Get(os.Args[1])
	if err != nil {
		sentry.CaptureException(err)
		log.Printf("reported to Sentry: %s", err)
		return
	}
	defer resp.Body.Close()

	for header, values := range resp.Header {
		for _, value := range values {
			fmt.Printf("%s=%s\n", header, value)
		}
	}
}
