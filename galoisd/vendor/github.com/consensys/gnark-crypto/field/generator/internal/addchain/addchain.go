// Original copyright :
// BSD 3-Clause License

// Copyright (c) 2019, Michael McLoughlin
// All rights reserved.

// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:

// 1. Redistributions of source code must retain the above copyright notice, this
//    list of conditions and the following disclaimer.

// 2. Redistributions in binary form must reproduce the above copyright notice,
//    this list of conditions and the following disclaimer in the documentation
//    and/or other materials provided with the distribution.

// 3. Neither the name of the copyright holder nor the names of its
//    contributors may be used to endorse or promote products derived from
//    this software without specific prior written permission.

// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

// Package addchain is derived from github.com/mmcloughlin/addchain internal packages or examples
package addchain

import (
	"bufio"
	"encoding/gob"
	"log"
	"math/big"
	"os"
	"path/filepath"
	"reflect"
	"strings"
	"sync"

	"github.com/mmcloughlin/addchain"
	"github.com/mmcloughlin/addchain/acc"
	"github.com/mmcloughlin/addchain/acc/ast"
	"github.com/mmcloughlin/addchain/acc/ir"
	"github.com/mmcloughlin/addchain/acc/pass"
	"github.com/mmcloughlin/addchain/acc/printer"
	"github.com/mmcloughlin/addchain/alg/ensemble"
	"github.com/mmcloughlin/addchain/alg/exec"
	"github.com/mmcloughlin/addchain/meta"
)

// most of these functions are derived from github.com/mmcloughlin/addchain internal packages or examples

var (
	once        sync.Once
	addChainDir string
	mAddchains  map[string]*AddChainData // key is big.Int.Text(16)
)

// GetAddChain retunrs template data of a short addition chain for given big.Int
func GetAddChain(n *big.Int) *AddChainData {

	// init the cache only once.
	once.Do(initCache)

	key := n.Text(16)
	if r, ok := mAddchains[key]; ok {
		return r
	}

	// Default ensemble of algorithms.
	algorithms := ensemble.Ensemble()

	// Use parallel executor.
	ex := exec.NewParallel()
	results := ex.Execute(n, algorithms)

	// Output best result.
	best := 0
	for i, r := range results {
		if r.Err != nil {
			log.Fatal(r.Err)
		}
		if len(results[i].Program) < len(results[best].Program) {
			best = i
		}
	}
	r := results[best]
	data := processSearchResult(r.Program, key)

	mAddchains[key] = data
	// gob encode
	file := filepath.Join(addChainDir, key)
	log.Println("saving addchain", file)
	f, err := os.Create(file)
	if err != nil {
		log.Fatal(err)
	}
	enc := gob.NewEncoder(f)

	if err := enc.Encode(r.Program); err != nil {
		_ = f.Close()
		log.Fatal(err)
	}
	_ = f.Close()

	return data
}

func processSearchResult(_p addchain.Program, n string) *AddChainData {
	p, err := acc.Decompile(_p)
	if err != nil {
		log.Fatal(err)
	}
	chain, err := acc.Build(p)
	if err != nil {
		log.Fatal(err)
	}

	data, err := prepareAddChainData(chain, n)
	if err != nil {
		log.Fatal(err)
	}
	return data
}

// Data provided to templates.
type AddChainData struct {
	// Chain is the addition chain as a list of integers.
	Chain addchain.Chain

	// Ops is the complete sequence of addition operations required to compute
	// the addition chain.
	Ops addchain.Program

	// Script is the condensed representation of the addition chain computation
	// in the "addition chain calculator" language.
	Script *ast.Chain

	// Program is the intermediate representation of the addition chain
	// computation. This representation is likely the most convenient for code
	// generation. It contains a sequence of add, double and shift (repeated
	// doubling) instructions required to compute the chain. Temporary variable
	// allocation has been performed and the list of required temporaries
	// populated.
	Program *ir.Program

	// Metadata about the addchain project and the specific release parameters.
	// Please use this to include a reference or citation back to the addchain
	// project in your generated output.
	Meta *meta.Properties

	N string // base 16 value of the value
}

