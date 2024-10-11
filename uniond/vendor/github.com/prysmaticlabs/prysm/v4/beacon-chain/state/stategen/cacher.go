package stategen

import (
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
)

var ErrNotInCache = errors.New("state not found in cache")

type CachedGetter interface {
	ByBlockRoot([32]byte) (state.BeaconState, error)
}

type CombinedCache struct {
	getters []CachedGetter
}

func (c CombinedCache) ByBlockRoot(r [32]byte) (state.BeaconState, error) {
	for _, getter := range c.getters {
		st, err := getter.ByBlockRoot(r)
		if err == nil {
			return st, nil
		}
		if errors.Is(err, ErrNotInCache) {
			continue
		}
		return nil, err
	}
	return nil, ErrNotInCache
}

var _ CachedGetter = &CombinedCache{}
