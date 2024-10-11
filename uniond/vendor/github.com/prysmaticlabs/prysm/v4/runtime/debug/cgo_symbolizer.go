//go:build cgosymbolizer_enabled

package debug

import (
	// Using this file with this imported library configures the process to
	// expose cgo symbols when possible. Use --config=cgo_symbolizer to make use of
	// this feature.
	_ "github.com/ianlancetaylor/cgosymbolizer"
)