// PrepareData builds input template data for the given addition chain script.
func prepareAddChainData(s *ast.Chain, n string) (*AddChainData, error) {
	// Prepare template data.
	allocator := pass.Allocator{
		Input:  "x",
		Output: "z",
		Format: "t%d",
	}
	// Translate to IR.
	p, err := acc.Translate(s)
	if err != nil {
		return nil, err
	}

	// Apply processing passes: temporary variable allocation, and computing the
	// full addition chain sequence and operations.
	if err := pass.Exec(p, allocator, pass.Func(pass.Eval)); err != nil {
		return nil, err
	}

	return &AddChainData{
		Chain:   p.Chain,
		Ops:     p.Program,
		Script:  s,
		Program: p,
		Meta:    meta.Meta,
		N:       n,
	}, nil
}

// Function is a function provided to templates.
type Function struct {
	Name        string
	Description string
	Func        interface{}
}

// Signature returns the function signature.
func (f *Function) Signature() string {
	return reflect.ValueOf(f.Func).Type().String()
}

// Functions is the list of functions provided to templates.
var Functions = []*Function{
	{
		Name:        "add_",
		Description: "If the input operation is an `ir.Add` then return it, otherwise return `nil`",
		Func: func(op ir.Op) ir.Op {
			if a, ok := op.(ir.Add); ok {
				return a
			}
			return nil
		},
	},
	{
		Name:        "double_",
		Description: "If the input operation is an `ir.Double` then return it, otherwise return `nil`",
		Func: func(op ir.Op) ir.Op {
			if d, ok := op.(ir.Double); ok {
				return d
			}
			return nil
		},
	},
	{
		Name:        "shift_",
		Description: "If the input operation is an `ir.Shift` then return it, otherwise return `nil`",
		Func: func(op ir.Op) ir.Op {
			if s, ok := op.(ir.Shift); ok {
				return s
			}
			return nil
		},
	},
	{
		Name:        "inc_",
		Description: "Increment an integer",
		Func:        func(n int) int { return n + 1 },
	},
	{
		Name:        "format_",
		Description: "Formats an addition chain script (`*ast.Chain`) as a string",
		Func:        printer.String,
	},
	{
		Name:        "split_",
		Description: "Calls `strings.Split`",
		Func:        strings.Split,
	},
	{
		Name:        "join_",
		Description: "Calls `strings.Join`",
		Func:        strings.Join,
	},
	{
		Name:        "lines_",
		Description: "Split input string into lines",
		Func: func(s string) []string {
			var lines []string
			scanner := bufio.NewScanner(strings.NewReader(s))
			for scanner.Scan() {
				lines = append(lines, scanner.Text())
			}
			return lines
		},
	},
	{
		Name:        "ptr_",
		Description: "adds & if it's a value",
		Func: func(s *ir.Operand) string {
			if s.String() == "x" {
				return "&"
			}
			return ""
		},
	},
	{
		Name: "last_",
		Func: func(x int, a interface{}) bool {
			return x == reflect.ValueOf(a).Len()-1
		},
	},
}

// to speed up code generation, we cache addchain search results on disk
func initCache() {
	mAddchains = make(map[string]*AddChainData)

	// read existing files in addchain directory
	path, err := os.Getwd()
	if err != nil {
		log.Fatal(err)
	}
	addChainDir = filepath.Join(path, "addchain")
	_ = os.Mkdir(addChainDir, 0700)
	files, err := os.ReadDir(addChainDir)
	if err != nil {
		log.Fatal(err)
	}

	// preload pre-computed add chains
	for _, entry := range files {
		if entry.IsDir() {
			continue
		}
		f, err := os.Open(filepath.Join(addChainDir, entry.Name()))
		if err != nil {
			log.Fatal(err)
		}

		// decode the addchain.Program
		dec := gob.NewDecoder(f)
		var p addchain.Program
		err = dec.Decode(&p)
		_ = f.Close()
		if err != nil {
			log.Fatal(err)
		}
		data := processSearchResult(p, filepath.Base(f.Name()))
		log.Println("read", filepath.Base(f.Name()))

		// save the data
		mAddchains[filepath.Base(f.Name())] = data

	}

}
