/*
 * Copyright 2017 Dgraph Labs, Inc. and Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package main

import (
	"fmt"
	"net/http"
	_ "net/http/pprof" //nolint:gosec
	"runtime"

	"github.com/dustin/go-humanize"
	"go.opencensus.io/zpages"

	"github.com/dgraph-io/badger/v4/badger/cmd"
	"github.com/dgraph-io/ristretto/z"
)

func main() {
	go func() {
		for i := 8080; i < 9080; i++ {
			fmt.Printf("Listening for /debug HTTP requests at port: %d\n", i)
			if err := http.ListenAndServe(fmt.Sprintf("0.0.0.0:%d", i), nil); err != nil {
				fmt.Println("Port busy. Trying another one...")
				continue

			}
		}
	}()
	zpages.Handle(nil, "/z")
	runtime.SetBlockProfileRate(100)
	runtime.GOMAXPROCS(128)

	out := z.CallocNoRef(1, "Badger.Main")
	fmt.Printf("jemalloc enabled: %v\n", len(out) > 0)
	z.StatsPrint()
	z.Free(out)

	cmd.Execute()
	fmt.Printf("Num Allocated Bytes at program end: %s\n",
		humanize.IBytes(uint64(z.NumAllocBytes())))
	if z.NumAllocBytes() > 0 {
		fmt.Println(z.Leaks())
	}
}
