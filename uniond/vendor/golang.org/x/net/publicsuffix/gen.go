// Copyright 2012 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build ignore

package main

// This program generates table.go and table_test.go based on the authoritative
// public suffix list at https://publicsuffix.org/list/effective_tld_names.dat
//
// The version is derived from
// https://api.github.com/repos/publicsuffix/list/commits?path=public_suffix_list.dat
// and a human-readable form is at
// https://github.com/publicsuffix/list/commits/master/public_suffix_list.dat
//
// To fetch a particular git revision, such as 5c70ccd250, pass
// -url "https://raw.githubusercontent.com/publicsuffix/list/5c70ccd250/public_suffix_list.dat"
// and -version "an explicit version string".

import (
	"bufio"
	"bytes"
	"encoding/binary"
	"flag"
	"fmt"
	"go/format"
	"io"
	"net/http"
	"os"
	"regexp"
	"sort"
	"strings"

	"golang.org/x/net/idna"
)

const (
	// This must be a multiple of 8 and no greater than 64.
	// Update nodeValue in list.go if this changes.
	nodesBits = 40

	// These sum of these four values must be no greater than nodesBits.
	nodesBitsChildren   = 10
	nodesBitsICANN      = 1
	nodesBitsTextOffset = 16
	nodesBitsTextLength = 6

	// These sum of these four values must be no greater than 32.
	childrenBitsWildcard = 1
	childrenBitsNodeType = 2
	childrenBitsHi       = 14
	childrenBitsLo       = 14
)

var (
	combinedText  string
	maxChildren   int
	maxTextOffset int
	maxTextLength int
	maxHi         uint32
	maxLo         uint32
)

func max(a, b int) int {
	if a < b {
		return b
	}
	return a
}

func u32max(a, b uint32) uint32 {
	if a < b {
		return b
	}
	return a
}

const (
	nodeTypeNormal     = 0
	nodeTypeException  = 1
	nodeTypeParentOnly = 2
	numNodeType        = 3
)

func nodeTypeStr(n int) string {
	switch n {
	case nodeTypeNormal:
		return "+"
	case nodeTypeException:
		return "!"
	case nodeTypeParentOnly:
		return "o"
	}
	panic("unreachable")
}

const (
	defaultURL   = "https://publicsuffix.org/list/effective_tld_names.dat"
	gitCommitURL = "https://api.github.com/repos/publicsuffix/list/commits?path=public_suffix_list.dat"
)

var (
	labelEncoding = map[string]uint64{}
	labelsList    = []string{}
	labelsMap     = map[string]bool{}
	rules         = []string{}
	numICANNRules = 0

	// validSuffixRE is used to check that the entries in the public suffix
	// list are in canonical form (after Punycode encoding). Specifically,
	// capital letters are not allowed.
	validSuffixRE = regexp.MustCompile(`^[a-z0-9_\!\*\-\.]+$`)

	shaRE  = regexp.MustCompile(`"sha":"([^"]+)"`)
	dateRE = regexp.MustCompile(`"committer":{[^{]+"date":"([^"]+)"`)

	subset  = flag.Bool("subset", false, "generate only a subset of the full table, for debugging")
	url     = flag.String("url", defaultURL, "URL of the publicsuffix.org list. If empty, stdin is read instead")
	v       = flag.Bool("v", false, "verbose output (to stderr)")
	version = flag.String("version", "", "the effective_tld_names.dat version")
)

