package main

import (
	"bufio"
	"bytes"
	"fmt"
	"io"
	"regexp"
	"strings"
)

// tocMarker is a marker placed in the output to indicate where the table of
// contents should be inserted.
const tocMarker = `<!--- TABLE OF CONTENTS --->`

// heading in a markdown file.
type heading struct {
	level int
	text  string
}

var headingx = regexp.MustCompile(`^(#+)\s+(.+)$`)

// headings extracts headings from the markdown stream.
func headings(r io.Reader) ([]heading, error) {
	var hs []heading
	s := bufio.NewScanner(r)
	for s.Scan() {
		match := headingx.FindStringSubmatch(s.Text())
		if match != nil {
			hs = append(hs, heading{
				level: len(match[1]),
				text:  match[2],
			})
		}
	}
	if err := s.Err(); err != nil {
		return nil, err
	}
	return hs, nil
}

// generateTOC inserts a table of contents into body. Table of contents includes
// headings with the given range of levels.
func generateTOC(body []byte, minlevel, maxlevel int) ([]byte, error) {
	r := bytes.NewReader(body)

	// Extract headings.
	hs, err := headings(r)
	if err != nil {
		return nil, err
	}

	// Build table of contents.
	var buf bytes.Buffer
	fmt.Fprint(&buf, "## Table of Contents\n\n")
	for _, h := range hs {
		if h.level >= minlevel && h.level <= maxlevel {
			indent := strings.Repeat("  ", h.level-minlevel)
			fmt.Fprintf(&buf, "%s* [%s](#%s)\n", indent, h.text, anchor(h.text))
		}
	}

	// Replace marker.
	body = bytes.ReplaceAll(body, []byte(tocMarker), buf.Bytes())

	return body, nil
}

// toc is a template function that leaves a marker which can later be replaced.
func toc() string {
	return tocMarker
}
