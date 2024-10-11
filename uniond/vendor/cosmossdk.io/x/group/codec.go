package group

import (
	"cosmossdk.io/core/registry"
	coretransaction "cosmossdk.io/core/transaction"

	"github.com/cosmos/cosmos-sdk/codec/legacy"
	"github.com/cosmos/cosmos-sdk/types/msgservice"
)

// RegisterLegacyAminoCodec registers all the necessary group module concrete
// types and interfaces with the provided codec reference.
// These types are used for Amino JSON serialization.
func RegisterLegacyAminoCodec(registrar registry.AminoRegistrar) {
	registrar.RegisterInterface((*DecisionPolicy)(nil), nil)
	registrar.RegisterConcrete(&ThresholdDecisionPolicy{}, "cosmos-sdk/ThresholdDecisionPolicy")
	registrar.RegisterConcrete(&PercentageDecisionPolicy{}, "cosmos-sdk/PercentageDecisionPolicy")

	legacy.RegisterAminoMsg(registrar, &MsgCreateGroup{}, "cosmos-sdk/MsgCreateGroup")
	legacy.RegisterAminoMsg(registrar, &MsgUpdateGroupMembers{}, "cosmos-sdk/MsgUpdateGroupMembers")
	legacy.RegisterAminoMsg(registrar, &MsgUpdateGroupAdmin{}, "cosmos-sdk/MsgUpdateGroupAdmin")
	legacy.RegisterAminoMsg(registrar, &MsgUpdateGroupMetadata{}, "cosmos-sdk/MsgUpdateGroupMetadata")
	legacy.RegisterAminoMsg(registrar, &MsgCreateGroupWithPolicy{}, "cosmos-sdk/MsgCreateGroupWithPolicy")
	legacy.RegisterAminoMsg(registrar, &MsgCreateGroupPolicy{}, "cosmos-sdk/MsgCreateGroupPolicy")
	legacy.RegisterAminoMsg(registrar, &MsgUpdateGroupPolicyAdmin{}, "cosmos-sdk/MsgUpdateGroupPolicyAdmin")
	legacy.RegisterAminoMsg(registrar, &MsgUpdateGroupPolicyDecisionPolicy{}, "cosmos-sdk/MsgUpdateGroupDecisionPolicy")
	legacy.RegisterAminoMsg(registrar, &MsgUpdateGroupPolicyMetadata{}, "cosmos-sdk/MsgUpdateGroupPolicyMetadata")
	legacy.RegisterAminoMsg(registrar, &MsgSubmitProposal{}, "cosmos-sdk/group/MsgSubmitProposal")
	legacy.RegisterAminoMsg(registrar, &MsgWithdrawProposal{}, "cosmos-sdk/group/MsgWithdrawProposal")
	legacy.RegisterAminoMsg(registrar, &MsgVote{}, "cosmos-sdk/group/MsgVote")
	legacy.RegisterAminoMsg(registrar, &MsgExec{}, "cosmos-sdk/group/MsgExec")
	legacy.RegisterAminoMsg(registrar, &MsgLeaveGroup{}, "cosmos-sdk/group/MsgLeaveGroup")
}

// RegisterInterfaces registers the interfaces types with the interface registry.
func RegisterInterfaces(registrar registry.InterfaceRegistrar) {
	registrar.RegisterImplementations((*coretransaction.Msg)(nil),
		&MsgCreateGroup{},
		&MsgUpdateGroupMembers{},
		&MsgUpdateGroupAdmin{},
		&MsgUpdateGroupMetadata{},
		&MsgCreateGroupWithPolicy{},
		&MsgCreateGroupPolicy{},
		&MsgUpdateGroupPolicyAdmin{},
		&MsgUpdateGroupPolicyDecisionPolicy{},
		&MsgUpdateGroupPolicyMetadata{},
		&MsgSubmitProposal{},
		&MsgWithdrawProposal{},
		&MsgVote{},
		&MsgExec{},
		&MsgLeaveGroup{},
	)

	msgservice.RegisterMsgServiceDesc(registrar, &_Msg_serviceDesc)

	registrar.RegisterInterface(
		"cosmos.group.v1.DecisionPolicy",
		(*DecisionPolicy)(nil),
		&ThresholdDecisionPolicy{},
		&PercentageDecisionPolicy{},
	)
}
