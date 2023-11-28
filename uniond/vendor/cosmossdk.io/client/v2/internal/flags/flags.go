package flags

// This defines flag names that can be used in autocli.
const (
	// FlagFrom is the flag to set the from address with which to sign the transaction.
	FlagFrom = "from"

	// FlagOutput is the flag to set the output format.
	FlagOutput = "output"

	// FlagNoIndent is the flag to not indent the output.
	FlagNoIndent = "no-indent"
)

// List of supported output formats
const (
	OutputFormatJSON = "json"
	OutputFormatText = "text"
)
