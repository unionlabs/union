//go:build go1.21

package sentry

import (
	"testing"
)

func Test_cleanupFunctionNamePrefix(t *testing.T) {
	cases := map[string]struct {
		f    []Frame
		want []Frame
	}{
		"SimpleCase": {
			f: []Frame{
				{Function: "main.main"},
				{Function: "main.main.func1"},
			},
			want: []Frame{
				{Function: "main.main"},
				{Function: "func1"},
			},
		},
		"MultipleLevels": {
			f: []Frame{
				{Function: "main.main"},
				{Function: "main.main.func1"},
				{Function: "main.main.func1.func2"},
			},
			want: []Frame{
				{Function: "main.main"},
				{Function: "func1"},
				{Function: "func2"},
			},
		},
		"PrefixWithRun": {
			f: []Frame{
				{Function: "Run.main"},
				{Function: "Run.main.func1"},
			},
			want: []Frame{
				{Function: "Run.main"},
				{Function: "func1"},
			},
		},
		"NoPrefixMatch": {
			f: []Frame{
				{Function: "main.main"},
				{Function: "main.handler"},
			},
			want: []Frame{
				{Function: "main.main"},
				{Function: "main.handler"},
			},
		},
		"SingleFrame": {
			f: []Frame{
				{Function: "main.main"},
			},
			want: []Frame{
				{Function: "main.main"},
			},
		},
		"ComplexPrefix": {
			f: []Frame{
				{Function: "app.package.Run"},
				{Function: "app.package.Run.Logger.func1"},
			},
			want: []Frame{
				{Function: "app.package.Run"},
				{Function: "Logger.func1"},
			},
		},
	}
	for name, tt := range cases {
		t.Run(name, func(t *testing.T) {
			got := cleanupFunctionNamePrefix(tt.f)
			assertEqual(t, got, tt.want)
		})
	}
}
