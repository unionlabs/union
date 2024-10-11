// Copyright 2018 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// httpr is a proxy that can record or replay HTTP requests.
// Start httpr with either the -record or -replay flags, providing a filename.
// Terminate the process with an interrupt (kill -2) to write the log file when recording.
// To get the CA certificate of the proxy, issue a GET to http://localhost:CP/authority.cer, where
// CP is the control port.

package main

import (
	"flag"
	"fmt"
	"io"
	"log"
	"net"
	"net/http"
	"os"
	"os/signal"
	"strings"

	"cloud.google.com/go/httpreplay/internal/proxy"
	"github.com/google/martian/v3/martianhttp"
)

var (
	port          = flag.Int("port", 8080, "port of the proxy")
	controlPort   = flag.Int("control-port", 8181, "port for controlling the proxy")
	record        = flag.String("record", "", "record traffic and save to filename")
	replay        = flag.String("replay", "", "read filename and replay traffic")
	debugHeaders  = flag.Bool("debug-headers", false, "log header mismatches")
	ignoreHeaders repeatedString
)

func main() {
	flag.Var(&ignoreHeaders, "ignore-header", "header key(s) to ignore when matching")

	flag.Parse()
	if *record == "" && *replay == "" {
		log.Fatal("provide either -record or -replay")
	}
	if *record != "" && *replay != "" {
		log.Fatal("provide only one of -record and -replay")
	}
	log.Printf("httpr: starting proxy on port %d and control on port %d", *port, *controlPort)

	var pr *proxy.Proxy
	var err error
	if *record != "" {
		pr, err = proxy.ForRecording(*record, *port)
	} else {
		pr, err = proxy.ForReplaying(*replay, *port)
	}
	if err != nil {
		log.Fatal(err)
	}
	proxy.DebugHeaders = *debugHeaders
	for _, key := range ignoreHeaders {
		pr.IgnoreHeader(key)
	}

	// Expose handlers on the control port.
	mux := http.NewServeMux()
	mux.Handle("/authority.cer", martianhttp.NewAuthorityHandler(pr.CACert))
	mux.HandleFunc("/initial", handleInitial(pr))
	lControl, err := net.Listen("tcp", fmt.Sprintf(":%d", *controlPort))
	if err != nil {
		log.Fatal(err)
	}
	go http.Serve(lControl, mux)

	sigc := make(chan os.Signal, 1)
	signal.Notify(sigc, os.Interrupt)

	<-sigc

	log.Println("httpr: shutting down")
	if err := pr.Close(); err != nil {
		log.Fatal(err)
	}
}

func handleInitial(pr *proxy.Proxy) http.HandlerFunc {
	return func(w http.ResponseWriter, req *http.Request) {
		switch req.Method {
		case "GET":
			if pr.Initial != nil {
				w.Write(pr.Initial)
			}

		case "POST":
			bytes, err := io.ReadAll(req.Body)
			req.Body.Close()
			if err != nil {
				w.WriteHeader(http.StatusInternalServerError)
				fmt.Fprintf(w, "reading body: %v", err)
			}
			pr.Initial = bytes

		default:
			w.WriteHeader(http.StatusBadRequest)
			fmt.Fprint(w, "use GET to retrieve initial or POST to set it")
		}
	}
}

type repeatedString []string

func (i *repeatedString) String() string {
	v := make([]string, 0)
	if i != nil {
		v = *i
	}
	return strings.Join(v, ",")
}

func (i *repeatedString) Set(value string) error {
	*i = append(*i, value)
	return nil
}
