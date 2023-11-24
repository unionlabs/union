package types

import (
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/msgservice"
	"github.com/cosmos/cosmos-sdk/x/authz"
	"github.com/cosmos/cosmos-sdk/x/gov/types/v1beta1"
)

// RegisterLegacyAminoCodec registers the concrete types and interface
func RegisterLegacyAminoCodec(cdc *codec.LegacyAmino) {
	cdc.RegisterConcrete(&MsgStoreCode{}, "wasm/MsgStoreCode", nil)
	cdc.RegisterConcrete(&MsgInstantiateContract{}, "wasm/MsgInstantiateContract", nil)
	cdc.RegisterConcrete(&MsgInstantiateContract2{}, "wasm/MsgInstantiateContract2", nil)
	cdc.RegisterConcrete(&MsgExecuteContract{}, "wasm/MsgExecuteContract", nil)
	cdc.RegisterConcrete(&MsgMigrateContract{}, "wasm/MsgMigrateContract", nil)
	cdc.RegisterConcrete(&MsgUpdateAdmin{}, "wasm/MsgUpdateAdmin", nil)
	cdc.RegisterConcrete(&MsgClearAdmin{}, "wasm/MsgClearAdmin", nil)
	cdc.RegisterConcrete(&MsgUpdateInstantiateConfig{}, "wasm/MsgUpdateInstantiateConfig", nil)
	cdc.RegisterConcrete(&MsgUpdateParams{}, "wasm/MsgUpdateParams", nil)
	cdc.RegisterConcrete(&MsgSudoContract{}, "wasm/MsgSudoContract", nil)
	cdc.RegisterConcrete(&MsgPinCodes{}, "wasm/MsgPinCodes", nil)
	cdc.RegisterConcrete(&MsgUnpinCodes{}, "wasm/MsgUnpinCodes", nil)
	cdc.RegisterConcrete(&MsgStoreAndInstantiateContract{}, "wasm/MsgStoreAndInstantiateContract", nil)
	cdc.RegisterConcrete(&MsgAddCodeUploadParamsAddresses{}, "wasm/MsgAddCodeUploadParamsAddresses", nil)
	cdc.RegisterConcrete(&MsgRemoveCodeUploadParamsAddresses{}, "wasm/MsgRemoveCodeUploadParamsAddresses", nil)
	cdc.RegisterConcrete(&MsgStoreAndMigrateContract{}, "wasm/MsgStoreAndMigrateContract", nil)
	cdc.RegisterConcrete(&MsgUpdateContractLabel{}, "wasm/MsgUpdateContractLabel", nil)

	cdc.RegisterInterface((*ContractInfoExtension)(nil), nil)

	cdc.RegisterInterface((*ContractAuthzFilterX)(nil), nil)
	cdc.RegisterConcrete(&AllowAllMessagesFilter{}, "wasm/AllowAllMessagesFilter", nil)
	cdc.RegisterConcrete(&AcceptedMessageKeysFilter{}, "wasm/AcceptedMessageKeysFilter", nil)
	cdc.RegisterConcrete(&AcceptedMessagesFilter{}, "wasm/AcceptedMessagesFilter", nil)

	cdc.RegisterInterface((*ContractAuthzLimitX)(nil), nil)
	cdc.RegisterConcrete(&MaxCallsLimit{}, "wasm/MaxCallsLimit", nil)
	cdc.RegisterConcrete(&MaxFundsLimit{}, "wasm/MaxFundsLimit", nil)
	cdc.RegisterConcrete(&CombinedLimit{}, "wasm/CombinedLimit", nil)

	cdc.RegisterConcrete(&StoreCodeAuthorization{}, "wasm/StoreCodeAuthorization", nil)
	cdc.RegisterConcrete(&ContractExecutionAuthorization{}, "wasm/ContractExecutionAuthorization", nil)
	cdc.RegisterConcrete(&ContractMigrationAuthorization{}, "wasm/ContractMigrationAuthorization", nil)

	// legacy gov v1beta1 types that may be used for unmarshalling stored gov data
	cdc.RegisterConcrete(&PinCodesProposal{}, "wasm/PinCodesProposal", nil)
	cdc.RegisterConcrete(&UnpinCodesProposal{}, "wasm/UnpinCodesProposal", nil)
	cdc.RegisterConcrete(&StoreCodeProposal{}, "wasm/StoreCodeProposal", nil)
	cdc.RegisterConcrete(&InstantiateContractProposal{}, "wasm/InstantiateContractProposal", nil)
	cdc.RegisterConcrete(&InstantiateContract2Proposal{}, "wasm/InstantiateContract2Proposal", nil)
	cdc.RegisterConcrete(&MigrateContractProposal{}, "wasm/MigrateContractProposal", nil)
	cdc.RegisterConcrete(&SudoContractProposal{}, "wasm/SudoContractProposal", nil)
	cdc.RegisterConcrete(&ExecuteContractProposal{}, "wasm/ExecuteContractProposal", nil)
	cdc.RegisterConcrete(&UpdateAdminProposal{}, "wasm/UpdateAdminProposal", nil)
	cdc.RegisterConcrete(&ClearAdminProposal{}, "wasm/ClearAdminProposal", nil)
	cdc.RegisterConcrete(&UpdateInstantiateConfigProposal{}, "wasm/UpdateInstantiateConfigProposal", nil)
	cdc.RegisterConcrete(&StoreAndInstantiateContractProposal{}, "wasm/StoreAndInstantiateContractProposal", nil)
}

