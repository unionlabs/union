// Copyright 2023 Google LLC
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

package main

import (
	"bytes"
	"flag"
	"fmt"
	"io"
	"os"
	"strings"
	"text/template"

	"github.com/klauspost/compress/zip"
)

var (
	dictPath   = flag.String("dict-path", "", "dict path")
	outputFile = flag.String("output-file", "", "output file")
)

func main() {
	flag.Parse()
	if *dictPath == "" {
		panic("Need a dict path")
	}
	if *outputFile == "" {
		panic("Need an output file")
	}
	dicts := getFuzzDicts(*dictPath)

	t, err := template.New("todos").Parse(`
package zstd
var fuzzDicts = make([][]byte, 0)
func init() {
{{range $val := .}}
    fuzzDicts = append(fuzzDicts, {{$val}})
{{end}}
}
`)
	if err != nil {
		panic(err)
	}
	f, err := os.Create(*outputFile)
	err = t.Execute(f, dicts)
	if err != nil {
		panic(err)
	}
	f.Close()
}

func getFuzzDicts(path string) []string {
	data, err := os.ReadFile(path)
	if err != nil {
		panic(err)
	}
	zr, err := zip.NewReader(bytes.NewReader(data), int64(len(data)))
	if err != nil {
		panic(err)
	}
	var dicts [][]byte
	for _, tt := range zr.File {
		if !strings.HasSuffix(tt.Name, ".dict") {
			continue
		}
		func() {
			r, err := tt.Open()
			if err != nil {
				panic(err)
			}
			defer r.Close()
			in, err := io.ReadAll(r)
			if err != nil {
				panic(err)
			}
			dicts = append(dicts, in)
		}()
	}
	stringDicts := make([]string, 0)
	for _, d := range dicts {
		stringedArray := fmt.Sprintf("%v", d)
		withComma := strings.Replace(stringedArray, " ", ", ", -1)
		withClosingBracket := strings.Replace(withComma, "]", "}", -1)
		withOpenBracket := strings.Replace(withClosingBracket, "[", "[]byte{", -1)
		stringDicts = append(stringDicts, withOpenBracket)
	}
	return stringDicts
}
