package v1

import (
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdktx "github.com/cosmos/cosmos-sdk/types/tx"
	"github.com/cosmos/cosmos-sdk/x/gov/types/v1beta1"
)

var (
	_, _, _, _, _, _, _ sdk.Msg                            = &MsgSubmitProposal{}, &MsgDeposit{}, &MsgVote{}, &MsgVoteWeighted{}, &MsgExecLegacyContent{}, &MsgUpdateParams{}, &MsgCancelProposal{}
	_, _                codectypes.UnpackInterfacesMessage = &MsgSubmitProposal{}, &MsgExecLegacyContent{}
)

// NewMsgSubmitProposal creates a new MsgSubmitProposal.
func NewMsgSubmitProposal(
	messages []sdk.Msg,
	initialDeposit sdk.Coins,
	proposer, metadata, title, summary string,
	expedited bool,
) (*MsgSubmitProposal, error) {
	m := &MsgSubmitProposal{
		InitialDeposit: initialDeposit,
		Proposer:       proposer,
		Metadata:       metadata,
		Title:          title,
		Summary:        summary,
		Expedited:      expedited,
	}

	anys, err := sdktx.SetMsgs(messages)
	if err != nil {
		return nil, err
	}

	m.Messages = anys

	return m, nil
}

// GetMsgs unpacks m.Messages Any's into sdk.Msg's
func (m *MsgSubmitProposal) GetMsgs() ([]sdk.Msg, error) {
	return sdktx.GetMsgs(m.Messages, "sdk.MsgProposal")
}

// SetMsgs packs sdk.Msg's into m.Messages Any's
// NOTE: this will overwrite any existing messages
func (m *MsgSubmitProposal) SetMsgs(msgs []sdk.Msg) error {
	anys, err := sdktx.SetMsgs(msgs)
	if err != nil {
		return err
	}

	m.Messages = anys
	return nil
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (m MsgSubmitProposal) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	return sdktx.UnpackInterfaces(unpacker, m.Messages)
}

// NewMsgDeposit creates a new MsgDeposit instance
func NewMsgDeposit(depositor sdk.AccAddress, proposalID uint64, amount sdk.Coins) *MsgDeposit {
	return &MsgDeposit{proposalID, depositor.String(), amount}
}

// NewMsgVote creates a message to cast a vote on an active proposal
func NewMsgVote(voter sdk.AccAddress, proposalID uint64, option VoteOption, metadata string) *MsgVote {
	return &MsgVote{proposalID, voter.String(), option, metadata}
}

// NewMsgVoteWeighted creates a message to cast a vote on an active proposal
func NewMsgVoteWeighted(voter sdk.AccAddress, proposalID uint64, options WeightedVoteOptions, metadata string) *MsgVoteWeighted {
	return &MsgVoteWeighted{proposalID, voter.String(), options, metadata}
}

// NewMsgExecLegacyContent creates a new MsgExecLegacyContent instance.
func NewMsgExecLegacyContent(content *codectypes.Any, authority string) *MsgExecLegacyContent {
	return &MsgExecLegacyContent{
		Content:   content,
		Authority: authority,
	}
}

// ValidateBasic implements the sdk.Msg interface.
func (c MsgExecLegacyContent) ValidateBasic() error {
	_, err := sdk.AccAddressFromBech32(c.Authority)
	if err != nil {
		return err
	}

	return nil
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (c MsgExecLegacyContent) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	var content v1beta1.Content
	return unpacker.UnpackAny(c.Content, &content)
}

// NewMsgCancelProposal creates a new MsgCancelProposal instance.
func NewMsgCancelProposal(proposalID uint64, proposer string) *MsgCancelProposal {
	return &MsgCancelProposal{
		ProposalId: proposalID,
		Proposer:   proposer,
	}
}
