package client

import (
	govclient "cosmossdk.io/x/gov/client"
	"cosmossdk.io/x/params/client/cli"
)

// ProposalHandler is the param change proposal handler.
var ProposalHandler = govclient.NewProposalHandler(cli.NewSubmitParamChangeProposalTxCmd)
