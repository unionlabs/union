// Copyright 2010 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package html

import (
	"bytes"
	"fmt"
	"strings"
	"testing"
)

func TestRenderer(t *testing.T) {
	nodes := [...]*Node{
		0: {
			Type: ElementNode,
			Data: "html",
		},
		1: {
			Type: ElementNode,
			Data: "head",
		},
		2: {
			Type: ElementNode,
			Data: "body",
		},
		3: {
			Type: TextNode,
			Data: "0<1",
		},
		4: {
			Type: ElementNode,
			Data: "p",
			Attr: []Attribute{
				{
					Key: "id",
					Val: "A",
				},
				{
					Key: "foo",
					Val: `abc"def`,
				},
			},
		},
		5: {
			Type: TextNode,
			Data: "2",
		},
		6: {
			Type: ElementNode,
			Data: "b",
			Attr: []Attribute{
				{
					Key: "empty",
					Val: "",
				},
			},
		},
		7: {
			Type: TextNode,
			Data: "3",
		},
		8: {
			Type: ElementNode,
			Data: "i",
			Attr: []Attribute{
				{
					Key: "backslash",
					Val: `\`,
				},
			},
		},
		9: {
			Type: TextNode,
			Data: "&4",
		},
		10: {
			Type: TextNode,
			Data: "5",
		},
		11: {
			Type: ElementNode,
			Data: "blockquote",
		},
		12: {
			Type: ElementNode,
			Data: "br",
		},
		13: {
			Type: TextNode,
			Data: "6",
		},
		14: {
			Type: CommentNode,
			Data: "comm",
		},
		15: {
			Type: CommentNode,
			Data: "x-->y", // Needs escaping.
		},
		16: {
			Type: RawNode,
			Data: "7<pre>8</pre>9",
		},
	}

	// Build a tree out of those nodes, based on a textual representation.
	// Only the ".\t"s are significant. The trailing HTML-like text is
	// just commentary. The "0:" prefixes are for easy cross-reference with
	// the nodes array.
	treeAsText := [...]string{
		0:  `<html>`,
		1:  `.	<head>`,
		2:  `.	<body>`,
		3:  `.	.	"0&lt;1"`,
		4:  `.	.	<p id="A" foo="abc&#34;def">`,
		5:  `.	.	.	"2"`,
		6:  `.	.	.	<b empty="">`,
		7:  `.	.	.	.	"3"`,
		8:  `.	.	.	<i backslash="\">`,
		9:  `.	.	.	.	"&amp;4"`,
		10: `.	.	"5"`,
		11: `.	.	<blockquote>`,
		12: `.	.	<br>`,
		13: `.	.	"6"`,
		14: `.	.	"<!--comm-->"`,
		15: `.	.	"<!--x--&gt;y-->"`,
		16: `.	.	"7<pre>8</pre>9"`,
	}
	if len(nodes) != len(treeAsText) {
		t.Fatal("len(nodes) != len(treeAsText)")
	}
	var stack [8]*Node
	for i, line := range treeAsText {
		level := 0
		for line[0] == '.' {
			// Strip a leading ".\t".
			line = line[2:]
			level++
		}
		n := nodes[i]
		if level == 0 {
			if stack[0] != nil {
				t.Fatal("multiple root nodes")
			}
			stack[0] = n
		} else {
			stack[level-1].AppendChild(n)
			stack[level] = n
			for i := level + 1; i < len(stack); i++ {
				stack[i] = nil
			}
		}
		// At each stage of tree construction, we check all nodes for consistency.
		for j, m := range nodes {
			if err := checkNodeConsistency(m); err != nil {
				t.Fatalf("i=%d, j=%d: %v", i, j, err)
			}
		}
	}

	want := `<html><head></head><body>0&lt;1<p id="A" foo="abc&#34;def">` +
		`2<b empty="">3</b><i backslash="\">&amp;4</i></p>` +
		`5<blockquote></blockquote><br/>6<!--comm--><!--x--&gt;y-->7<pre>8</pre>9</body></html>`
	b := new(bytes.Buffer)
	if err := Render(b, nodes[0]); err != nil {
		t.Fatal(err)
	}
	if got := b.String(); got != want {
		t.Errorf("got vs want:\n%s\n%s\n", got, want)
	}
}

func TestRenderTextNodes(t *testing.T) {
	elements := []string{"style", "script", "xmp", "iframe", "noembed", "noframes", "plaintext", "noscript"}
	for _, namespace := range []string{
		"", // html
		"svg",
		"math",
	} {
		for _, e := range elements {
			var namespaceOpen, namespaceClose string
			if namespace != "" {
				namespaceOpen, namespaceClose = fmt.Sprintf("<%s>", namespace), fmt.Sprintf("</%s>", namespace)
			}
			doc := fmt.Sprintf(`<html><head></head><body>%s<%s>&</%s>%s</body></html>`, namespaceOpen, e, e, namespaceClose)
			n, err := Parse(strings.NewReader(doc))
			if err != nil {
				t.Fatal(err)
			}
			b := bytes.NewBuffer(nil)
			if err := Render(b, n); err != nil {
				t.Fatal(err)
			}

			expected := doc
			if namespace != "" {
				expected = strings.Replace(expected, "&", "&amp;", 1)
			}

			if b.String() != expected {
				t.Errorf("unexpected output: got %q, want %q", b.String(), expected)
			}
		}
	}
}
