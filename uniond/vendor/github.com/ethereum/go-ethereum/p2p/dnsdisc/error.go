// Copyright 2019 The go-ethereum Authors
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

package dnsdisc

import (
	"errors"
	"fmt"
)

// Entry parse errors.
var (
	errUnknownEntry = errors.New("unknown entry type")
	errNoPubkey     = errors.New("missing public key")
	errBadPubkey    = errors.New("invalid public key")
	errInvalidENR   = errors.New("invalid node record")
	errInvalidChild = errors.New("invalid child hash")
	errInvalidSig   = errors.New("invalid base64 signature")
	errSyntax       = errors.New("invalid syntax")
)

// Resolver/sync errors
var (
	errNoRoot        = errors.New("no valid root found")
	errNoEntry       = errors.New("no valid tree entry found")
	errHashMismatch  = errors.New("hash mismatch")
	errENRInLinkTree = errors.New("enr entry in link tree")
	errLinkInENRTree = errors.New("link entry in ENR tree")
)

type nameError struct {
	name string
	err  error
}

func (err nameError) Error() string {
	if ee, ok := err.err.(entryError); ok {
		return fmt.Sprintf("invalid %s entry at %s: %v", ee.typ, err.name, ee.err)
	}
	return err.name + ": " + err.err.Error()
}

type entryError struct {
	typ string
	err error
}

func (err entryError) Error() string {
	return fmt.Sprintf("invalid %s entry: %v", err.typ, err.err)
}
