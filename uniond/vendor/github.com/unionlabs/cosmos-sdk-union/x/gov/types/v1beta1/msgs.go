package v1beta1

import (
	"fmt"

	"github.com/cosmos/gogoproto/proto"
	gogoprotoany "github.com/cosmos/gogoproto/types/any"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// Governance message types and routes
const (
	TypeMsgDeposit        = "deposit"
	TypeMsgVote           = "vote"
	TypeMsgVoteWeighted   = "weighted_vote"
	TypeMsgSubmitProposal = "submit_proposal"
)

var (
	_, _, _, _ sdk.Msg = &MsgSubmitProposal{}, &MsgDeposit{}, &MsgVote{}, &MsgVoteWeighted{}

	_ gogoprotoany.UnpackInterfacesMessage = &MsgSubmitProposal{}
)

// NewMsgSubmitProposal creates a new MsgSubmitProposal.
func NewMsgSubmitProposal(content Content, initialDeposit sdk.Coins, proposer string) (*MsgSubmitProposal, error) {
	m := &MsgSubmitProposal{
		InitialDeposit: initialDeposit,
		Proposer:       proposer,
	}
	err := m.SetContent(content)
	if err != nil {
		return nil, err
	}
	return m, nil
}

// GetInitialDeposit returns the initial deposit of MsgSubmitProposal.
func (m *MsgSubmitProposal) GetInitialDeposit() sdk.Coins { return m.InitialDeposit }

// GetContent returns the content of MsgSubmitProposal.
func (m *MsgSubmitProposal) GetContent() Content {
	content, ok := m.Content.GetCachedValue().(Content)
	if !ok {
		return nil
	}
	return content
}

// SetInitialDeposit sets the given initial deposit for MsgSubmitProposal.
func (m *MsgSubmitProposal) SetInitialDeposit(coins sdk.Coins) {
	m.InitialDeposit = coins
}

// SetProposer sets the given proposer address for MsgSubmitProposal.
func (m *MsgSubmitProposal) SetProposer(address string) {
	m.Proposer = address
}

// SetContent sets the content for MsgSubmitProposal.
func (m *MsgSubmitProposal) SetContent(content Content) error {
	msg, ok := content.(proto.Message)
	if !ok {
		return fmt.Errorf("can't proto marshal %T", msg)
	}
	any, err := codectypes.NewAnyWithValue(msg)
	if err != nil {
		return err
	}
	m.Content = any
	return nil
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (m MsgSubmitProposal) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
	var content Content
	return unpacker.UnpackAny(m.Content, &content)
}

// NewMsgDeposit creates a new MsgDeposit instance
func NewMsgDeposit(depositor string, proposalID uint64, amount sdk.Coins) *MsgDeposit {
	return &MsgDeposit{proposalID, depositor, amount}
}

// NewMsgVote creates a message to cast a vote on an active proposal
func NewMsgVote(voter string, proposalID uint64, option VoteOption) *MsgVote {
	return &MsgVote{proposalID, voter, option}
}

// NewMsgVoteWeighted creates a message to cast a vote on an active proposal.
func NewMsgVoteWeighted(voter string, proposalID uint64, options WeightedVoteOptions) *MsgVoteWeighted {
	return &MsgVoteWeighted{proposalID, voter, options}
}
