package consensus

import (
	"fmt"

	"github.com/cosmos/gogoproto/proto"

	"github.com/cometbft/cometbft/p2p"
)

var _ p2p.Wrapper = &VoteSetBits{}
var _ p2p.Wrapper = &VoteSetMaj23{}
var _ p2p.Wrapper = &Vote{}
var _ p2p.Wrapper = &ProposalPOL{}
var _ p2p.Wrapper = &Proposal{}
var _ p2p.Wrapper = &NewValidBlock{}
var _ p2p.Wrapper = &NewRoundStep{}
var _ p2p.Wrapper = &HasVote{}
var _ p2p.Wrapper = &BlockPart{}

func (m *VoteSetBits) Wrap() proto.Message {
	cm := &Message{}
	cm.Sum = &Message_VoteSetBits{VoteSetBits: m}
	return cm

}

func (m *VoteSetMaj23) Wrap() proto.Message {
	cm := &Message{}
	cm.Sum = &Message_VoteSetMaj23{VoteSetMaj23: m}
	return cm
}

func (m *HasVote) Wrap() proto.Message {
	cm := &Message{}
	cm.Sum = &Message_HasVote{HasVote: m}
	return cm
}

func (m *Vote) Wrap() proto.Message {
	cm := &Message{}
	cm.Sum = &Message_Vote{Vote: m}
	return cm
}

func (m *BlockPart) Wrap() proto.Message {
	cm := &Message{}
	cm.Sum = &Message_BlockPart{BlockPart: m}
	return cm
}

func (m *ProposalPOL) Wrap() proto.Message {
	cm := &Message{}
	cm.Sum = &Message_ProposalPol{ProposalPol: m}
	return cm
}

func (m *Proposal) Wrap() proto.Message {
	cm := &Message{}
	cm.Sum = &Message_Proposal{Proposal: m}
	return cm
}

func (m *NewValidBlock) Wrap() proto.Message {
	cm := &Message{}
	cm.Sum = &Message_NewValidBlock{NewValidBlock: m}
	return cm
}

func (m *NewRoundStep) Wrap() proto.Message {
	cm := &Message{}
	cm.Sum = &Message_NewRoundStep{NewRoundStep: m}
	return cm
}

// Unwrap implements the p2p Wrapper interface and unwraps a wrapped consensus
// proto message.
func (m *Message) Unwrap() (proto.Message, error) {
	switch msg := m.Sum.(type) {
	case *Message_NewRoundStep:
		return m.GetNewRoundStep(), nil

	case *Message_NewValidBlock:
		return m.GetNewValidBlock(), nil

	case *Message_Proposal:
		return m.GetProposal(), nil

	case *Message_ProposalPol:
		return m.GetProposalPol(), nil

	case *Message_BlockPart:
		return m.GetBlockPart(), nil

	case *Message_Vote:
		return m.GetVote(), nil

	case *Message_HasVote:
		return m.GetHasVote(), nil

	case *Message_VoteSetMaj23:
		return m.GetVoteSetMaj23(), nil

	case *Message_VoteSetBits:
		return m.GetVoteSetBits(), nil

	default:
		return nil, fmt.Errorf("unknown message: %T", msg)
	}
}
