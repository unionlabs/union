// Copyright © 2016 Steve Francia <spf@spf13.com>.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file.

// Viper is a application configuration system.
// It believes that applications can be configured a variety of ways
// via flags, ENVIRONMENT variables, configuration files retrieved
// from the file system, or a remote key/value store.

package viper

import (
	"os"
	"path/filepath"
	"testing"

	slog "github.com/sagikazarmark/slog-shim"
	"github.com/stretchr/testify/assert"
)

func TestCopyAndInsensitiviseMap(t *testing.T) {
	var (
		given = map[string]any{
			"Foo": 32,
			"Bar": map[any]any{
				"ABc": "A",
				"cDE": "B",
			},
		}
		expected = map[string]any{
			"foo": 32,
			"bar": map[string]any{
				"abc": "A",
				"cde": "B",
			},
		}
	)

	got := copyAndInsensitiviseMap(given)

	assert.Equal(t, expected, got)
	_, ok := given["foo"]
	assert.False(t, ok)
	_, ok = given["bar"]
	assert.False(t, ok)

	m := given["Bar"].(map[any]any)
	_, ok = m["ABc"]
	assert.True(t, ok)
}

func TestAbsPathify(t *testing.T) {
	skipWindows(t)

	home := userHomeDir()
	homer := filepath.Join(home, "homer")
	wd, _ := os.Getwd()

	t.Setenv("HOMER_ABSOLUTE_PATH", homer)
	t.Setenv("VAR_WITH_RELATIVE_PATH", "relative")

	tests := []struct {
		input  string
		output string
	}{
		{"", wd},
		{"sub", filepath.Join(wd, "sub")},
		{"./", wd},
		{"./sub", filepath.Join(wd, "sub")},
		{"$HOME", home},
		{"$HOME/", home},
		{"$HOME/sub", filepath.Join(home, "sub")},
		{"$HOMER_ABSOLUTE_PATH", homer},
		{"$HOMER_ABSOLUTE_PATH/", homer},
		{"$HOMER_ABSOLUTE_PATH/sub", filepath.Join(homer, "sub")},
		{"$VAR_WITH_RELATIVE_PATH", filepath.Join(wd, "relative")},
		{"$VAR_WITH_RELATIVE_PATH/", filepath.Join(wd, "relative")},
		{"$VAR_WITH_RELATIVE_PATH/sub", filepath.Join(wd, "relative", "sub")},
	}

	for _, test := range tests {
		got := absPathify(slog.Default(), test.input)
		assert.Equal(t, test.output, got)
	}
}