// RegisterInterfaces registers the concrete proto types and interfaces with the SDK interface registry
func RegisterInterfaces(registry types.InterfaceRegistry) {
	registry.RegisterImplementations(
		(*sdk.Msg)(nil),
		&MsgStoreCode{},
		&MsgInstantiateContract{},
		&MsgInstantiateContract2{},
		&MsgExecuteContract{},
		&MsgMigrateContract{},
		&MsgUpdateAdmin{},
		&MsgClearAdmin{},
		&MsgIBCCloseChannel{},
		&MsgIBCSend{},
		&MsgUpdateInstantiateConfig{},
		&MsgUpdateParams{},
		&MsgSudoContract{},
		&MsgPinCodes{},
		&MsgUnpinCodes{},
		&MsgStoreAndInstantiateContract{},
		&MsgAddCodeUploadParamsAddresses{},
		&MsgRemoveCodeUploadParamsAddresses{},
		&MsgStoreAndMigrateContract{},
		&MsgUpdateContractLabel{},
	)
	registry.RegisterInterface("cosmwasm.wasm.v1.ContractInfoExtension", (*ContractInfoExtension)(nil))

	registry.RegisterInterface("cosmwasm.wasm.v1.ContractAuthzFilterX", (*ContractAuthzFilterX)(nil))
	registry.RegisterImplementations(
		(*ContractAuthzFilterX)(nil),
		&AllowAllMessagesFilter{},
		&AcceptedMessageKeysFilter{},
		&AcceptedMessagesFilter{},
	)

	registry.RegisterInterface("cosmwasm.wasm.v1.ContractAuthzLimitX", (*ContractAuthzLimitX)(nil))
	registry.RegisterImplementations(
		(*ContractAuthzLimitX)(nil),
		&MaxCallsLimit{},
		&MaxFundsLimit{},
		&CombinedLimit{},
	)

	registry.RegisterImplementations(
		(*authz.Authorization)(nil),
		&StoreCodeAuthorization{},
		&ContractExecutionAuthorization{},
		&ContractMigrationAuthorization{},
	)

	msgservice.RegisterMsgServiceDesc(registry, &_Msg_serviceDesc)

	// legacy gov v1beta1 types that may be used for unmarshalling stored gov data
	registry.RegisterImplementations(
		(*v1beta1.Content)(nil),
		&StoreCodeProposal{},
		&InstantiateContractProposal{},
		&InstantiateContract2Proposal{},
		&MigrateContractProposal{},
		&SudoContractProposal{},
		&ExecuteContractProposal{},
		&UpdateAdminProposal{},
		&ClearAdminProposal{},
		&PinCodesProposal{},
		&UnpinCodesProposal{},
		&UpdateInstantiateConfigProposal{},
		&StoreAndInstantiateContractProposal{},
	)
}
