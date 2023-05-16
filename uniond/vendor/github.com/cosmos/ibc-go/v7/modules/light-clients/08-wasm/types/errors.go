package types

import sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"

var (
	ErrInvalid                = sdkerrors.Register(ModuleName, 1, "invalid")
	ErrInvalidData            = sdkerrors.Register(ModuleName, 2, "invalid data")
	ErrInvalidCodeID          = sdkerrors.Register(ModuleName, 3, "invalid code ID")
	ErrInvalidHeader          = sdkerrors.Register(ModuleName, 4, "invalid header")
	ErrCreateContractFailed   = sdkerrors.Register(ModuleName, 5, "create wasm contract failed")
	ErrInitContractFailed     = sdkerrors.Register(ModuleName, 6, "initialize wasm contract failes")
	ErrCallContractFailed     = sdkerrors.Register(ModuleName, 7, "call to wasm contract failed")
	ErrQueryContractFailed    = sdkerrors.Register(ModuleName, 8, "queryto  wasm contract failed")
	ErrUnmarshalPayloadFailed = sdkerrors.Register(ModuleName, 9, "unmarshal wasm contract payload failed")
	ErrMarshalPayloadFailed   = sdkerrors.Register(ModuleName, 10, "marshal wasm contract payload failed")
	// Wasm specific
	ErrWasmEmptyCode      = sdkerrors.Register(ModuleName, 11, "empty wasm code")
	ErrWasmEmptyCodeHash  = sdkerrors.Register(ModuleName, 12, "empty wasm code hash")
	ErrWasmCodeTooLarge   = sdkerrors.Register(ModuleName, 13, "wasm code too large")
	ErrWasmCodeExists     = sdkerrors.Register(ModuleName, 14, "wasm code already exists")
	ErrWasmCodeValidation = sdkerrors.Register(ModuleName, 15, "unable to validate wasm code")
	ErrWasmInvalidCode    = sdkerrors.Register(ModuleName, 16, "invalid wasm code")
	ErrWasmInvalidCodeID  = sdkerrors.Register(ModuleName, 17, "invalid wasm code id")
	ErrWasmCodeIDNotFound = sdkerrors.Register(ModuleName, 18, "wasm code id not found")
)
