package keeper_test

import (
	"encoding/hex"
	"encoding/json"
	"errors"

	wasmvm "github.com/CosmWasm/wasmvm/v2"
	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	govtypes "cosmossdk.io/x/gov/types"

	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/internal/ibcwasm"
	wasmtesting "github.com/cosmos/ibc-go/modules/light-clients/08-wasm/testing"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
	localhost "github.com/cosmos/ibc-go/v8/modules/light-clients/09-localhost"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

func (suite *KeeperTestSuite) TestMsgStoreCode() {
	var (
		msg    *types.MsgStoreCode
		signer string
		data   []byte
	)

	testCases := []struct {
		name     string
		malleate func()
		expError error
	}{
		{
			"success",
			func() {
				msg = types.NewMsgStoreCode(signer, data)
			},
			nil,
		},
		{
			"fails with duplicate wasm code",
			func() {
				msg = types.NewMsgStoreCode(signer, data)

				_, err := GetSimApp(suite.chainA).WasmClientKeeper.StoreCode(suite.chainA.GetContext(), msg)
				suite.Require().NoError(err)
			},
			types.ErrWasmCodeExists,
		},
		{
			"fails with zero-length wasm code",
			func() {
				msg = types.NewMsgStoreCode(signer, []byte{})
			},
			types.ErrWasmEmptyCode,
		},
		{
			"fails with checksum",
			func() {
				msg = types.NewMsgStoreCode(signer, []byte{0, 1, 3, 4})
			},
			errors.New("Wasm bytes do not start with Wasm magic number"),
		},
		{
			"fails with wasm code too large",
			func() {
				msg = types.NewMsgStoreCode(signer, append(wasmtesting.WasmMagicNumber, []byte(ibctesting.GenerateString(uint(types.MaxWasmByteSize())))...))
			},
			types.ErrWasmCodeTooLarge,
		},
		{
			"fails with unauthorized signer",
			func() {
				signer = suite.chainA.SenderAccount.GetAddress().String()
				msg = types.NewMsgStoreCode(signer, data)
			},
			ibcerrors.ErrUnauthorized,
		},
		{
			"failure: checksum could not be pinned",
			func() {
				msg = types.NewMsgStoreCode(signer, data)

				suite.mockVM.PinFn = func(_ wasmvm.Checksum) error {
					return wasmtesting.ErrMockVM
				}
			},
			wasmtesting.ErrMockVM,
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			suite.SetupWasmWithMockVM()

			signer = authtypes.NewModuleAddress(govtypes.ModuleName).String()
			data = wasmtesting.Code

			tc.malleate()

			ctx := suite.chainA.GetContext()
			res, err := GetSimApp(suite.chainA).WasmClientKeeper.StoreCode(ctx, msg)
			events := ctx.EventManager().Events()

			if tc.expError == nil {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)
				suite.Require().NotEmpty(res.Checksum)

				// Verify events
				expectedEvents := sdk.Events{
					sdk.NewEvent(
						"store_wasm_code",
						sdk.NewAttribute(types.AttributeKeyWasmChecksum, hex.EncodeToString(res.Checksum)),
					),
					sdk.NewEvent(
						sdk.EventTypeMessage,
						sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
					),
				}

				for _, evt := range expectedEvents {
					suite.Require().Contains(events, evt)
				}
			} else {
				suite.Require().Contains(err.Error(), tc.expError.Error())
				suite.Require().Nil(res)
				suite.Require().Empty(events)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestMsgMigrateContract() {
	oldChecksum, err := types.CreateChecksum(wasmtesting.Code)
	suite.Require().NoError(err)

	newByteCode := wasmtesting.CreateMockContract([]byte("MockByteCode-TestMsgMigrateContract"))
	govAcc := authtypes.NewModuleAddress(govtypes.ModuleName).String()

	var (
		newChecksum    []byte
		msg            *types.MsgMigrateContract
		expClientState *types.ClientState
	)

	testCases := []struct {
		name     string
		malleate func()
		expError error
	}{
		{
			"success: no update to client state",
			func() {
				msg = types.NewMsgMigrateContract(govAcc, defaultWasmClientID, newChecksum, []byte("{}"))

				suite.mockVM.MigrateFn = func(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ []byte, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
					data, err := json.Marshal(types.EmptyResult{})
					suite.Require().NoError(err)

					return &wasmvmtypes.ContractResult{Ok: &wasmvmtypes.Response{Data: data}}, wasmtesting.DefaultGasUsed, nil
				}
			},
			nil,
		},
		{
			"success: update client state",
			func() {
				msg = types.NewMsgMigrateContract(govAcc, defaultWasmClientID, newChecksum, []byte("{}"))

				suite.mockVM.MigrateFn = func(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ []byte, store wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
					// the checksum written in the client state will later be overwritten by the message server.
					expClientStateBz := wasmtesting.CreateMockClientStateBz(suite.chainA.App.AppCodec(), []byte("invalid checksum"))
					expClientState = clienttypes.MustUnmarshalClientState(suite.chainA.App.AppCodec(), expClientStateBz).(*types.ClientState)
					store.Set(host.ClientStateKey(), expClientStateBz)

					data, err := json.Marshal(types.EmptyResult{})
					suite.Require().NoError(err)

					return &wasmvmtypes.ContractResult{Ok: &wasmvmtypes.Response{Data: data}}, wasmtesting.DefaultGasUsed, nil
				}
			},
			nil,
		},
		{
			"failure: same checksum",
			func() {
				msg = types.NewMsgMigrateContract(govAcc, defaultWasmClientID, oldChecksum, []byte("{}"))

				suite.mockVM.MigrateFn = func(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ []byte, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
					panic("unreachable")
				}
			},
			types.ErrWasmCodeExists,
		},
		{
			"failure: unauthorized signer",
			func() {
				msg = types.NewMsgMigrateContract(suite.chainA.SenderAccount.GetAddress().String(), defaultWasmClientID, newChecksum, []byte("{}"))
			},
			ibcerrors.ErrUnauthorized,
		},
		{
			"failure: invalid wasm checksum",
			func() {
				msg = types.NewMsgMigrateContract(govAcc, defaultWasmClientID, []byte(ibctesting.InvalidID), []byte("{}"))
			},
			types.ErrWasmChecksumNotFound,
		},
		{
			"failure: invalid client id",
			func() {
				msg = types.NewMsgMigrateContract(govAcc, ibctesting.InvalidID, newChecksum, []byte("{}"))
			},
			clienttypes.ErrClientTypeNotFound,
		},
		{
			"failure: vm returns error",
			func() {
				msg = types.NewMsgMigrateContract(govAcc, defaultWasmClientID, newChecksum, []byte("{}"))

				suite.mockVM.MigrateFn = func(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ []byte, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
					return nil, wasmtesting.DefaultGasUsed, wasmtesting.ErrMockVM
				}
			},
			types.ErrVMError,
		},
		{
			"failure: contract returns error",
			func() {
				msg = types.NewMsgMigrateContract(govAcc, defaultWasmClientID, newChecksum, []byte("{}"))

				suite.mockVM.MigrateFn = func(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ []byte, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
					return &wasmvmtypes.ContractResult{Err: wasmtesting.ErrMockContract.Error()}, wasmtesting.DefaultGasUsed, nil
				}
			},
			types.ErrWasmContractCallFailed,
		},
		{
			"failure: incorrect state update",
			func() {
				msg = types.NewMsgMigrateContract(govAcc, defaultWasmClientID, newChecksum, []byte("{}"))

				suite.mockVM.MigrateFn = func(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ []byte, store wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
					// the checksum written in here will be overwritten
					newClientState := localhost.NewClientState(clienttypes.NewHeight(1, 1))

					store.Set(host.ClientStateKey(), clienttypes.MustMarshalClientState(suite.chainA.App.AppCodec(), newClientState))

					data, err := json.Marshal(types.EmptyResult{})
					suite.Require().NoError(err)

					return &wasmvmtypes.ContractResult{Ok: &wasmvmtypes.Response{Data: data}}, wasmtesting.DefaultGasUsed, nil
				}
			},
			types.ErrWasmInvalidContractModification,
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			suite.SetupWasmWithMockVM()

			storeWasmCode(suite, wasmtesting.Code)
			newChecksum = storeWasmCode(suite, newByteCode)

			endpoint := wasmtesting.NewWasmEndpoint(suite.chainA)
			err := endpoint.CreateClient()
			suite.Require().NoError(err)

			// this is the old client state
			expClientState = endpoint.GetClientState().(*types.ClientState)

			tc.malleate()

			ctx := suite.chainA.GetContext()
			res, err := GetSimApp(suite.chainA).WasmClientKeeper.MigrateContract(ctx, msg)
			events := ctx.EventManager().Events().ToABCIEvents()

			if tc.expError == nil {
				expClientState.Checksum = newChecksum

				suite.Require().NoError(err)
				suite.Require().NotNil(res)

				// updated client state
				clientState, ok := endpoint.GetClientState().(*types.ClientState)
				suite.Require().True(ok)

				suite.Require().Equal(expClientState, clientState)

				// Verify events
				expectedEvents := sdk.Events{
					sdk.NewEvent(
						"migrate_contract",
						sdk.NewAttribute(types.AttributeKeyClientID, defaultWasmClientID),
						sdk.NewAttribute(types.AttributeKeyWasmChecksum, hex.EncodeToString(oldChecksum)),
						sdk.NewAttribute(types.AttributeKeyNewChecksum, hex.EncodeToString(newChecksum)),
					),
					sdk.NewEvent(
						sdk.EventTypeMessage,
						sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
					),
				}.ToABCIEvents()

				for _, evt := range expectedEvents {
					suite.Require().Contains(events, evt)
				}
			} else {
				suite.Require().ErrorIs(err, tc.expError)
				suite.Require().Nil(res)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestMsgRemoveChecksum() {
	checksum, err := types.CreateChecksum(wasmtesting.Code)
	suite.Require().NoError(err)

	govAcc := authtypes.NewModuleAddress(govtypes.ModuleName).String()

	var (
		msg          *types.MsgRemoveChecksum
		expChecksums []types.Checksum
	)

	testCases := []struct {
		name     string
		malleate func()
		expError error
	}{
		{
			"success",
			func() {
				msg = types.NewMsgRemoveChecksum(govAcc, checksum)

				expChecksums = []types.Checksum{}
			},
			nil,
		},
		{
			"success: many checksums",
			func() {
				msg = types.NewMsgRemoveChecksum(govAcc, checksum)

				expChecksums = []types.Checksum{}

				for i := 0; i < 20; i++ {
					mockCode := wasmtesting.CreateMockContract([]byte{byte(i)})
					checksum, err := types.CreateChecksum(mockCode)
					suite.Require().NoError(err)

					err = ibcwasm.Checksums.Set(suite.chainA.GetContext(), checksum)
					suite.Require().NoError(err)

					expChecksums = append(expChecksums, checksum)
				}
			},
			nil,
		},
		{
			"failure: checksum is missing",
			func() {
				msg = types.NewMsgRemoveChecksum(govAcc, []byte{1})
			},
			types.ErrWasmChecksumNotFound,
		},
		{
			"failure: unauthorized signer",
			func() {
				msg = types.NewMsgRemoveChecksum(suite.chainA.SenderAccount.GetAddress().String(), checksum)
			},
			ibcerrors.ErrUnauthorized,
		},
		{
			"failure: code has could not be unpinned",
			func() {
				msg = types.NewMsgRemoveChecksum(govAcc, checksum)

				suite.mockVM.UnpinFn = func(_ wasmvm.Checksum) error {
					return wasmtesting.ErrMockVM
				}
			},
			wasmtesting.ErrMockVM,
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			suite.SetupWasmWithMockVM()

			storeWasmCode(suite, wasmtesting.Code)

			endpoint := wasmtesting.NewWasmEndpoint(suite.chainA)
			err := endpoint.CreateClient()
			suite.Require().NoError(err)

			tc.malleate()

			ctx := suite.chainA.GetContext()
			res, err := GetSimApp(suite.chainA).WasmClientKeeper.RemoveChecksum(ctx, msg)
			events := ctx.EventManager().Events().ToABCIEvents()

			if tc.expError == nil {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)

				checksums, err := types.GetAllChecksums(suite.chainA.GetContext())
				suite.Require().NoError(err)

				// Check equality of checksums up to order
				suite.Require().ElementsMatch(expChecksums, checksums)

				// Verify events
				suite.Require().Len(events, 0)
			} else {
				suite.Require().ErrorIs(err, tc.expError)
				suite.Require().Nil(res)
			}
		})
	}
}
