package v1

import (
	"cosmossdk.io/core/registry"
	coretransaction "cosmossdk.io/core/transaction"

	"github.com/cosmos/cosmos-sdk/codec/legacy"
	"github.com/cosmos/cosmos-sdk/types/msgservice"
)

// RegisterLegacyAminoCodec registers all the necessary types and interfaces for the
// governance module.
func RegisterLegacyAminoCodec(registrar registry.AminoRegistrar) {
	legacy.RegisterAminoMsg(registrar, &MsgSubmitProposal{}, "cosmos-sdk/v1/MsgSubmitProposal")
	legacy.RegisterAminoMsg(registrar, &MsgSubmitMultipleChoiceProposal{}, "gov/MsgSubmitMultipleChoiceProposal")
	legacy.RegisterAminoMsg(registrar, &MsgDeposit{}, "cosmos-sdk/v1/MsgDeposit")
	legacy.RegisterAminoMsg(registrar, &MsgVote{}, "cosmos-sdk/v1/MsgVote")
	legacy.RegisterAminoMsg(registrar, &MsgVoteWeighted{}, "cosmos-sdk/v1/MsgVoteWeighted")
	legacy.RegisterAminoMsg(registrar, &MsgExecLegacyContent{}, "cosmos-sdk/v1/MsgExecLegacyContent")
	legacy.RegisterAminoMsg(registrar, &MsgUpdateParams{}, "cosmos-sdk/x/gov/v1/MsgUpdateParams")
	legacy.RegisterAminoMsg(registrar, &MsgUpdateMessageParams{}, "x/gov/v1/MsgUpdateMessageParams")
	legacy.RegisterAminoMsg(registrar, &MsgSudoExec{}, "cosmos-sdk/x/gov/v1/MsgSudoExec")
}

// RegisterInterfaces registers the interfaces types with the Interface Registry.
func RegisterInterfaces(registrar registry.InterfaceRegistrar) {
	registrar.RegisterImplementations((*coretransaction.Msg)(nil),
		&MsgSubmitProposal{},
		&MsgSubmitMultipleChoiceProposal{},
		&MsgVote{},
		&MsgVoteWeighted{},
		&MsgDeposit{},
		&MsgExecLegacyContent{},
		&MsgUpdateParams{},
		&MsgUpdateMessageParams{},
		&MsgSudoExec{},
	)

	msgservice.RegisterMsgServiceDesc(registrar, &_Msg_serviceDesc)
}
