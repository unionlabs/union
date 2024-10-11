package v1

import (
	"errors"
	"fmt"

	gogoprotoany "github.com/cosmos/gogoproto/types/any"

	"cosmossdk.io/x/gov/types/v1beta1"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdktx "github.com/cosmos/cosmos-sdk/types/tx"
)

var (
	_, _, _, _, _, _, _, _ sdk.Msg                              = &MsgSubmitProposal{}, &MsgDeposit{}, &MsgVote{}, &MsgVoteWeighted{}, &MsgExecLegacyContent{}, &MsgUpdateParams{}, &MsgCancelProposal{}, &MsgSubmitMultipleChoiceProposal{}
	_, _                   gogoprotoany.UnpackInterfacesMessage = &MsgSubmitProposal{}, &MsgExecLegacyContent{}
)

// NewMsgSubmitProposal creates a new MsgSubmitProposal.
func NewMsgSubmitProposal(
	messages []sdk.Msg,
	initialDeposit sdk.Coins,
	proposer, metadata, title, summary string,
	proposalType ProposalType,
) (*MsgSubmitProposal, error) {
	m := &MsgSubmitProposal{
		InitialDeposit: initialDeposit,
		Proposer:       proposer,
		Metadata:       metadata,
		Title:          title,
		Summary:        summary,
		ProposalType:   proposalType,
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
func (m MsgSubmitProposal) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
	return sdktx.UnpackInterfaces(unpacker, m.Messages)
}

// NewMsgSubmitMultipleChoiceProposal creates a new MsgSubmitMultipleChoiceProposal.
func NewMultipleChoiceMsgSubmitProposal(
	initialDeposit sdk.Coins,
	proposer, metadata, title, summary string,
	votingOptions *ProposalVoteOptions,
) (*MsgSubmitMultipleChoiceProposal, error) {
	if votingOptions == nil {
		return nil, errors.New("voting options cannot be nil")
	}

	m := &MsgSubmitMultipleChoiceProposal{
		InitialDeposit: initialDeposit,
		Proposer:       proposer,
		Metadata:       metadata,
		Title:          title,
		Summary:        summary,
		VoteOptions:    votingOptions,
	}

	return m, nil
}

// NewMsgDeposit creates a new MsgDeposit instance
func NewMsgDeposit(depositor string, proposalID uint64, amount sdk.Coins) *MsgDeposit {
	return &MsgDeposit{proposalID, depositor, amount}
}

// NewMsgVote creates a message to cast a vote on an active proposal
func NewMsgVote(voter string, proposalID uint64, option VoteOption, metadata string) *MsgVote {
	return &MsgVote{proposalID, voter, option, metadata}
}

// NewMsgVoteWeighted creates a message to cast a vote on an active proposal
func NewMsgVoteWeighted(voter string, proposalID uint64, options WeightedVoteOptions, metadata string) *MsgVoteWeighted {
	return &MsgVoteWeighted{proposalID, voter, options, metadata}
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
func (c MsgExecLegacyContent) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
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

// GetSudoedMsg returns the cache values from the MsgSudoExec.Msg if present.
func (msg *MsgSudoExec) GetSudoedMsg() (sdk.Msg, error) {
	if msg.Msg == nil {
		return nil, errors.New("message is empty")
	}

	msgAny, ok := msg.Msg.GetCachedValue().(sdk.Msg)
	if !ok {
		return nil, fmt.Errorf("messages contains %T which is not a sdk.Msg", msgAny)
	}

	return msgAny, nil
}

// SetSudoedMsg sets a sdk.Msg into the MsgSudoExec.Msg.
func (msg *MsgSudoExec) SetSudoedMsg(input sdk.Msg) (*MsgSudoExec, error) {
	any, err := sdktx.SetMsg(input)
	if err != nil {
		return nil, err
	}
	msg.Msg = any

	return msg, nil
}
