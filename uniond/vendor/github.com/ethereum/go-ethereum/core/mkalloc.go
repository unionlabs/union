// Copyright 2017 The go-ethereum Authors
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

//go:build none
// +build none

/*
The mkalloc tool creates the genesis allocation constants in genesis_alloc.go
It outputs a const declaration that contains an RLP-encoded list of (address, balance) tuples.

	go run mkalloc.go genesis.json
*/
package main

import (
	"encoding/json"
	"fmt"
	"math/big"
	"os"
	"strconv"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/rlp"
	"golang.org/x/exp/slices"
)

type allocItem struct {
	Addr    *big.Int
	Balance *big.Int
	Misc    *allocItemMisc `rlp:"optional"`
}

type allocItemMisc struct {
	Nonce uint64
	Code  []byte
	Slots []allocItemStorageItem
}

type allocItemStorageItem struct {
	Key common.Hash
	Val common.Hash
}

func makelist(g *core.Genesis) []allocItem {
	items := make([]allocItem, 0, len(g.Alloc))
	for addr, account := range g.Alloc {
		var misc *allocItemMisc
		if len(account.Storage) > 0 || len(account.Code) > 0 || account.Nonce != 0 {
			misc = &allocItemMisc{
				Nonce: account.Nonce,
				Code:  account.Code,
				Slots: make([]allocItemStorageItem, 0, len(account.Storage)),
			}
			for key, val := range account.Storage {
				misc.Slots = append(misc.Slots, allocItemStorageItem{key, val})
			}
			slices.SortFunc(misc.Slots, func(a, b allocItemStorageItem) int {
				return a.Key.Cmp(b.Key)
			})
		}
		bigAddr := new(big.Int).SetBytes(addr.Bytes())
		items = append(items, allocItem{bigAddr, account.Balance, misc})
	}
	slices.SortFunc(items, func(a, b allocItem) int {
		return a.Addr.Cmp(b.Addr)
	})
	return items
}

func makealloc(g *core.Genesis) string {
	a := makelist(g)
	data, err := rlp.EncodeToBytes(a)
	if err != nil {
		panic(err)
	}
	return strconv.QuoteToASCII(string(data))
}

func main() {
	if len(os.Args) != 2 {
		fmt.Fprintln(os.Stderr, "Usage: mkalloc genesis.json")
		os.Exit(1)
	}

	g := new(core.Genesis)
	file, err := os.Open(os.Args[1])
	if err != nil {
		panic(err)
	}
	if err := json.NewDecoder(file).Decode(g); err != nil {
		panic(err)
	}
	fmt.Println("const allocData =", makealloc(g))
}
