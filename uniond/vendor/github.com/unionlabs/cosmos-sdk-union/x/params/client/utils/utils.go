package utils

import (
	"encoding/json"
	"os"

	"cosmossdk.io/x/params/types/proposal"

	"github.com/cosmos/cosmos-sdk/codec"
)

type (
	// ParamChangesJSON defines a slice of ParamChangeJSON objects which can be
	// converted to a slice of ParamChange objects.
	ParamChangesJSON []ParamChangeJSON

	// ParamChangeJSON defines a parameter change used in JSON input. This
	// allows values to be specified in raw JSON instead of being string encoded.
	ParamChangeJSON struct {
		Subspace string          `json:"subspace" yaml:"subspace"`
		Key      string          `json:"key" yaml:"key"`
		Value    json.RawMessage `json:"value" yaml:"value"`
	}

	// ParamChangeProposalJSON defines a ParameterChangeProposal with a deposit used
	// to parse parameter change proposals from a JSON file.
	ParamChangeProposalJSON struct {
		Title       string           `json:"title" yaml:"title"`
		Description string           `json:"description" yaml:"description"`
		Changes     ParamChangesJSON `json:"changes" yaml:"changes"`
		Deposit     string           `json:"deposit" yaml:"deposit"`
	}
)

func NewParamChangeJSON(subspace, key string, value json.RawMessage) ParamChangeJSON {
	return ParamChangeJSON{subspace, key, value}
}

// ToParamChange converts a ParamChangeJSON object to ParamChange.
func (pcj ParamChangeJSON) ToParamChange() proposal.ParamChange {
	return proposal.NewParamChange(pcj.Subspace, pcj.Key, string(pcj.Value))
}

// ToParamChanges converts a slice of ParamChangeJSON objects to a slice of
// ParamChange.
func (pcj ParamChangesJSON) ToParamChanges() []proposal.ParamChange {
	res := make([]proposal.ParamChange, len(pcj))
	for i, pc := range pcj {
		res[i] = pc.ToParamChange()
	}
	return res
}

// ParseParamChangeProposalJSON reads and parses a ParamChangeProposalJSON from
// file.
func ParseParamChangeProposalJSON(cdc *codec.LegacyAmino, proposalFile string) (ParamChangeProposalJSON, error) {
	proposal := ParamChangeProposalJSON{}

	contents, err := os.ReadFile(proposalFile)
	if err != nil {
		return proposal, err
	}

	if err := cdc.UnmarshalJSON(contents, &proposal); err != nil {
		return proposal, err
	}

	return proposal, nil
}
