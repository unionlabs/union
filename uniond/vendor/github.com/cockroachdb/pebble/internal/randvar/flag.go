// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package randvar

import (
	"encoding/binary"
	"regexp"
	"strconv"
	"strings"

	"github.com/cockroachdb/errors"
	"golang.org/x/exp/rand"
)

var randVarRE = regexp.MustCompile(`^(?:(latest|uniform|zipf):)?(\d+)(?:-(\d+))?$`)

// Flag provides a command line flag interface for specifying static random
// variables.
type Flag struct {
	Static
	spec string
}

// NewFlag creates a new Flag initialized with the specified spec.
func NewFlag(spec string) *Flag {
	f := &Flag{}
	if err := f.Set(spec); err != nil {
		panic(err)
	}
	return f
}

func (f *Flag) String() string {
	return f.spec
}

// Type implements the Flag.Value interface.
func (f *Flag) Type() string {
	return "randvar"
}

// Set implements the Flag.Value interface.
func (f *Flag) Set(spec string) error {
	m := randVarRE.FindStringSubmatch(spec)
	if m == nil {
		return errors.Errorf("invalid random var spec: %s", errors.Safe(spec))
	}

	min, err := strconv.Atoi(m[2])
	if err != nil {
		return err
	}
	max := min
	if m[3] != "" {
		max, err = strconv.Atoi(m[3])
		if err != nil {
			return err
		}
	}

	switch strings.ToLower(m[1]) {
	case "", "uniform":
		f.Static = NewUniform(uint64(min), uint64(max))
	case "latest":
		f.Static, err = NewSkewedLatest(uint64(min), uint64(max), 0.99)
		if err != nil {
			return err
		}
	case "zipf":
		var err error
		f.Static, err = NewZipf(uint64(min), uint64(max), 0.99)
		if err != nil {
			return err
		}
	default:
		return errors.Errorf("unknown random var distribution: %s", errors.Safe(m[1]))
	}
	f.spec = spec
	return nil
}

// BytesFlag provides a command line flag interface for specifying random
// bytes. The specification provides for both the length of the random bytes
// and a target compression ratio.
type BytesFlag struct {
	sizeFlag          Flag
	targetCompression float64
	spec              string
}

// NewBytesFlag creates a new BytesFlag initialized with the specified spec.
func NewBytesFlag(spec string) *BytesFlag {
	f := &BytesFlag{}
	if err := f.Set(spec); err != nil {
		panic(err)
	}
	return f
}

func (f *BytesFlag) String() string {
	return f.spec
}

// Type implements the Flag.Value interface.
func (f *BytesFlag) Type() string {
	return "randbytes"
}

// Set implements the Flag.Value interface.
func (f *BytesFlag) Set(spec string) error {
	parts := strings.Split(spec, "/")
	if len(parts) == 0 || len(parts) > 2 {
		return errors.Errorf("invalid randbytes spec: %s", errors.Safe(spec))
	}
	if err := f.sizeFlag.Set(parts[0]); err != nil {
		return err
	}
	f.targetCompression = 1.0
	if len(parts) == 2 {
		var err error
		f.targetCompression, err = strconv.ParseFloat(parts[1], 64)
		if err != nil {
			return err
		}
	}
	f.spec = spec
	return nil
}

// Bytes returns random bytes. The length of the random bytes comes from the
// internal sizeFlag.
func (f *BytesFlag) Bytes(r *rand.Rand, buf []byte) []byte {
	size := int(f.sizeFlag.Uint64(r))
	uniqueSize := int(float64(size) / f.targetCompression)
	if uniqueSize < 1 {
		uniqueSize = 1
	}
	if cap(buf) < size {
		buf = make([]byte, size)
	}
	data := buf[:size]
	offset := 0
	for offset+8 <= uniqueSize {
		binary.LittleEndian.PutUint64(data[offset:], r.Uint64())
		offset += 8
	}
	word := r.Uint64()
	for offset < uniqueSize {
		data[offset] = byte(word)
		word >>= 8
		offset++
	}
	for offset < size {
		data[offset] = data[offset-uniqueSize]
		offset++
	}
	return data
}
