// Package filepathx adds double-star globbing support to the Glob function from the core path/filepath package.
// You might recognize "**" recursive globs from things like your .gitignore file, and zsh.
// The "**" glob represents a recursive wildcard matching zero-or-more directory levels deep.
package filepathx

import (
	"os"
	"path/filepath"
	"strings"
)

// Globs represents one filepath glob, with its elements joined by "**".
type Globs []string

// Glob adds double-star support to the core path/filepath Glob function.
// It's useful when your globs might have double-stars, but you're not sure.
func Glob(pattern string) ([]string, error) {
	if !strings.Contains(pattern, "**") {
		// passthru to core package if no double-star
		return filepath.Glob(pattern)
	}
	return Globs(strings.Split(pattern, "**")).Expand()
}

// Expand finds matches for the provided Globs.
func (globs Globs) Expand() ([]string, error) {
	var matches = []string{""} // accumulate here
	for _, glob := range globs {
		var hits []string
		var hitMap = map[string]bool{}
		for _, match := range matches {
			paths, err := filepath.Glob(match + glob)
			if err != nil {
				return nil, err
			}
			for _, path := range paths {
				err = filepath.Walk(path, func(path string, info os.FileInfo, err error) error {
					if err != nil {
						return err
					}
					// save deduped match from current iteration
					if _, ok := hitMap[path]; !ok {
						hits = append(hits, path)
						hitMap[path] = true
					}
					return nil
				})
				if err != nil {
					return nil, err
				}
			}
		}
		matches = hits
	}

	// fix up return value for nil input
	if globs == nil && len(matches) > 0 && matches[0] == "" {
		matches = matches[1:]
	}

	return matches, nil
}
