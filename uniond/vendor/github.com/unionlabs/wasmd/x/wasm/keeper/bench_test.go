package keeper

import (
	"os"
	"testing"

	dbm "github.com/cosmos/cosmos-db"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

// BenchmarkVerification benchmarks secp256k1 verification which is 1000 gas based on cpu time.
//
// Just this function is copied from
// https://github.com/cosmos/cosmos-sdk/blob/90e9370bd80d9a3d41f7203ddb71166865561569/crypto/keys/internal/benchmarking/bench.go#L48-L62
// And thus under the GO license (BSD style)
func BenchmarkGasNormalization(b *testing.B) {
	priv := secp256k1.GenPrivKey()
	pub := priv.PubKey()

	// use a short message, so this time doesn't get dominated by hashing.
	message := []byte("Hello, world!")
	signature, err := priv.Sign(message)
	if err != nil {
		b.Fatal(err)
	}
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		pub.VerifySignature(message, signature)
	}
}

// By comparing the timing for queries on pinned vs unpinned, the difference gives us the overhead of
// instantiating an unpinned contract. That value can be used to determine a reasonable gas price
// for the InstantiationCost
func BenchmarkInstantiationOverhead(b *testing.B) {
	specs := map[string]struct {
		pinned bool
		db     func() dbm.DB
	}{
		"unpinned, memory db": {
			db: func() dbm.DB { return dbm.NewMemDB() },
		},
		"pinned, memory db": {
			db:     func() dbm.DB { return dbm.NewMemDB() },
			pinned: true,
		},
	}
	for name, spec := range specs {
		b.Run(name, func(b *testing.B) {
			wasmConfig := types.WasmConfig{MemoryCacheSize: 0}
			ctx, keepers := createTestInput(b, false, AvailableCapabilities, wasmConfig, spec.db())
			example := InstantiateHackatomExampleContract(b, ctx, keepers)
			if spec.pinned {
				require.NoError(b, keepers.ContractKeeper.PinCode(ctx, example.CodeID))
			}
			b.ResetTimer()
			for i := 0; i < b.N; i++ {
				_, err := keepers.WasmKeeper.QuerySmart(ctx, example.Contract, []byte(`{"verifier":{}}`))
				require.NoError(b, err)
			}
		})
	}
}

// Calculate the time it takes to compile some wasm code the first time.
// This will help us adjust pricing for UploadCode
func BenchmarkCompilation(b *testing.B) {
	specs := map[string]struct {
		wasmFile string
	}{
		"hackatom": {
			wasmFile: "./testdata/hackatom.wasm",
		},
		"burner": {
			wasmFile: "./testdata/burner.wasm",
		},
		"ibc_reflect": {
			wasmFile: "./testdata/ibc_reflect.wasm",
		},
	}

	for name, spec := range specs {
		b.Run(name, func(b *testing.B) {
			wasmConfig := types.WasmConfig{MemoryCacheSize: 0}
			db := dbm.NewMemDB()
			ctx, keepers := createTestInput(b, false, AvailableCapabilities, wasmConfig, db)

			// print out code size for comparisons
			code, err := os.ReadFile(spec.wasmFile)
			require.NoError(b, err)
			b.Logf("\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b(size: %d)  ", len(code))

			b.ResetTimer()
			for i := 0; i < b.N; i++ {
				_ = StoreExampleContract(b, ctx, keepers, spec.wasmFile)
			}
		})
	}
}
