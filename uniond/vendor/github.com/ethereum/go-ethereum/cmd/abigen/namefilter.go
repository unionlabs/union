package main

import (
	"fmt"
	"strings"
)

type nameFilter struct {
	fulls map[string]bool // path/to/contract.sol:Type
	files map[string]bool // path/to/contract.sol:*
	types map[string]bool // *:Type
}

func newNameFilter(patterns ...string) (*nameFilter, error) {
	f := &nameFilter{
		fulls: make(map[string]bool),
		files: make(map[string]bool),
		types: make(map[string]bool),
	}
	for _, pattern := range patterns {
		if err := f.add(pattern); err != nil {
			return nil, err
		}
	}
	return f, nil
}

func (f *nameFilter) add(pattern string) error {
	ft := strings.Split(pattern, ":")
	if len(ft) != 2 {
		// filenames and types must not include ':' symbol
		return fmt.Errorf("invalid pattern: %s", pattern)
	}

	file, typ := ft[0], ft[1]
	if file == "*" {
		f.types[typ] = true
		return nil
	} else if typ == "*" {
		f.files[file] = true
		return nil
	}
	f.fulls[pattern] = true
	return nil
}

func (f *nameFilter) Matches(name string) bool {
	ft := strings.Split(name, ":")
	if len(ft) != 2 {
		// If contract names are always of the fully-qualified form
		// <filePath>:<type>, then this case will never happen.
		return false
	}

	file, typ := ft[0], ft[1]
	// full paths > file paths > types
	return f.fulls[name] || f.files[file] || f.types[typ]
}
