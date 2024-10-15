package blst

import (
	"fmt"
	"runtime"

	blst "github.com/supranational/blst/bindings/go"

	"github.com/cosmos/crypto/internal/cache"
)

func init() {
	// Reserve 1 core for general application work
	maxProcs := runtime.GOMAXPROCS(0) - 1
	if maxProcs <= 0 {
		maxProcs = 1
	}
	blst.SetMaxProcs(maxProcs)
	onEvict := func(_ [48]byte, _ PubKey) {}
	keysCache, err := cache.NewLRU(maxKeys, onEvict)
	if err != nil {
		panic(fmt.Sprintf("Could not initiate public keys cache: %v", err))
	}
	pubkeyCache = keysCache
}
