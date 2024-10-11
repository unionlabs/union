// Copyright 2022 The Prometheus Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//go:build go1.17
// +build go1.17

// A minimal example of how to include Prometheus instrumentation.
package main

import (
	"flag"
	"fmt"
	"log"
	"net/http"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/collectors/version"
	"github.com/prometheus/client_golang/prometheus/promhttp"
)

var addr = flag.String("listen-address", ":8080", "The address to listen on for HTTP requests.")

// Build using ldflags, for example:
// go build -ldflags "-X github.com/prometheus/common/version.Version=1.0.0 -X github.com/prometheus/common/version.Branch=abc123" .
func main() {
	flag.Parse()

	// Create a new registry.
	reg := prometheus.NewRegistry()

	// Register version collector.
	reg.MustRegister(version.NewCollector("example"))

	// Expose the registered metrics via HTTP.
	http.Handle("/metrics", promhttp.HandlerFor(
		reg,
		promhttp.HandlerOpts{},
	))
	fmt.Println("Hello world from new Version Collector!")
	log.Fatal(http.ListenAndServe(*addr, nil))
}
