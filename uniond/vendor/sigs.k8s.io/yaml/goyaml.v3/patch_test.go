/*
Copyright 2023 The Kubernetes Authors.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

package yaml_test

import (
	"bytes"

	. "gopkg.in/check.v1"
	yaml "sigs.k8s.io/yaml/goyaml.v3"
)

func (s *S) TestCompactSeqIndentDefault(c *C) {
	var buf bytes.Buffer
	enc := yaml.NewEncoder(&buf)
	enc.CompactSeqIndent()
	err := enc.Encode(map[string]interface{}{"a": []string{"b", "c"}})
	c.Assert(err, Equals, nil)
	err = enc.Close()
	c.Assert(err, Equals, nil)
	// The default indent is 4, so these sequence elements get 2 indents as before
	c.Assert(buf.String(), Equals, `a:
  - b
  - c
`)
}

func (s *S) TestCompactSequenceWithSetIndent(c *C) {
	var buf bytes.Buffer
	enc := yaml.NewEncoder(&buf)
	enc.CompactSeqIndent()
	enc.SetIndent(2)
	err := enc.Encode(map[string]interface{}{"a": []string{"b", "c"}})
	c.Assert(err, Equals, nil)
	err = enc.Close()
	c.Assert(err, Equals, nil)
	// The sequence indent is 2, so these sequence elements don't get indented at all
	c.Assert(buf.String(), Equals, `a:
- b
- c
`)
}

type normal string
type compact string

// newlinePlusNormalToNewlinePlusCompact maps the normal encoding (prefixed with a newline)
// to the compact encoding (prefixed with a newline), for test cases in marshalTests
var newlinePlusNormalToNewlinePlusCompact = map[normal]compact{
	normal(`
v:
    - A
    - B
`): compact(`
v:
  - A
  - B
`),

	normal(`
v:
    - A
    - |-
      B
      C
`): compact(`
v:
  - A
  - |-
    B
    C
`),

	normal(`
v:
    - A
    - 1
    - B:
        - 2
        - 3
`): compact(`
v:
  - A
  - 1
  - B:
      - 2
      - 3
`),

	normal(`
a:
    - 1
    - 2
`): compact(`
a:
  - 1
  - 2
`),

	normal(`
a:
    b:
        - c: 1
          d: 2
`): compact(`
a:
    b:
      - c: 1
        d: 2
`),
}

func (s *S) TestEncoderCompactIndents(c *C) {
	for i, item := range marshalTests {
		c.Logf("test %d. %q", i, item.data)
		var buf bytes.Buffer
		enc := yaml.NewEncoder(&buf)
		enc.CompactSeqIndent()
		err := enc.Encode(item.value)
		c.Assert(err, Equals, nil)
		err = enc.Close()
		c.Assert(err, Equals, nil)

		// Default to expecting the item data
		expected := item.data
		// If there's a different compact representation, use that
		if c, ok := newlinePlusNormalToNewlinePlusCompact[normal("\n"+item.data)]; ok {
			expected = string(c[1:])
		}

		c.Assert(buf.String(), Equals, expected)
	}
}

func (s *S) TestNewLinePreserved(c *C) {
	obj := &marshalerValue{}
	obj.Field.value = "a:\n        b:\n                c: d\n"
	data, err := yaml.Marshal(obj)
	c.Assert(err, IsNil)
	c.Assert(string(data), Equals, "_: |\n    a:\n            b:\n                    c: d\n")

	obj.Field.value = "\na:\n        b:\n                c: d\n"
	data, err = yaml.Marshal(obj)
	c.Assert(err, IsNil)
	// the newline at the start of the file should be preserved
	c.Assert(string(data), Equals, "_: |4\n\n    a:\n            b:\n                    c: d\n")
}
