package keeper_test

import (
	"encoding/hex"
	"time"

	govtypes "cosmossdk.io/x/gov/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"

	tmproto "github.com/cometbft/cometbft/api/cometbft/types/v1"

	wasmtesting "github.com/cosmos/ibc-go/modules/light-clients/08-wasm/testing"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/testing/simapp"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
)

func (suite *KeeperTestSuite) TestSnapshotter() {
	gzippedContract, err := types.GzipIt(wasmtesting.CreateMockContract([]byte("gzipped-contract")))
	suite.Require().NoError(err)

	testCases := []struct {
		name      string
		contracts [][]byte
	}{
		{
			name:      "single contract",
			contracts: [][]byte{wasmtesting.Code},
		},
		{
			name:      "multiple contracts",
			contracts: [][]byte{wasmtesting.Code, gzippedContract},
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			t := suite.T()
			wasmClientApp := suite.SetupSnapshotterWithMockVM()

			ctx := wasmClientApp.NewUncachedContext(false, tmproto.Header{
				ChainID: "foo",
				Height:  wasmClientApp.LastBlockHeight() + 1,
				Time:    time.Now(),
			})

			var srcChecksumCodes []byte
			var checksums [][]byte
			// store contract on chain
			for _, contract := range tc.contracts {
				signer := authtypes.NewModuleAddress(govtypes.ModuleName).String()
				msg := types.NewMsgStoreCode(signer, contract)

				res, err := wasmClientApp.WasmClientKeeper.StoreCode(ctx, msg)
				suite.Require().NoError(err)

				checksums = append(checksums, res.Checksum)
				srcChecksumCodes = append(srcChecksumCodes, res.Checksum...)

				suite.Require().NoError(err)
			}

			// create snapshot
			res, err := wasmClientApp.Commit()
			suite.Require().NoError(err)
			suite.Require().NotNil(res)

			snapshotHeight := uint64(wasmClientApp.LastBlockHeight())
			snapshot, err := wasmClientApp.SnapshotManager().Create(snapshotHeight)
			suite.Require().NoError(err)
			suite.Require().NotNil(snapshot)

			// setup dest app with snapshot imported
			destWasmClientApp := simapp.SetupWithEmptyStore(t, suite.mockVM)
			destCtx := destWasmClientApp.NewUncachedContext(false, tmproto.Header{
				ChainID: "bar",
				Height:  destWasmClientApp.LastBlockHeight() + 1,
				Time:    time.Now(),
			})

			resp, err := destWasmClientApp.WasmClientKeeper.Checksums(destCtx, &types.QueryChecksumsRequest{})
			suite.Require().NoError(err)
			suite.Require().Empty(resp.Checksums)

			suite.Require().NoError(destWasmClientApp.SnapshotManager().Restore(*snapshot))

			for i := uint32(0); i < snapshot.Chunks; i++ {
				chunkBz, err := wasmClientApp.SnapshotManager().LoadChunk(snapshot.Height, snapshot.Format, i)
				suite.Require().NoError(err)

				end, err := destWasmClientApp.SnapshotManager().RestoreChunk(chunkBz)
				suite.Require().NoError(err)

				if end {
					break
				}
			}

			var allDestAppChecksumsInWasmVMStore []byte
			// check wasm contracts are imported
			ctx = destWasmClientApp.NewUncachedContext(false, tmproto.Header{
				ChainID: "foo",
				Height:  destWasmClientApp.LastBlockHeight() + 1,
				Time:    time.Now(),
			})

			for _, checksum := range checksums {
				resp, err := destWasmClientApp.WasmClientKeeper.Code(ctx, &types.QueryCodeRequest{Checksum: hex.EncodeToString(checksum)})
				suite.Require().NoError(err)

				checksum, err := types.CreateChecksum(resp.Data)
				suite.Require().NoError(err)

				allDestAppChecksumsInWasmVMStore = append(allDestAppChecksumsInWasmVMStore, checksum...)
			}

			suite.Require().Equal(srcChecksumCodes, allDestAppChecksumsInWasmVMStore)
		})
	}
}
