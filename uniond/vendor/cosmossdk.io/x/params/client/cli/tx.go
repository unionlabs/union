package cli

import (
	"fmt"
	"strings"

	"github.com/spf13/cobra"

	govv1beta1 "cosmossdk.io/x/gov/types/v1beta1"
	paramscutils "cosmossdk.io/x/params/client/utils"
	paramproposal "cosmossdk.io/x/params/types/proposal"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/tx"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/version"
)

// NewSubmitParamChangeProposalTxCmd returns a CLI command handler for creating
// a parameter change proposal governance transaction.
func NewSubmitParamChangeProposalTxCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "param-change <proposal-file>",
		Args:  cobra.ExactArgs(1),
		Short: "Submit a parameter change proposal",
		Long: strings.TrimSpace(
			fmt.Sprintf(`Submit a parameter proposal along with an initial deposit.
The proposal details must be supplied via a JSON file. For values that contains
objects, only non-empty fields will be updated.

IMPORTANT: Currently parameter changes are evaluated but not validated, so it is
very important that any "value" change is valid (ie. correct type and within bounds)
for its respective parameter, eg. "MaxValidators" should be an integer and not a decimal.

Proper vetting of a parameter change proposal should prevent this from happening
(no deposits should occur during the governance process), but it should be noted
regardless.

Example:
$ %s tx gov submit-proposal param-change <path/to/proposal.json> --from=<key_or_address>

Where proposal.json contains:

{
  "title": "Staking Param Change",
  "description": "Update max validators",
  "changes": [
    {
      "subspace": "staking",
      "key": "MaxValidators",
      "value": 105
    }
  ],
  "deposit": "1000stake"
}
`,
				version.AppName,
			),
		),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}
			proposal, err := paramscutils.ParseParamChangeProposalJSON(clientCtx.LegacyAmino, args[0])
			if err != nil {
				return err
			}

			from, err := clientCtx.AddressCodec.BytesToString(clientCtx.GetFromAddress())
			if err != nil {
				return err
			}

			content := paramproposal.NewParameterChangeProposal(
				proposal.Title, proposal.Description, proposal.Changes.ToParamChanges(),
			)

			deposit, err := sdk.ParseCoinsNormalized(proposal.Deposit)
			if err != nil {
				return err
			}

			msg, err := govv1beta1.NewMsgSubmitProposal(content, deposit, from)
			if err != nil {
				return err
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}
}
