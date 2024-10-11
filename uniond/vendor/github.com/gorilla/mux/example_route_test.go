package mux_test

import (
	"fmt"
	"net/http"

	"github.com/gorilla/mux"
)

// This example demonstrates setting a regular expression matcher for
// the header value. A plain word will match any value that contains a
// matching substring as if the pattern was wrapped with `.*`.
func ExampleRoute_HeadersRegexp() {
	r := mux.NewRouter()
	route := r.NewRoute().HeadersRegexp("Accept", "html")

	req1, _ := http.NewRequest("GET", "example.com", nil)
	req1.Header.Add("Accept", "text/plain")
	req1.Header.Add("Accept", "text/html")

	req2, _ := http.NewRequest("GET", "example.com", nil)
	req2.Header.Set("Accept", "application/xhtml+xml")

	matchInfo := &mux.RouteMatch{}
	fmt.Printf("Match: %v %q\n", route.Match(req1, matchInfo), req1.Header["Accept"])
	fmt.Printf("Match: %v %q\n", route.Match(req2, matchInfo), req2.Header["Accept"])
	// Output:
	// Match: true ["text/plain" "text/html"]
	// Match: true ["application/xhtml+xml"]
}

// This example demonstrates setting a strict regular expression matcher
// for the header value. Using the start and end of string anchors, the
// value must be an exact match.
func ExampleRoute_HeadersRegexp_exactMatch() {
	r := mux.NewRouter()
	route := r.NewRoute().HeadersRegexp("Origin", "^https://example.co$")

	yes, _ := http.NewRequest("GET", "example.co", nil)
	yes.Header.Set("Origin", "https://example.co")

	no, _ := http.NewRequest("GET", "example.co.uk", nil)
	no.Header.Set("Origin", "https://example.co.uk")

	matchInfo := &mux.RouteMatch{}
	fmt.Printf("Match: %v %q\n", route.Match(yes, matchInfo), yes.Header["Origin"])
	fmt.Printf("Match: %v %q\n", route.Match(no, matchInfo), no.Header["Origin"])
	// Output:
	// Match: true ["https://example.co"]
	// Match: false ["https://example.co.uk"]
}
