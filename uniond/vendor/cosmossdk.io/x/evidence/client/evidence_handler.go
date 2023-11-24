package client

import "github.com/spf13/cobra"

type (
	// CLIHandlerFn defines a CLI command handler for evidence submission
	CLIHandlerFn func() *cobra.Command

	// EvidenceHandler wraps CLIHandlerFn.
	EvidenceHandler struct {
		CLIHandler CLIHandlerFn
	}
)

// NewEvidenceHandler returns an EvidenceHandler.
func NewEvidenceHandler(cliHandler CLIHandlerFn) EvidenceHandler {
	return EvidenceHandler{
		CLIHandler: cliHandler,
	}
}
