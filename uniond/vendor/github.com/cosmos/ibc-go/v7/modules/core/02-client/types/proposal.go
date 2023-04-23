package types

import (
	"fmt"
	"reflect"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	govtypes "github.com/cosmos/cosmos-sdk/x/gov/types/v1beta1"
	upgradetypes "github.com/cosmos/cosmos-sdk/x/upgrade/types"

	"github.com/cosmos/ibc-go/v7/modules/core/exported"
)

const (
	// ProposalTypeClientUpdate defines the type for a ClientUpdateProposal
	ProposalTypeClientUpdate = "ClientUpdate"
	ProposalTypeUpgrade      = "IBCUpgrade"
)

var (
	_ govtypes.Content                   = &ClientUpdateProposal{}
	_ govtypes.Content                   = &UpgradeProposal{}
	_ codectypes.UnpackInterfacesMessage = &UpgradeProposal{}
)

func init() {
	govtypes.RegisterProposalType(ProposalTypeClientUpdate)
	govtypes.RegisterProposalType(ProposalTypeUpgrade)
}

// NewClientUpdateProposal creates a new client update proposal.
func NewClientUpdateProposal(title, description, subjectClientID, substituteClientID string) govtypes.Content {
	return &ClientUpdateProposal{
		Title:              title,
		Description:        description,
		SubjectClientId:    subjectClientID,
		SubstituteClientId: substituteClientID,
	}
}

// GetTitle returns the title of a client update proposal.
func (cup *ClientUpdateProposal) GetTitle() string { return cup.Title }

// GetDescription returns the description of a client update proposal.
func (cup *ClientUpdateProposal) GetDescription() string { return cup.Description }

// ProposalRoute returns the routing key of a client update proposal.
func (cup *ClientUpdateProposal) ProposalRoute() string { return RouterKey }

// ProposalType returns the type of a client update proposal.
func (cup *ClientUpdateProposal) ProposalType() string { return ProposalTypeClientUpdate }

// ValidateBasic runs basic stateless validity checks
func (cup *ClientUpdateProposal) ValidateBasic() error {
	err := govtypes.ValidateAbstract(cup)
	if err != nil {
		return err
	}

	if cup.SubjectClientId == cup.SubstituteClientId {
		return sdkerrors.Wrap(ErrInvalidSubstitute, "subject and substitute client identifiers are equal")
	}
	if _, _, err := ParseClientIdentifier(cup.SubjectClientId); err != nil {
		return err
	}
	if _, _, err := ParseClientIdentifier(cup.SubstituteClientId); err != nil {
		return err
	}

	return nil
}

// NewUpgradeProposal creates a new IBC breaking upgrade proposal.
func NewUpgradeProposal(title, description string, plan upgradetypes.Plan, upgradedClientState exported.ClientState) (govtypes.Content, error) {
	any, err := PackClientState(upgradedClientState)
	if err != nil {
		return nil, err
	}

	return &UpgradeProposal{
		Title:               title,
		Description:         description,
		Plan:                plan,
		UpgradedClientState: any,
	}, nil
}

// GetTitle returns the title of a upgrade proposal.
func (up *UpgradeProposal) GetTitle() string { return up.Title }

// GetDescription returns the description of a upgrade proposal.
func (up *UpgradeProposal) GetDescription() string { return up.Description }

// ProposalRoute returns the routing key of a upgrade proposal.
func (up *UpgradeProposal) ProposalRoute() string { return RouterKey }

// ProposalType returns the upgrade proposal type.
func (up *UpgradeProposal) ProposalType() string { return ProposalTypeUpgrade }

// ValidateBasic runs basic stateless validity checks
func (up *UpgradeProposal) ValidateBasic() error {
	if err := govtypes.ValidateAbstract(up); err != nil {
		return err
	}

	if err := up.Plan.ValidateBasic(); err != nil {
		return err
	}

	if up.UpgradedClientState == nil {
		return sdkerrors.Wrap(ErrInvalidUpgradeProposal, "upgraded client state cannot be nil")
	}

	clientState, err := UnpackClientState(up.UpgradedClientState)
	if err != nil {
		return sdkerrors.Wrap(err, "failed to unpack upgraded client state")
	}

	if !reflect.DeepEqual(clientState, clientState.ZeroCustomFields()) {
		return sdkerrors.Wrap(ErrInvalidUpgradeProposal, "upgraded client state is not zeroed out")
	}

	return nil
}

// String returns the string representation of the UpgradeProposal.
func (up UpgradeProposal) String() string {
	var upgradedClientStr string
	upgradedClient, err := UnpackClientState(up.UpgradedClientState)
	if err != nil {
		upgradedClientStr = "invalid IBC Client State"
	} else {
		upgradedClientStr = upgradedClient.String()
	}

	return fmt.Sprintf(`IBC Upgrade Proposal
  Title: %s
  Description: %s
  %s
  Upgraded IBC Client: %s`, up.Title, up.Description, up.Plan, upgradedClientStr)
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (up UpgradeProposal) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	return unpacker.UnpackAny(up.UpgradedClientState, new(exported.ClientState))
}
