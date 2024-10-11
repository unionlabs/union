/*
 * Copyright 2019 Dgraph Labs, Inc. and Contributors
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

package sim

import (
	"bufio"
	"errors"
	"fmt"
	"io"
	"math/rand"
	"strconv"
	"strings"
	"time"
)

var (
	// ErrDone is returned when the underlying file has ran out of lines.
	ErrDone = errors.New("no more values in the Simulator")
	// ErrBadLine is returned when the trace file line is unrecognizable to
	// the Parser.
	ErrBadLine = errors.New("bad line for trace format")
)

// Simulator is the central type of the `sim` package. It is a function
// returning a key from some source (composed from the other functions in this
// package, either generated or parsed). You can use these Simulators to
// approximate access distributions.
type Simulator func() (uint64, error)

// NewZipfian creates a Simulator returning numbers following a Zipfian [1]
// distribution infinitely. Zipfian distributions are useful for simulating real
// workloads.
//
// [1]: https://en.wikipedia.org/wiki/Zipf%27s_law
func NewZipfian(s, v float64, n uint64) Simulator {
	z := rand.NewZipf(rand.New(rand.NewSource(time.Now().UnixNano())), s, v, n)
	return func() (uint64, error) {
		return z.Uint64(), nil
	}
}

// NewUniform creates a Simulator returning uniformly distributed [1] (random)
// numbers [0, max) infinitely.
//
// [1]: https://en.wikipedia.org/wiki/Uniform_distribution_(continuous)
func NewUniform(max uint64) Simulator {
	m := int64(max)
	r := rand.New(rand.NewSource(time.Now().UnixNano()))
	return func() (uint64, error) {
		return uint64(r.Int63n(m)), nil
	}
}

// Parser is used as a parameter to NewReader so we can create Simulators from
// varying trace file formats easily.
type Parser func(string, error) ([]uint64, error)

// NewReader creates a Simulator from two components: the Parser, which is a
// filetype specific function for parsing lines, and the file itself, which will
// be read from.
//
// When every line in the file has been read, ErrDone will be returned. For some
// trace formats (LIRS) there is one item per line. For others (ARC) there is a
// range of items on each line. Thus, the true number of items in each file
// is hard to determine, so it's up to the user to handle ErrDone accordingly.
func NewReader(parser Parser, file io.Reader) Simulator {
	b := bufio.NewReader(file)
	s := make([]uint64, 0)
	i := -1
	var err error
	return func() (uint64, error) {
		// only parse a new line when we've run out of items
		if i++; i == len(s) {
			// parse sequence from line
			if s, err = parser(b.ReadString('\n')); err != nil {
				s = []uint64{0}
			}
			i = 0
		}
		return s[i], err
	}
}

// ParseLIRS takes a single line of input from a LIRS trace file as described in
// multiple papers [1] and returns a slice containing one number. A nice
// collection of LIRS trace files can be found in Ben Manes' repo [2].
//
// [1]: https://en.wikipedia.org/wiki/LIRS_caching_algorithm
// [2]: https://git.io/fj9gU
func ParseLIRS(line string, err error) ([]uint64, error) {
	if line = strings.TrimSpace(line); line != "" {
		// example: "1\r\n"
		key, err := strconv.ParseUint(line, 10, 64)
		return []uint64{key}, err
	}
	return nil, ErrDone
}

// ParseARC takes a single line of input from an ARC trace file as described in
// "ARC: a self-tuning, low overhead replacement cache" [1] by Nimrod Megiddo
// and Dharmendra S. Modha [1] and returns a sequence of numbers generated from
// the line and any error. For use with NewReader.
//
// [1]: https://scinapse.io/papers/1860107648
func ParseARC(line string, err error) ([]uint64, error) {
	if line != "" {
		// example: "0 5 0 0\n"
		//
		// -  first block: starting number in sequence
		// - second block: number of items in sequence
		// -  third block: ignore
		// - fourth block: global line number (not used)
		cols := strings.Fields(line)
		if len(cols) != 4 {
			return nil, ErrBadLine
		}
		start, err := strconv.ParseUint(cols[0], 10, 64)
		if err != nil {
			return nil, err
		}
		count, err := strconv.ParseUint(cols[1], 10, 64)
		if err != nil {
			return nil, err
		}
		// populate sequence from start to start + count
		seq := make([]uint64, count)
		for i := range seq {
			seq[i] = start + uint64(i)
		}
		return seq, nil
	}
	return nil, ErrDone
}

// Collection evaluates the Simulator size times and saves each item to the
// returned slice.
func Collection(simulator Simulator, size uint64) []uint64 {
	collection := make([]uint64, size)
	for i := range collection {
		collection[i], _ = simulator()
	}
	return collection
}

// StringCollection evaluates the Simulator size times and saves each item to
// the returned slice, after converting it to a string.
func StringCollection(simulator Simulator, size uint64) []string {
	collection := make([]string, size)
	for i := range collection {
		n, _ := simulator()
		collection[i] = fmt.Sprintf("%d", n)
	}
	return collection
}