func main() {
	if err := main1(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}

func main1() error {
	flag.Parse()
	if nodesBits > 64 {
		return fmt.Errorf("nodesBits is too large")
	}
	if nodesBits%8 != 0 {
		return fmt.Errorf("nodesBits must be a multiple of 8")
	}
	if nodesBitsTextLength+nodesBitsTextOffset+nodesBitsICANN+nodesBitsChildren > nodesBits {
		return fmt.Errorf("not enough bits to encode the nodes table")
	}
	if childrenBitsLo+childrenBitsHi+childrenBitsNodeType+childrenBitsWildcard > 32 {
		return fmt.Errorf("not enough bits to encode the children table")
	}
	if *version == "" {
		if *url != defaultURL {
			return fmt.Errorf("-version was not specified, and the -url is not the default one")
		}
		sha, date, err := gitCommit()
		if err != nil {
			return err
		}
		*version = fmt.Sprintf("publicsuffix.org's public_suffix_list.dat, git revision %s (%s)", sha, date)
	}
	var r io.Reader = os.Stdin
	if *url != "" {
		res, err := http.Get(*url)
		if err != nil {
			return err
		}
		if res.StatusCode != http.StatusOK {
			return fmt.Errorf("bad GET status for %s: %s", *url, res.Status)
		}
		r = res.Body
		defer res.Body.Close()
	}

	var root node
	icann := false
	br := bufio.NewReader(r)
	for {
		s, err := br.ReadString('\n')
		if err != nil {
			if err == io.EOF {
				break
			}
			return err
		}
		s = strings.TrimSpace(s)
		if strings.Contains(s, "BEGIN ICANN DOMAINS") {
			if len(rules) != 0 {
				return fmt.Errorf(`expected no rules before "BEGIN ICANN DOMAINS"`)
			}
			icann = true
			continue
		}
		if strings.Contains(s, "END ICANN DOMAINS") {
			icann, numICANNRules = false, len(rules)
			continue
		}
		if s == "" || strings.HasPrefix(s, "//") {
			continue
		}
		s, err = idna.ToASCII(s)
		if err != nil {
			return err
		}
		if !validSuffixRE.MatchString(s) {
			return fmt.Errorf("bad publicsuffix.org list data: %q", s)
		}

		if *subset {
			switch {
			case s == "ac.jp" || strings.HasSuffix(s, ".ac.jp"):
			case s == "ak.us" || strings.HasSuffix(s, ".ak.us"):
			case s == "ao" || strings.HasSuffix(s, ".ao"):
			case s == "ar" || strings.HasSuffix(s, ".ar"):
			case s == "arpa" || strings.HasSuffix(s, ".arpa"):
			case s == "cy" || strings.HasSuffix(s, ".cy"):
			case s == "dyndns.org" || strings.HasSuffix(s, ".dyndns.org"):
			case s == "jp":
			case s == "kobe.jp" || strings.HasSuffix(s, ".kobe.jp"):
			case s == "kyoto.jp" || strings.HasSuffix(s, ".kyoto.jp"):
			case s == "om" || strings.HasSuffix(s, ".om"):
			case s == "uk" || strings.HasSuffix(s, ".uk"):
			case s == "uk.com" || strings.HasSuffix(s, ".uk.com"):
			case s == "tw" || strings.HasSuffix(s, ".tw"):
			case s == "zw" || strings.HasSuffix(s, ".zw"):
			case s == "xn--p1ai" || strings.HasSuffix(s, ".xn--p1ai"):
				// xn--p1ai is Russian-Cyrillic "рф".
			default:
				continue
			}
		}

		rules = append(rules, s)

		nt, wildcard := nodeTypeNormal, false
		switch {
		case strings.HasPrefix(s, "*."):
			s, nt = s[2:], nodeTypeParentOnly
			wildcard = true
		case strings.HasPrefix(s, "!"):
			s, nt = s[1:], nodeTypeException
		}
		labels := strings.Split(s, ".")
		for n, i := &root, len(labels)-1; i >= 0; i-- {
			label := labels[i]
			n = n.child(label)
			if i == 0 {
				if nt != nodeTypeParentOnly && n.nodeType == nodeTypeParentOnly {
					n.nodeType = nt
				}
				n.icann = n.icann && icann
				n.wildcard = n.wildcard || wildcard
			}
			labelsMap[label] = true
		}
	}
	labelsList = make([]string, 0, len(labelsMap))
	for label := range labelsMap {
		labelsList = append(labelsList, label)
	}
	sort.Strings(labelsList)

	combinedText = combineText(labelsList)
	if combinedText == "" {
		return fmt.Errorf("internal error: combineText returned no text")
	}
	for _, label := range labelsList {
		offset, length := strings.Index(combinedText, label), len(label)
		if offset < 0 {
			return fmt.Errorf("internal error: could not find %q in text %q", label, combinedText)
		}
		maxTextOffset, maxTextLength = max(maxTextOffset, offset), max(maxTextLength, length)
		if offset >= 1<<nodesBitsTextOffset {
			return fmt.Errorf("text offset %d is too large, or nodeBitsTextOffset is too small", offset)
		}
		if length >= 1<<nodesBitsTextLength {
			return fmt.Errorf("text length %d is too large, or nodeBitsTextLength is too small", length)
		}
		labelEncoding[label] = uint64(offset)<<nodesBitsTextLength | uint64(length)
	}

	if err := root.walk(assignIndexes); err != nil {
		return err
	}

	if err := generate(printMetadata, &root, "table.go"); err != nil {
		return err
	}
	if err := generateBinaryData(&root, combinedText); err != nil {
		return err
	}
	if err := generate(printTest, &root, "table_test.go"); err != nil {
		return err
	}
	return nil
}

func generate(p func(io.Writer, *node) error, root *node, filename string) error {
	buf := new(bytes.Buffer)
	if err := p(buf, root); err != nil {
		return err
	}
	b, err := format.Source(buf.Bytes())
	if err != nil {
		return err
	}
	return os.WriteFile(filename, b, 0644)
}

func gitCommit() (sha, date string, retErr error) {
	res, err := http.Get(gitCommitURL)
	if err != nil {
		return "", "", err
	}
	if res.StatusCode != http.StatusOK {
		return "", "", fmt.Errorf("bad GET status for %s: %s", gitCommitURL, res.Status)
	}
	defer res.Body.Close()
	b, err := io.ReadAll(res.Body)
	if err != nil {
		return "", "", err
	}
	if m := shaRE.FindSubmatch(b); m != nil {
		sha = string(m[1])
	}
	if m := dateRE.FindSubmatch(b); m != nil {
		date = string(m[1])
	}
	if sha == "" || date == "" {
		retErr = fmt.Errorf("could not find commit SHA and date in %s", gitCommitURL)
	}
	return sha, date, retErr
}

func printTest(w io.Writer, n *node) error {
	fmt.Fprintf(w, "// generated by go run gen.go; DO NOT EDIT\n\n")
	fmt.Fprintf(w, "package publicsuffix\n\nconst numICANNRules = %d\n\nvar rules = [...]string{\n", numICANNRules)
	for _, rule := range rules {
		fmt.Fprintf(w, "%q,\n", rule)
	}
	fmt.Fprintf(w, "}\n\nvar nodeLabels = [...]string{\n")
	if err := n.walk(func(n *node) error {
		return printNodeLabel(w, n)
	}); err != nil {
		return err
	}
	fmt.Fprintf(w, "}\n")
	return nil
}

func generateBinaryData(root *node, combinedText string) error {
	if err := os.WriteFile("data/text", []byte(combinedText), 0666); err != nil {
		return err
	}

	var nodes []byte
	if err := root.walk(func(n *node) error {
		for _, c := range n.children {
			nodes = appendNodeEncoding(nodes, c)
		}
		return nil
	}); err != nil {
		return err
	}
	if err := os.WriteFile("data/nodes", nodes, 0666); err != nil {
		return err
	}

	var children []byte
	for _, c := range childrenEncoding {
		children = binary.BigEndian.AppendUint32(children, c)
	}
	if err := os.WriteFile("data/children", children, 0666); err != nil {
		return err
	}

	return nil
}

func appendNodeEncoding(b []byte, n *node) []byte {
	encoding := labelEncoding[n.label]
	if n.icann {
		encoding |= 1 << (nodesBitsTextLength + nodesBitsTextOffset)
	}
	encoding |= uint64(n.childrenIndex) << (nodesBitsTextLength + nodesBitsTextOffset + nodesBitsICANN)
	for i := nodesBits - 8; i >= 0; i -= 8 {
		b = append(b, byte((encoding>>i)&0xff))
	}
	return b
}

func printMetadata(w io.Writer, n *node) error {
	const header = `// generated by go run gen.go; DO NOT EDIT

package publicsuffix

import _ "embed"

const version = %q

const (
	nodesBits           = %d
	nodesBitsChildren   = %d
	nodesBitsICANN      = %d
	nodesBitsTextOffset = %d
	nodesBitsTextLength = %d

	childrenBitsWildcard = %d
	childrenBitsNodeType = %d
	childrenBitsHi       = %d
	childrenBitsLo       = %d
)

const (
	nodeTypeNormal     = %d
	nodeTypeException  = %d
	nodeTypeParentOnly = %d
)

// numTLD is the number of top level domains.
const numTLD = %d

// text is the combined text of all labels.
//
//go:embed data/text
var text string

`
	fmt.Fprintf(w, header, *version,
		nodesBits,
		nodesBitsChildren, nodesBitsICANN, nodesBitsTextOffset, nodesBitsTextLength,
		childrenBitsWildcard, childrenBitsNodeType, childrenBitsHi, childrenBitsLo,
		nodeTypeNormal, nodeTypeException, nodeTypeParentOnly, len(n.children))
	fmt.Fprintf(w, `
// nodes is the list of nodes. Each node is represented as a %v-bit integer,
// which encodes the node's children, wildcard bit and node type (as an index
// into the children array), ICANN bit and text.
//
// The layout within the node, from MSB to LSB, is:
//	[%2d bits] unused
//	[%2d bits] children index
//	[%2d bits] ICANN bit
//	[%2d bits] text index
//	[%2d bits] text length
//
//go:embed data/nodes
var nodes uint40String
`,
		nodesBits,
		nodesBits-nodesBitsChildren-nodesBitsICANN-nodesBitsTextOffset-nodesBitsTextLength,
		nodesBitsChildren, nodesBitsICANN, nodesBitsTextOffset, nodesBitsTextLength)
	fmt.Fprintf(w, `
// children is the list of nodes' children, the parent's wildcard bit and the
// parent's node type. If a node has no children then their children index
// will be in the range [0, 6), depending on the wildcard bit and node type.
//
// The layout within the uint32, from MSB to LSB, is:
//	[%2d bits] unused
//	[%2d bits] wildcard bit
//	[%2d bits] node type
//	[%2d bits] high nodes index (exclusive) of children
//	[%2d bits] low nodes index (inclusive) of children
//
//go:embed data/children
var children uint32String
`,
		32-childrenBitsWildcard-childrenBitsNodeType-childrenBitsHi-childrenBitsLo,
		childrenBitsWildcard, childrenBitsNodeType, childrenBitsHi, childrenBitsLo)

	fmt.Fprintf(w, "// max children %d (capacity %d)\n", maxChildren, 1<<nodesBitsChildren-1)
	fmt.Fprintf(w, "// max text offset %d (capacity %d)\n", maxTextOffset, 1<<nodesBitsTextOffset-1)
	fmt.Fprintf(w, "// max text length %d (capacity %d)\n", maxTextLength, 1<<nodesBitsTextLength-1)
	fmt.Fprintf(w, "// max hi %d (capacity %d)\n", maxHi, 1<<childrenBitsHi-1)
	fmt.Fprintf(w, "// max lo %d (capacity %d)\n", maxLo, 1<<childrenBitsLo-1)
	return nil
}

type node struct {
	label    string
	nodeType int
	icann    bool
	wildcard bool
	// nodesIndex and childrenIndex are the index of this node in the nodes
	// and the index of its children offset/length in the children arrays.
	nodesIndex, childrenIndex int
	// firstChild is the index of this node's first child, or zero if this
	// node has no children.
	firstChild int
	// children are the node's children, in strictly increasing node label order.
	children []*node
}

func (n *node) walk(f func(*node) error) error {
	if err := f(n); err != nil {
		return err
	}
	for _, c := range n.children {
		if err := c.walk(f); err != nil {
			return err
		}
	}
	return nil
}

// child returns the child of n with the given label. The child is created if
// it did not exist beforehand.
func (n *node) child(label string) *node {
	for _, c := range n.children {
		if c.label == label {
			return c
		}
	}
	c := &node{
		label:    label,
		nodeType: nodeTypeParentOnly,
		icann:    true,
	}
	n.children = append(n.children, c)
	sort.Sort(byLabel(n.children))
	return c
}

type byLabel []*node

func (b byLabel) Len() int           { return len(b) }
func (b byLabel) Swap(i, j int)      { b[i], b[j] = b[j], b[i] }
func (b byLabel) Less(i, j int) bool { return b[i].label < b[j].label }

var nextNodesIndex int

// childrenEncoding are the encoded entries in the generated children array.
// All these pre-defined entries have no children.
var childrenEncoding = []uint32{
	0 << (childrenBitsLo + childrenBitsHi), // Without wildcard bit, nodeTypeNormal.
	1 << (childrenBitsLo + childrenBitsHi), // Without wildcard bit, nodeTypeException.
	2 << (childrenBitsLo + childrenBitsHi), // Without wildcard bit, nodeTypeParentOnly.
	4 << (childrenBitsLo + childrenBitsHi), // With wildcard bit, nodeTypeNormal.
	5 << (childrenBitsLo + childrenBitsHi), // With wildcard bit, nodeTypeException.
	6 << (childrenBitsLo + childrenBitsHi), // With wildcard bit, nodeTypeParentOnly.
}

var firstCallToAssignIndexes = true

func assignIndexes(n *node) error {
	if len(n.children) != 0 {
		// Assign nodesIndex.
		n.firstChild = nextNodesIndex
		for _, c := range n.children {
			c.nodesIndex = nextNodesIndex
			nextNodesIndex++
		}

		// The root node's children is implicit.
		if firstCallToAssignIndexes {
			firstCallToAssignIndexes = false
			return nil
		}

		// Assign childrenIndex.
		maxChildren = max(maxChildren, len(childrenEncoding))
		if len(childrenEncoding) >= 1<<nodesBitsChildren {
			return fmt.Errorf("children table size %d is too large, or nodeBitsChildren is too small", len(childrenEncoding))
		}
		n.childrenIndex = len(childrenEncoding)
		lo := uint32(n.firstChild)
		hi := lo + uint32(len(n.children))
		maxLo, maxHi = u32max(maxLo, lo), u32max(maxHi, hi)
		if lo >= 1<<childrenBitsLo {
			return fmt.Errorf("children lo %d is too large, or childrenBitsLo is too small", lo)
		}
		if hi >= 1<<childrenBitsHi {
			return fmt.Errorf("children hi %d is too large, or childrenBitsHi is too small", hi)
		}
		enc := hi<<childrenBitsLo | lo
		enc |= uint32(n.nodeType) << (childrenBitsLo + childrenBitsHi)
		if n.wildcard {
			enc |= 1 << (childrenBitsLo + childrenBitsHi + childrenBitsNodeType)
		}
		childrenEncoding = append(childrenEncoding, enc)
	} else {
		n.childrenIndex = n.nodeType
		if n.wildcard {
			n.childrenIndex += numNodeType
		}
	}
	return nil
}

func printNodeLabel(w io.Writer, n *node) error {
	for _, c := range n.children {
		fmt.Fprintf(w, "%q,\n", c.label)
	}
	return nil
}

func icannStr(icann bool) string {
	if icann {
		return "I"
	}
	return " "
}

func wildcardStr(wildcard bool) string {
	if wildcard {
		return "*"
	}
	return " "
}

// combineText combines all the strings in labelsList to form one giant string.
// Overlapping strings will be merged: "arpa" and "parliament" could yield
// "arparliament".
func combineText(labelsList []string) string {
	beforeLength := 0
	for _, s := range labelsList {
		beforeLength += len(s)
	}

	text := crush(removeSubstrings(labelsList))
	if *v {
		fmt.Fprintf(os.Stderr, "crushed %d bytes to become %d bytes\n", beforeLength, len(text))
	}
	return text
}

type byLength []string

func (s byLength) Len() int           { return len(s) }
func (s byLength) Swap(i, j int)      { s[i], s[j] = s[j], s[i] }
func (s byLength) Less(i, j int) bool { return len(s[i]) < len(s[j]) }

// removeSubstrings returns a copy of its input with any strings removed
// that are substrings of other provided strings.
func removeSubstrings(input []string) []string {
	// Make a copy of input.
	ss := append(make([]string, 0, len(input)), input...)
	sort.Sort(byLength(ss))

	for i, shortString := range ss {
		// For each string, only consider strings higher than it in sort order, i.e.
		// of equal length or greater.
		for _, longString := range ss[i+1:] {
			if strings.Contains(longString, shortString) {
				ss[i] = ""
				break
			}
		}
	}

	// Remove the empty strings.
	sort.Strings(ss)
	for len(ss) > 0 && ss[0] == "" {
		ss = ss[1:]
	}
	return ss
}

// crush combines a list of strings, taking advantage of overlaps. It returns a
// single string that contains each input string as a substring.
func crush(ss []string) string {
	maxLabelLen := 0
	for _, s := range ss {
		if maxLabelLen < len(s) {
			maxLabelLen = len(s)
		}
	}

	for prefixLen := maxLabelLen; prefixLen > 0; prefixLen-- {
		prefixes := makePrefixMap(ss, prefixLen)
		for i, s := range ss {
			if len(s) <= prefixLen {
				continue
			}
			mergeLabel(ss, i, prefixLen, prefixes)
		}
	}

	return strings.Join(ss, "")
}

// mergeLabel merges the label at ss[i] with the first available matching label
// in prefixMap, where the last "prefixLen" characters in ss[i] match the first
// "prefixLen" characters in the matching label.
// It will merge ss[i] repeatedly until no more matches are available.
// All matching labels merged into ss[i] are replaced by "".
func mergeLabel(ss []string, i, prefixLen int, prefixes prefixMap) {
	s := ss[i]
	suffix := s[len(s)-prefixLen:]
	for _, j := range prefixes[suffix] {
		// Empty strings mean "already used." Also avoid merging with self.
		if ss[j] == "" || i == j {
			continue
		}
		if *v {
			fmt.Fprintf(os.Stderr, "%d-length overlap at (%4d,%4d): %q and %q share %q\n",
				prefixLen, i, j, ss[i], ss[j], suffix)
		}
		ss[i] += ss[j][prefixLen:]
		ss[j] = ""
		// ss[i] has a new suffix, so merge again if possible.
		// Note: we only have to merge again at the same prefix length. Shorter
		// prefix lengths will be handled in the next iteration of crush's for loop.
		// Can there be matches for longer prefix lengths, introduced by the merge?
		// I believe that any such matches would by necessity have been eliminated
		// during substring removal or merged at a higher prefix length. For
		// instance, in crush("abc", "cde", "bcdef"), combining "abc" and "cde"
		// would yield "abcde", which could be merged with "bcdef." However, in
		// practice "cde" would already have been elimintated by removeSubstrings.
		mergeLabel(ss, i, prefixLen, prefixes)
		return
	}
}

// prefixMap maps from a prefix to a list of strings containing that prefix. The
// list of strings is represented as indexes into a slice of strings stored
// elsewhere.
type prefixMap map[string][]int

// makePrefixMap constructs a prefixMap from a slice of strings.
func makePrefixMap(ss []string, prefixLen int) prefixMap {
	prefixes := make(prefixMap)
	for i, s := range ss {
		// We use < rather than <= because if a label matches on a prefix equal to
		// its full length, that's actually a substring match handled by
		// removeSubstrings.
		if prefixLen < len(s) {
			prefix := s[:prefixLen]
			prefixes[prefix] = append(prefixes[prefix], i)
		}
	}

	return prefixes
}
