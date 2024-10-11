package flags

// This defines flag names that can be used in autocli.
const (
	// FlagFrom is the flag to set the from address with which to sign the transaction.
	FlagFrom = "from"

	// FlagOutput is the flag to set the output format.
	FlagOutput = "output"

	// FlagNoIndent is the flag to not indent the output.
	FlagNoIndent = "no-indent"

	// FlagNoPrompt is the flag to not use a prompt for commands.
	FlagNoPrompt = "no-prompt"

	// FlagNoProposal is the flag convert a gov proposal command into a normal command.
	// This is used to allow user of chains with custom authority to not use gov submit proposals for usual proposal commands.
	FlagNoProposal = "no-proposal"
)

// List of supported output formats
const (
	OutputFormatJSON = "json"
	OutputFormatText = "text"
)
