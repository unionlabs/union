package address

import (
	"crypto/rand"
	"sync"
	"sync/atomic"
	"testing"

	"github.com/hashicorp/golang-lru/simplelru"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/internal/conv"
)

var (
	lru, _       = simplelru.NewLRU(500, nil)
	cacheOptions = CachedCodecOptions{
		Mu:  &sync.Mutex{},
		Lru: lru,
	}
)

func generateAddresses(totalAddresses int) ([][]byte, error) {
	keys := make([][]byte, totalAddresses)
	addr := make([]byte, 32)
	for i := 0; i < totalAddresses; i++ {
		_, err := rand.Read(addr)
		if err != nil {
			return nil, err
		}
		keys[i] = addr
	}

	return keys, nil
}

func TestNewBech32Codec(t *testing.T) {
	tests := []struct {
		name  string
		mu    *sync.Mutex
		lru   func(t *testing.T) *simplelru.LRU
		error bool
	}{
		{
			name: "lru and mutex provided",
			mu:   &sync.Mutex{},
			lru: func(t *testing.T) *simplelru.LRU {
				t.Helper()
				newLru, err := simplelru.NewLRU(500, nil)
				require.NoError(t, err)
				return newLru
			},
		},
		{
			name: "both empty",
			mu:   nil,
			lru: func(t *testing.T) *simplelru.LRU {
				t.Helper()
				return nil
			},
		},
		{
			name: "only lru provided",
			mu:   nil,
			lru: func(t *testing.T) *simplelru.LRU {
				t.Helper()
				newLru, err := simplelru.NewLRU(500, nil)
				require.NoError(t, err)
				return newLru
			},
			error: true,
		},
		{
			name: "only mutex provided",
			mu:   &sync.Mutex{},
			lru: func(t *testing.T) *simplelru.LRU {
				t.Helper()
				return nil
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			newLru := tt.lru(t)
			got, err := NewCachedBech32Codec("cosmos", CachedCodecOptions{
				Mu:  tt.mu,
				Lru: newLru,
			})
			if tt.error {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
				require.NotNil(t, got)
			}
		})
	}
}

func TestBech32Codec(t *testing.T) {
	tests := []struct {
		name    string
		prefix  string
		lru     *simplelru.LRU
		address string
	}{
		{
			name:    "create accounts cached bech32 codec",
			prefix:  "cosmos",
			lru:     lru,
			address: "cosmos1p8s0p6gqc6c9gt77lgr2qqujz49huhu6a80smx",
		},
		{
			name:    "create validator cached bech32 codec",
			prefix:  "cosmosvaloper",
			lru:     lru,
			address: "cosmosvaloper1sjllsnramtg3ewxqwwrwjxfgc4n4ef9u2lcnj0",
		},
		{
			name:    "create consensus cached bech32 codec",
			prefix:  "cosmosvalcons",
			lru:     lru,
			address: "cosmosvalcons1ntk8eualewuprz0gamh8hnvcem2nrcdsgz563h",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ac, err := NewCachedBech32Codec(tt.prefix, cacheOptions)
			require.NoError(t, err)

			cached, ok := ac.(cachedBech32Codec)
			require.True(t, ok)
			require.Equal(t, cached.cache, tt.lru)

			addr, err := ac.StringToBytes(tt.address)
			require.NoError(t, err)

			cachedAddr, ok := tt.lru.Get(tt.address)
			require.True(t, ok)
			require.Equal(t, addr, cachedAddr)

			accAddr, err := ac.BytesToString(addr)
			require.NoError(t, err)

			cachedStrAddr, ok := tt.lru.Get(conv.UnsafeBytesToStr(addr))
			require.True(t, ok)
			cachedStrAddrMap, ok := cachedStrAddr.(map[string]string)
			require.True(t, ok)
			require.Equal(t, accAddr, cachedStrAddrMap[tt.prefix])
		})
	}
}

func TestMultipleBech32Codec(t *testing.T) {
	cAc, err := NewCachedBech32Codec("cosmos", cacheOptions)
	require.NoError(t, err)
	cosmosAc, ok := cAc.(cachedBech32Codec)
	require.True(t, ok)
	sAc, err := NewCachedBech32Codec("stake", cacheOptions)
	require.NoError(t, err)
	stakeAc, ok := sAc.(cachedBech32Codec)
	require.True(t, ok)
	require.Equal(t, cosmosAc.cache, stakeAc.cache)

	addr := make([]byte, 32)
	_, err = rand.Read(addr)
	require.NoError(t, err)

	cosmosAddr, err := cosmosAc.BytesToString(addr)
	require.NoError(t, err)
	stakeAddr, err := stakeAc.BytesToString(addr)
	require.NoError(t, err)
	require.True(t, cosmosAddr != stakeAddr)

	cachedCosmosAddr, err := cosmosAc.BytesToString(addr)
	require.NoError(t, err)
	require.Equal(t, cosmosAddr, cachedCosmosAddr)

	cachedStakeAddr, err := stakeAc.BytesToString(addr)
	require.NoError(t, err)
	require.Equal(t, stakeAddr, cachedStakeAddr)
}

func TestBech32CodecRace(t *testing.T) {
	ac, err := NewCachedBech32Codec("cosmos", cacheOptions)
	require.NoError(t, err)
	myAddrBz := []byte{0x1, 0x2, 0x3, 0x4, 0x5}

	var (
		wgStart, wgDone sync.WaitGroup
		errCount        atomic.Uint32
	)
	const n = 3
	wgStart.Add(n)
	wgDone.Add(n)
	for i := 0; i < n; i++ {
		go func() {
			wgStart.Done()
			wgStart.Wait() // wait for all routines started

			got, err := ac.BytesToString(myAddrBz)
			if err != nil || got != "cosmos1qypqxpq9dc9msf" {
				errCount.Add(1)
			}
			wgDone.Done()
		}()
	}
	wgDone.Wait() // wait for all routines completed
	require.Equal(t, errCount.Load(), uint32(0))
}
