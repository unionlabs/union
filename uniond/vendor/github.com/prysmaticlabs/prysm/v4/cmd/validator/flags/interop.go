package flags

import (
	"github.com/urfave/cli/v2"
)

// Flags defined for interoperability testing.
var (
	InteropStartIndex = &cli.Uint64Flag{
		Name: "interop-start-index",
		Usage: `Start index to deterministically generate validator keys when used in combination with --interop-num-validators.
		Example: --interop-start-index=5 --interop-num-validators=3 would generate keys from index 5 to 7.`,
	}
	InteropNumValidators = &cli.Uint64Flag{
		Name: "interop-num-validators",
		Usage: `Number of validators to deterministically generate.
		Example: --interop-start-index=5 --interop-num-validators=3 would generate keys from index 5 to 7.`,
	}
)
