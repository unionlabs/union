// Copyright 2022 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package main

import (
	"bytes"
	"fmt"
	"go/ast"
	"go/importer"
	"go/parser"
	"go/token"
	"go/types"
	"os"
	"path/filepath"
	"testing"
)

// Package RLP is loaded only once and reused for all tests.
var (
	testFset       = token.NewFileSet()
	testImporter   = importer.ForCompiler(testFset, "source", nil).(types.ImporterFrom)
	testPackageRLP *types.Package
)

func init() {
	cwd, err := os.Getwd()
	if err != nil {
		panic(err)
	}
	testPackageRLP, err = testImporter.ImportFrom(pathOfPackageRLP, cwd, 0)
	if err != nil {
		panic(fmt.Errorf("can't load package RLP: %v", err))
	}
}

var tests = []string{"uints", "nil", "rawvalue", "optional", "bigint", "uint256"}

func TestOutput(t *testing.T) {
	for _, test := range tests {
		test := test
		t.Run(test, func(t *testing.T) {
			inputFile := filepath.Join("testdata", test+".in.txt")
			outputFile := filepath.Join("testdata", test+".out.txt")
			bctx, typ, err := loadTestSource(inputFile, "Test")
			if err != nil {
				t.Fatal("error loading test source:", err)
			}
			output, err := bctx.generate(typ, true, true)
			if err != nil {
				t.Fatal("error in generate:", err)
			}

			// Set this environment variable to regenerate the test outputs.
			if os.Getenv("WRITE_TEST_FILES") != "" {
				os.WriteFile(outputFile, output, 0644)
			}

			// Check if output matches.
			wantOutput, err := os.ReadFile(outputFile)
			if err != nil {
				t.Fatal("error loading expected test output:", err)
			}
			if !bytes.Equal(output, wantOutput) {
				t.Fatalf("output mismatch, want: %v got %v", string(wantOutput), string(output))
			}
		})
	}
}

func loadTestSource(file string, typeName string) (*buildContext, *types.Named, error) {
	// Load the test input.
	content, err := os.ReadFile(file)
	if err != nil {
		return nil, nil, err
	}
	f, err := parser.ParseFile(testFset, file, content, 0)
	if err != nil {
		return nil, nil, err
	}
	conf := types.Config{Importer: testImporter}
	pkg, err := conf.Check("test", testFset, []*ast.File{f}, nil)
	if err != nil {
		return nil, nil, err
	}

	// Find the test struct.
	bctx := newBuildContext(testPackageRLP)
	typ, err := lookupStructType(pkg.Scope(), typeName)
	if err != nil {
		return nil, nil, fmt.Errorf("can't find type %s: %v", typeName, err)
	}
	return bctx, typ, nil
}
