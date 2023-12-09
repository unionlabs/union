package kvstore

import (
	"context"
	"encoding/base64"
	"fmt"
	"strings"

	"github.com/cometbft/cometbft/abci/types"
	cryptoencoding "github.com/cometbft/cometbft/crypto/encoding"
	cmtrand "github.com/cometbft/cometbft/libs/rand"
	"github.com/cometbft/cometbft/proto/tendermint/crypto"
)

// RandVal creates one random validator, with a key derived
// from the input value
func RandVal() types.ValidatorUpdate {
	pubkey := cmtrand.Bytes(32)
	power := cmtrand.Uint16() + 1
	v := types.UpdateValidator(pubkey, int64(power), "")
	return v
}

// RandVals returns a list of cnt validators for initializing
// the application. Note that the keys are deterministically
// derived from the index in the array, while the power is
// random (Change this if not desired)
func RandVals(cnt int) []types.ValidatorUpdate {
	res := make([]types.ValidatorUpdate, cnt)
	for i := 0; i < cnt; i++ {
		res[i] = RandVal()
	}
	return res
}

// InitKVStore initializes the kvstore app with some data,
// which allows tests to pass and is fine as long as you
// don't make any tx that modify the validator state
func InitKVStore(ctx context.Context, app *Application) error {
	_, err := app.InitChain(ctx, &types.RequestInitChain{
		Validators: RandVals(1),
	})
	return err
}

// Create a new transaction
func NewTx(key, value string) []byte {
	return []byte(strings.Join([]string{key, value}, "="))
}

func NewRandomTx(size int) []byte {
	if size < 4 {
		panic("random tx size must be greater than 3")
	}
	return NewTx(cmtrand.Str(2), cmtrand.Str(size-3))
}

func NewRandomTxs(n int) [][]byte {
	txs := make([][]byte, n)
	for i := 0; i < n; i++ {
		txs[i] = NewRandomTx(10)
	}
	return txs
}

func NewTxFromID(i int) []byte {
	return []byte(fmt.Sprintf("%d=%d", i, i))
}

// Create a transaction to add/remove/update a validator
// To remove, set power to 0.
func MakeValSetChangeTx(pubkey crypto.PublicKey, power int64) []byte {
	pk, err := cryptoencoding.PubKeyFromProto(pubkey)
	if err != nil {
		panic(err)
	}
	pubStr := base64.StdEncoding.EncodeToString(pk.Bytes())
	return []byte(fmt.Sprintf("%s%s!%d", ValidatorPrefix, pubStr, power))
}
