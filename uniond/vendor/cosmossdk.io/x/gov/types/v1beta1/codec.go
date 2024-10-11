package v1beta1

import (
	"cosmossdk.io/core/registry"
	coretransaction "cosmossdk.io/core/transaction"

	"github.com/cosmos/cosmos-sdk/codec/legacy"
	"github.com/cosmos/cosmos-sdk/types/msgservice"
)

// RegisterLegacyAminoCodec registers all the necessary types and interfaces for the
// governance module.
func RegisterLegacyAminoCodec(registrar registry.AminoRegistrar) {
	registrar.RegisterInterface((*Content)(nil), nil)
	legacy.RegisterAminoMsg(registrar, &MsgSubmitProposal{}, "cosmos-sdk/MsgSubmitProposal")
	legacy.RegisterAminoMsg(registrar, &MsgDeposit{}, "cosmos-sdk/MsgDeposit")
	legacy.RegisterAminoMsg(registrar, &MsgVote{}, "cosmos-sdk/MsgVote")
	legacy.RegisterAminoMsg(registrar, &MsgVoteWeighted{}, "cosmos-sdk/MsgVoteWeighted")
	registrar.RegisterConcrete(&TextProposal{}, "cosmos-sdk/TextProposal")
}

// RegisterInterfaces registers the interfaces types with the Interface Registry.
func RegisterInterfaces(registrar registry.InterfaceRegistrar) {
	registrar.RegisterImplementations((*coretransaction.Msg)(nil),
		&MsgSubmitProposal{},
		&MsgVote{},
		&MsgVoteWeighted{},
		&MsgDeposit{},
	)
	registrar.RegisterInterface(
		"cosmos.gov.v1beta1.Content",
		(*Content)(nil),
		&TextProposal{},
	)

	msgservice.RegisterMsgServiceDesc(registrar, &_Msg_serviceDesc)
}
