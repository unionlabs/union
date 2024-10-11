// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"bufio"
	"compress/bzip2"
	"embed"
	"encoding/hex"
	"fmt"
	"regexp"
	"strconv"
	"strings"

	"github.com/tunabay/go-bitarray"
)

//go:embed testdata/*.rsp.bz2
var cavpTestDataFS embed.FS

type cavpTestCase struct {
	ba *bitarray.BitArray
	md []byte
}

var cavpRspLineRE = regexp.MustCompile(`^([a-zA-Z0-9]+)\s*=\s*([a-zA-Z0-9]+)$`)

func cavpTestCases(name string) ([]*cavpTestCase, error) {
	fname := fmt.Sprintf("testdata/%s.rsp.bz2", name)
	file, err := cavpTestDataFS.Open(fname)
	if err != nil {
		return nil, fmt.Errorf("%s: %w", fname, err)
	}
	defer file.Close()

	bz2r := bzip2.NewReader(file)
	scanner := bufio.NewScanner(bz2r)

	var tcs []*cavpTestCase

	lineNo := 0
	st := 0
	var nBits int
	var buf []byte
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		lineNo++
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		m := cavpRspLineRE.FindStringSubmatch(line)
		if len(m) == 0 {
			continue
		}
		key, val := strings.ToLower(m[1]), m[2]
		switch st {
		case 0:
			if key != "len" {
				return nil, fmt.Errorf("unexpected line (Len): %s:L%d: %s", fname, lineNo, line)
			}
			n64, err := strconv.ParseUint(val, 10, 32)
			if err != nil {
				return nil, fmt.Errorf("invalid Len = %q: %s:L%d: %w", val, fname, lineNo, err)
			}
			nBits = int(n64)
			st = 1
		case 1:
			if key != "msg" {
				return nil, fmt.Errorf("unexpected line (Msg): %s:L%d: %s", fname, lineNo, line)
			}
			b, err := hex.DecodeString(val)
			if err != nil {
				return nil, fmt.Errorf("invalid Msg = %q: %s:L%d: %w", val, fname, lineNo, err)
			}
			buf = b
			st = 2
		case 2:
			if key != "md" {
				return nil, fmt.Errorf("unexpected line (MD): %s:L%d: %s", fname, lineNo, line)
			}
			md, err := hex.DecodeString(val)
			if err != nil {
				return nil, fmt.Errorf("invalid MD = %q: %s:L%d: %w", val, fname, lineNo, err)
			}
			tcs = append(tcs, &cavpTestCase{
				ba: bitarray.NewFromBytes(buf, 0, nBits),
				md: md,
			})
			st = 0
		}
	}

	return tcs, nil
}
