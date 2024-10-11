package proposal

import (
	govtypes "cosmossdk.io/x/gov/types/v1beta1"
)

const (
	// ProposalTypeChange defines the type for a ParameterChangeProposal
	ProposalTypeChange = "ParameterChange"
)

// Assert ParameterChangeProposal implements govtypes.Content at compile-time
var _ govtypes.Content = &ParameterChangeProposal{}

func init() {
	govtypes.RegisterProposalType(ProposalTypeChange)
}

func NewParameterChangeProposal(title, description string, changes []ParamChange) *ParameterChangeProposal {
	return &ParameterChangeProposal{title, description, changes}
}

// GetTitle returns the title of a parameter change proposal.
func (pcp *ParameterChangeProposal) GetTitle() string { return pcp.Title }

// GetDescription returns the description of a parameter change proposal.
func (pcp *ParameterChangeProposal) GetDescription() string { return pcp.Description }

// ProposalRoute returns the routing key of a parameter change proposal.
func (pcp *ParameterChangeProposal) ProposalRoute() string { return RouterKey }

// ProposalType returns the type of a parameter change proposal.
func (pcp *ParameterChangeProposal) ProposalType() string { return ProposalTypeChange }

// ValidateBasic validates the parameter change proposal
func (pcp *ParameterChangeProposal) ValidateBasic() error {
	err := govtypes.ValidateAbstract(pcp)
	if err != nil {
		return err
	}

	return ValidateChanges(pcp.Changes)
}

func NewParamChange(subspace, key, value string) ParamChange {
	return ParamChange{subspace, key, value}
}

// ValidateChanges performs basic validation checks over a set of ParamChange. It
// returns an error if any ParamChange is invalid.
func ValidateChanges(changes []ParamChange) error {
	if len(changes) == 0 {
		return ErrEmptyChanges
	}

	for _, pc := range changes {
		if len(pc.Subspace) == 0 {
			return ErrEmptySubspace
		}
		if len(pc.Key) == 0 {
			return ErrEmptyKey
		}
		if len(pc.Value) == 0 {
			return ErrEmptyValue
		}
	}

	return nil
}
