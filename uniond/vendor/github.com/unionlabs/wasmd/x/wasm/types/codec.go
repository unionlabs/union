package types

import (
	"cosmossdk.io/core/registry"
	"github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/msgservice"
	"cosmossdk.io/x/authz"
	"cosmossdk.io/x/gov/types/v1beta1"
)

// RegisterLegacyAminoCodec registers the concrete types and interface
func RegisterLegacyAminoCodec(cdc registry.AminoRegistrar) {
	cdc.RegisterConcrete(&MsgStoreCode{}, "wasm/MsgStoreCode")
	cdc.RegisterConcrete(&MsgInstantiateContract{}, "wasm/MsgInstantiateContract")
	cdc.RegisterConcrete(&MsgInstantiateContract2{}, "wasm/MsgInstantiateContract2")
	cdc.RegisterConcrete(&MsgExecuteContract{}, "wasm/MsgExecuteContract")
	cdc.RegisterConcrete(&MsgMigrateContract{}, "wasm/MsgMigrateContract")
	cdc.RegisterConcrete(&MsgUpdateAdmin{}, "wasm/MsgUpdateAdmin")
	cdc.RegisterConcrete(&MsgClearAdmin{}, "wasm/MsgClearAdmin")
	cdc.RegisterConcrete(&MsgUpdateInstantiateConfig{}, "wasm/MsgUpdateInstantiateConfig")
	cdc.RegisterConcrete(&MsgUpdateParams{}, "wasm/MsgUpdateParams")
	cdc.RegisterConcrete(&MsgSudoContract{}, "wasm/MsgSudoContract")
	cdc.RegisterConcrete(&MsgPinCodes{}, "wasm/MsgPinCodes")
	cdc.RegisterConcrete(&MsgUnpinCodes{}, "wasm/MsgUnpinCodes")
	cdc.RegisterConcrete(&MsgStoreAndInstantiateContract{}, "wasm/MsgStoreAndInstantiateContract")
	cdc.RegisterConcrete(&MsgAddCodeUploadParamsAddresses{}, "wasm/MsgAddCodeUploadParamsAddresses")
	cdc.RegisterConcrete(&MsgRemoveCodeUploadParamsAddresses{}, "wasm/MsgRemoveCodeUploadParamsAddresses")
	cdc.RegisterConcrete(&MsgStoreAndMigrateContract{}, "wasm/MsgStoreAndMigrateContract")
	cdc.RegisterConcrete(&MsgUpdateContractLabel{}, "wasm/MsgUpdateContractLabel")

	cdc.RegisterInterface((*ContractInfoExtension)(nil), nil)

	cdc.RegisterInterface((*ContractAuthzFilterX)(nil), nil)
	cdc.RegisterConcrete(&AllowAllMessagesFilter{}, "wasm/AllowAllMessagesFilter")
	cdc.RegisterConcrete(&AcceptedMessageKeysFilter{}, "wasm/AcceptedMessageKeysFilter")
	cdc.RegisterConcrete(&AcceptedMessagesFilter{}, "wasm/AcceptedMessagesFilter")

	cdc.RegisterInterface((*ContractAuthzLimitX)(nil), nil)
	cdc.RegisterConcrete(&MaxCallsLimit{}, "wasm/MaxCallsLimit")
	cdc.RegisterConcrete(&MaxFundsLimit{}, "wasm/MaxFundsLimit")
	cdc.RegisterConcrete(&CombinedLimit{}, "wasm/CombinedLimit")

	cdc.RegisterConcrete(&StoreCodeAuthorization{}, "wasm/StoreCodeAuthorization")
	cdc.RegisterConcrete(&ContractExecutionAuthorization{}, "wasm/ContractExecutionAuthorization")
	cdc.RegisterConcrete(&ContractMigrationAuthorization{}, "wasm/ContractMigrationAuthorization")

	// legacy gov v1beta1 types that may be used for unmarshalling stored gov data
	cdc.RegisterConcrete(&PinCodesProposal{}, "wasm/PinCodesProposal")
	cdc.RegisterConcrete(&UnpinCodesProposal{}, "wasm/UnpinCodesProposal")
	cdc.RegisterConcrete(&StoreCodeProposal{}, "wasm/StoreCodeProposal")
	cdc.RegisterConcrete(&InstantiateContractProposal{}, "wasm/InstantiateContractProposal")
	cdc.RegisterConcrete(&InstantiateContract2Proposal{}, "wasm/InstantiateContract2Proposal")
	cdc.RegisterConcrete(&MigrateContractProposal{}, "wasm/MigrateContractProposal")
	cdc.RegisterConcrete(&SudoContractProposal{}, "wasm/SudoContractProposal")
	cdc.RegisterConcrete(&ExecuteContractProposal{}, "wasm/ExecuteContractProposal")
	cdc.RegisterConcrete(&UpdateAdminProposal{}, "wasm/UpdateAdminProposal")
	cdc.RegisterConcrete(&ClearAdminProposal{}, "wasm/ClearAdminProposal")
	cdc.RegisterConcrete(&UpdateInstantiateConfigProposal{}, "wasm/UpdateInstantiateConfigProposal")
	cdc.RegisterConcrete(&StoreAndInstantiateContractProposal{}, "wasm/StoreAndInstantiateContractProposal")
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
