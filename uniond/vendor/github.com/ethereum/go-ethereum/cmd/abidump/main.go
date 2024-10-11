// Copyright 2020 The go-ethereum Authors
// This file is part of go-ethereum.
//
// go-ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// go-ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with go-ethereum. If not, see <http://www.gnu.org/licenses/>.

package main

import (
	"encoding/hex"
	"flag"
	"fmt"
	"os"
	"strings"

	"github.com/ethereum/go-ethereum/signer/core/apitypes"
	"github.com/ethereum/go-ethereum/signer/fourbyte"
)

func init() {
	flag.Usage = func() {
		fmt.Fprintln(os.Stderr, "Usage:", os.Args[0], "<hexdata>")
		flag.PrintDefaults()
		fmt.Fprintln(os.Stderr, `
Parses the given ABI data and tries to interpret it from the fourbyte database.`)
	}
}

func parse(data []byte) {
	db, err := fourbyte.New()
	if err != nil {
		die(err)
	}
	messages := apitypes.ValidationMessages{}
	db.ValidateCallData(nil, data, &messages)
	for _, m := range messages.Messages {
		fmt.Printf("%v: %v\n", m.Typ, m.Message)
	}
}

// Example
// ./abidump a9059cbb000000000000000000000000ea0e2dc7d65a50e77fc7e84bff3fd2a9e781ff5c0000000000000000000000000000000000000000000000015af1d78b58c40000
func main() {
	flag.Parse()

	switch {
	case flag.NArg() == 1:
		hexdata := flag.Arg(0)
		data, err := hex.DecodeString(strings.TrimPrefix(hexdata, "0x"))
		if err != nil {
			die(err)
		}
		parse(data)
	default:
		fmt.Fprintln(os.Stderr, "Error: one argument needed")
		flag.Usage()
		os.Exit(2)
	}
}

func die(args ...interface{}) {
	fmt.Fprintln(os.Stderr, args...)
	os.Exit(1)
}
