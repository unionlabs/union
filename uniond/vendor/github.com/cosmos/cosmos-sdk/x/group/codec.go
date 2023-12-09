package group

import (
	codectypes "github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/legacy"
	cdctypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/msgservice"
)

// RegisterLegacyAminoCodec registers all the necessary group module concrete
// types and interfaces with the provided codec reference.
// These types are used for Amino JSON serialization.
func RegisterLegacyAminoCodec(cdc *codectypes.LegacyAmino) {
	cdc.RegisterInterface((*DecisionPolicy)(nil), nil)
	cdc.RegisterConcrete(&ThresholdDecisionPolicy{}, "cosmos-sdk/ThresholdDecisionPolicy", nil)
	cdc.RegisterConcrete(&PercentageDecisionPolicy{}, "cosmos-sdk/PercentageDecisionPolicy", nil)

	legacy.RegisterAminoMsg(cdc, &MsgCreateGroup{}, "cosmos-sdk/MsgCreateGroup")
	legacy.RegisterAminoMsg(cdc, &MsgUpdateGroupMembers{}, "cosmos-sdk/MsgUpdateGroupMembers")
	legacy.RegisterAminoMsg(cdc, &MsgUpdateGroupAdmin{}, "cosmos-sdk/MsgUpdateGroupAdmin")
	legacy.RegisterAminoMsg(cdc, &MsgUpdateGroupMetadata{}, "cosmos-sdk/MsgUpdateGroupMetadata")
	legacy.RegisterAminoMsg(cdc, &MsgCreateGroupWithPolicy{}, "cosmos-sdk/MsgCreateGroupWithPolicy")
	legacy.RegisterAminoMsg(cdc, &MsgCreateGroupPolicy{}, "cosmos-sdk/MsgCreateGroupPolicy")
	legacy.RegisterAminoMsg(cdc, &MsgUpdateGroupPolicyAdmin{}, "cosmos-sdk/MsgUpdateGroupPolicyAdmin")
	legacy.RegisterAminoMsg(cdc, &MsgUpdateGroupPolicyDecisionPolicy{}, "cosmos-sdk/MsgUpdateGroupDecisionPolicy")
	legacy.RegisterAminoMsg(cdc, &MsgUpdateGroupPolicyMetadata{}, "cosmos-sdk/MsgUpdateGroupPolicyMetadata")
	legacy.RegisterAminoMsg(cdc, &MsgSubmitProposal{}, "cosmos-sdk/group/MsgSubmitProposal")
	legacy.RegisterAminoMsg(cdc, &MsgWithdrawProposal{}, "cosmos-sdk/group/MsgWithdrawProposal")
	legacy.RegisterAminoMsg(cdc, &MsgVote{}, "cosmos-sdk/group/MsgVote")
	legacy.RegisterAminoMsg(cdc, &MsgExec{}, "cosmos-sdk/group/MsgExec")
	legacy.RegisterAminoMsg(cdc, &MsgLeaveGroup{}, "cosmos-sdk/group/MsgLeaveGroup")
}

// RegisterInterfaces registers the interfaces types with the interface registry.
func RegisterInterfaces(registry cdctypes.InterfaceRegistry) {
	registry.RegisterImplementations((*sdk.Msg)(nil),
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

	msgservice.RegisterMsgServiceDesc(registry, &_Msg_serviceDesc)

	registry.RegisterInterface(
		"cosmos.group.v1.DecisionPolicy",
		(*DecisionPolicy)(nil),
		&ThresholdDecisionPolicy{},
		&PercentageDecisionPolicy{},
	)
}
