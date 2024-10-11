package client

import (
	"github.com/spf13/cobra"
)

// function to create the cli handler
type CLIHandlerFn func() *cobra.Command

// ProposalHandler wraps CLIHandlerFn
type ProposalHandler struct {
	CLIHandler CLIHandlerFn
}

// NewProposalHandler creates a new ProposalHandler object
func NewProposalHandler(cliHandler CLIHandlerFn) ProposalHandler {
	return ProposalHandler{
		CLIHandler: cliHandler,
	}
}
