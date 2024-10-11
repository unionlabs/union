// Copyright (c) 2021 Klaus Post. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package gzhttp_test

import (
	"fmt"
	"io"
	"io/ioutil"
	"log"
	"net/http"
	"net/http/httptest"

	"github.com/klauspost/compress/gzhttp"
	"github.com/klauspost/compress/gzip"
)

func ExampleTransport() {
	// Get a client.
	client := http.Client{
		// Wrap the transport:
		Transport: gzhttp.Transport(http.DefaultTransport),
	}

	resp, err := client.Get("https://google.com")
	if err != nil {
		fmt.Println(err)
		return
	}
	defer resp.Body.Close()

	body, _ := ioutil.ReadAll(resp.Body)
	fmt.Println("body:", string(body))
}

func ExampleNewWrapper() {
	handler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "text/plain")
		io.WriteString(w, "Hello, World, Welcome to the jungle...")
	})
	handler2 := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		io.WriteString(w, "Hello, Another World.................")
	})

	// Create a reusable wrapper with custom options.
	wrapper, err := gzhttp.NewWrapper(gzhttp.MinSize(20), gzhttp.CompressionLevel(gzip.BestSpeed))
	if err != nil {
		log.Fatalln(err)
	}
	server := http.NewServeMux()
	server.Handle("/a", wrapper(handler))
	server.Handle("/b", wrapper(handler2))

	test := httptest.NewServer(server)
	defer test.Close()

	resp, err := http.Get(test.URL + "/a")
	if err != nil {
		log.Fatalln(err)
	}
	content, _ := ioutil.ReadAll(resp.Body)
	fmt.Println(string(content))

	resp, err = http.Get(test.URL + "/b")
	if err != nil {
		log.Fatalln(err)
	}
	content, _ = ioutil.ReadAll(resp.Body)
	fmt.Println(string(content))
	// Output:
	// Hello, World, Welcome to the jungle...
	// Hello, Another World.................
}
