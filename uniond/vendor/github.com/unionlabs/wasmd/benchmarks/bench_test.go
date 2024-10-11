package benchmarks

import (
	"encoding/json"
	"testing"
	"time"

	abci "github.com/cometbft/cometbft/abci/types"
	dbm "github.com/cosmos/cosmos-db"
	"github.com/stretchr/testify/require"
	"github.com/syndtr/goleveldb/leveldb/opt"

	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	sdk "github.com/cosmos/cosmos-sdk/types"
	banktypes "cosmossdk.io/x/bank/types"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
)

func BenchmarkTxSending(b *testing.B) {
	cases := map[string]struct {
		db          func(*testing.B) dbm.DB
		txBuilder   func(*testing.B, *AppInfo) []sdk.Tx
		blockSize   int
		numAccounts int
	}{
		"basic send - memdb": {
			db:          buildMemDB,
			blockSize:   20,
			txBuilder:   buildTxFromMsg(bankSendMsg),
			numAccounts: 50,
		},
		"cw20 transfer - memdb": {
			db:          buildMemDB,
			blockSize:   20,
			txBuilder:   buildTxFromMsg(cw20TransferMsg),
			numAccounts: 50,
		},
		"basic send - leveldb": {
			db:          buildLevelDB,
			blockSize:   20,
			txBuilder:   buildTxFromMsg(bankSendMsg),
			numAccounts: 50,
		},
		"cw20 transfer - leveldb": {
			db:          buildLevelDB,
			blockSize:   20,
			txBuilder:   buildTxFromMsg(cw20TransferMsg),
			numAccounts: 50,
		},
		"basic send - leveldb - 8k accounts": {
			db:          buildLevelDB,
			blockSize:   20,
			txBuilder:   buildTxFromMsg(bankSendMsg),
			numAccounts: 8000,
		},
		"cw20 transfer - leveldb - 8k accounts": {
			db:          buildLevelDB,
			blockSize:   20,
			txBuilder:   buildTxFromMsg(cw20TransferMsg),
			numAccounts: 8000,
		},
		"basic send - leveldb - 8k accounts - huge blocks": {
			db:          buildLevelDB,
			blockSize:   1000,
			txBuilder:   buildTxFromMsg(bankSendMsg),
			numAccounts: 8000,
		},
		"cw20 transfer - leveldb - 8k accounts - huge blocks": {
			db:          buildLevelDB,
			blockSize:   1000,
			txBuilder:   buildTxFromMsg(cw20TransferMsg),
			numAccounts: 8000,
		},
		"basic send - leveldb - 80k accounts": {
			db:          buildLevelDB,
			blockSize:   20,
			txBuilder:   buildTxFromMsg(bankSendMsg),
			numAccounts: 80000,
		},
		"cw20 transfer - leveldb - 80k accounts": {
			db:          buildLevelDB,
			blockSize:   20,
			txBuilder:   buildTxFromMsg(cw20TransferMsg),
			numAccounts: 80000,
		},
	}

	for name, tc := range cases {
		b.Run(name, func(b *testing.B) {
			db := tc.db(b)
			defer db.Close()
			appInfo := InitializeWasmApp(b, db, tc.numAccounts)
			txs := tc.txBuilder(b, &appInfo)

			// number of Tx per block for the benchmarks
			blockSize := tc.blockSize
			height := int64(2)
			txEncoder := appInfo.TxConfig.TxEncoder()

			b.ResetTimer()

			for i := 0; i < b.N/blockSize; i++ {
				xxx := make([][]byte, blockSize)
				for j := 0; j < blockSize; j++ {
					idx := i*blockSize + j
					bz, err := txEncoder(txs[idx])
					require.NoError(b, err)
					xxx[j] = bz
				}
				_, err := appInfo.App.FinalizeBlock(&abci.RequestFinalizeBlock{Txs: xxx, Height: height, Time: time.Now()})
				require.NoError(b, err)

				_, err = appInfo.App.Commit()
				require.NoError(b, err)
				height++
			}
		})
	}
}

func bankSendMsg(info *AppInfo) ([]sdk.Msg, error) {
	// Precompute all txs
	rcpt := sdk.AccAddress(secp256k1.GenPrivKey().PubKey().Address())
	coins := sdk.Coins{sdk.NewInt64Coin(info.Denom, 100)}
	sendMsg := banktypes.NewMsgSend(info.MinterAddr, rcpt, coins)
	return []sdk.Msg{sendMsg}, nil
}

func cw20TransferMsg(info *AppInfo) ([]sdk.Msg, error) {
	rcpt := sdk.AccAddress(secp256k1.GenPrivKey().PubKey().Address())
	transfer := cw20ExecMsg{Transfer: &transferMsg{
		Recipient: rcpt.String(),
		Amount:    765,
	}}
	transferBz, err := json.Marshal(transfer)
	if err != nil {
		return nil, err
	}

	sendMsg := &wasmtypes.MsgExecuteContract{
		Sender:   info.MinterAddr.String(),
		Contract: info.ContractAddr,
		Msg:      transferBz,
	}
	return []sdk.Msg{sendMsg}, nil
}

func buildTxFromMsg(builder func(info *AppInfo) ([]sdk.Msg, error)) func(b *testing.B, info *AppInfo) []sdk.Tx {
	return func(b *testing.B, info *AppInfo) []sdk.Tx {
		return GenSequenceOfTxs(b, info, builder, b.N)
	}
}

func buildMemDB(_ *testing.B) dbm.DB {
	return dbm.NewMemDB()
}

func buildLevelDB(b *testing.B) dbm.DB {
	levelDB, err := dbm.NewGoLevelDBWithOpts("testing", b.TempDir(), &opt.Options{BlockCacher: opt.NoCacher})
	require.NoError(b, err)
	return levelDB
}
