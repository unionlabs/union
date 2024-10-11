// Copyright 2019, 2020 OCI Contributors
// Copyright 2017 Docker, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package digest

import (
	"testing"
)

func TestParseDigest(t *testing.T) {
	for _, testcase := range []struct {
		input     string
		err       error
		algorithm Algorithm
		encoded   string
	}{
		{
			input:     "sha256:e58fcf7418d4390dec8e8fb69d88c06ec07039d651fedd3aa72af9972e7d046b",
			algorithm: "sha256",
			encoded:   "e58fcf7418d4390dec8e8fb69d88c06ec07039d651fedd3aa72af9972e7d046b",
		},
		{
			input:     "sha384:d3fc7881460b7e22e3d172954463dddd7866d17597e7248453c48b3e9d26d9596bf9c4a9cf8072c9d5bad76e19af801d",
			algorithm: "sha384",
			encoded:   "d3fc7881460b7e22e3d172954463dddd7866d17597e7248453c48b3e9d26d9596bf9c4a9cf8072c9d5bad76e19af801d",
		},
		{
			// empty hex
			input: "sha256:",
			err:   ErrDigestInvalidFormat,
		},
		{
			// empty hex
			input: ":",
			err:   ErrDigestInvalidFormat,
		},
		{
			// just hex
			input: "d41d8cd98f00b204e9800998ecf8427e",
			err:   ErrDigestInvalidFormat,
		},
		{
			// not hex
			input: "sha256:d41d8cd98f00b204e9800m98ecf8427e",
			err:   ErrDigestInvalidLength,
		},
		{
			// too short
			input: "sha256:abcdef0123456789",
			err:   ErrDigestInvalidLength,
		},
		{
			// too short (from different algorithm)
			input: "sha512:abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
			err:   ErrDigestInvalidLength,
		},
		{
			input: "foo:d41d8cd98f00b204e9800998ecf8427e",
			err:   ErrDigestUnsupported,
		},
		{
			// repeated separators
			input: "sha384__foo+bar:d3fc7881460b7e22e3d172954463dddd7866d17597e7248453c48b3e9d26d9596bf9c4a9cf8072c9d5bad76e19af801d",
			err:   ErrDigestInvalidFormat,
		},
		{
			// ensure that we parse, but we don't have support for the algorithm
			input:     "sha384.foo+bar:d3fc7881460b7e22e3d172954463dddd7866d17597e7248453c48b3e9d26d9596bf9c4a9cf8072c9d5bad76e19af801d",
			algorithm: "sha384.foo+bar",
			encoded:   "d3fc7881460b7e22e3d172954463dddd7866d17597e7248453c48b3e9d26d9596bf9c4a9cf8072c9d5bad76e19af801d",
			err:       ErrDigestUnsupported,
		},
		{
			input:     "sha384_foo+bar:d3fc7881460b7e22e3d172954463dddd7866d17597e7248453c48b3e9d26d9596bf9c4a9cf8072c9d5bad76e19af801d",
			algorithm: "sha384_foo+bar",
			encoded:   "d3fc7881460b7e22e3d172954463dddd7866d17597e7248453c48b3e9d26d9596bf9c4a9cf8072c9d5bad76e19af801d",
			err:       ErrDigestUnsupported,
		},
		{
			input:     "sha256+b64:LCa0a2j_xo_5m0U8HTBBNBNCLXBkg7-g-YpeiGJm564",
			algorithm: "sha256+b64",
			encoded:   "LCa0a2j_xo_5m0U8HTBBNBNCLXBkg7-g-YpeiGJm564",
			err:       ErrDigestUnsupported,
		},
		{
			input: "sha256:E58FCF7418D4390DEC8E8FB69D88C06EC07039D651FEDD3AA72AF9972E7D046B",
			err:   ErrDigestInvalidFormat,
		},
	} {
		digest, err := Parse(testcase.input)
		if err != testcase.err {
			t.Fatalf("error differed from expected while parsing %q: %v != %v", testcase.input, err, testcase.err)
		}

		if testcase.err != nil {
			continue
		}

		if digest.Algorithm() != testcase.algorithm {
			t.Fatalf("incorrect algorithm for parsed digest: %q != %q", digest.Algorithm(), testcase.algorithm)
		}

		if digest.Encoded() != testcase.encoded {
			t.Fatalf("incorrect hex for parsed digest: %q != %q", digest.Encoded(), testcase.encoded)
		}

		// Parse string return value and check equality
		newParsed, err := Parse(digest.String())

		if err != nil {
			t.Fatalf("unexpected error parsing input %q: %v", testcase.input, err)
		}

		if newParsed != digest {
			t.Fatalf("expected equal: %q != %q", newParsed, digest)
		}

		newFromHex := NewDigestFromEncoded(newParsed.Algorithm(), newParsed.Encoded())
		if newFromHex != digest {
			t.Fatalf("%v != %v", newFromHex, digest)
		}
	}
}
