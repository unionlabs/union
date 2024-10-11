package stateutil

import (
	"sync"

	coreutils "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition/stateutils"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// ValidatorMapHandler is a container to hold the map and a reference tracker for how many
// states shared this.
type ValidatorMapHandler struct {
	valIdxMap map[[fieldparams.BLSPubkeyLength]byte]primitives.ValidatorIndex
	mapRef    *Reference
	*sync.RWMutex
}

// NewValMapHandler returns a new validator map handler.
func NewValMapHandler(vals []*ethpb.Validator) *ValidatorMapHandler {
	return &ValidatorMapHandler{
		valIdxMap: coreutils.ValidatorIndexMap(vals),
		mapRef:    &Reference{refs: 1},
		RWMutex:   new(sync.RWMutex),
	}
}

// AddRef copies the whole map and returns a map handler with the copied map.
func (v *ValidatorMapHandler) AddRef() {
	v.mapRef.AddRef()
}

// IsNil returns true if the underlying validator index map is nil.
func (v *ValidatorMapHandler) IsNil() bool {
	return v.mapRef == nil || v.valIdxMap == nil
}

// Copy the whole map and returns a map handler with the copied map.
func (v *ValidatorMapHandler) Copy() *ValidatorMapHandler {
	if v == nil || v.valIdxMap == nil {
		return &ValidatorMapHandler{valIdxMap: map[[fieldparams.BLSPubkeyLength]byte]primitives.ValidatorIndex{}, mapRef: new(Reference), RWMutex: new(sync.RWMutex)}
	}
	v.RLock()
	defer v.RUnlock()
	m := make(map[[fieldparams.BLSPubkeyLength]byte]primitives.ValidatorIndex, len(v.valIdxMap))
	for k, v := range v.valIdxMap {
		m[k] = v
	}
	return &ValidatorMapHandler{
		valIdxMap: m,
		mapRef:    &Reference{refs: 1},
		RWMutex:   new(sync.RWMutex),
	}
}

// Get the validator index using the corresponding public key.
func (v *ValidatorMapHandler) Get(key [fieldparams.BLSPubkeyLength]byte) (primitives.ValidatorIndex, bool) {
	v.RLock()
	defer v.RUnlock()
	idx, ok := v.valIdxMap[key]
	if !ok {
		return 0, false
	}
	return idx, true
}

// Set the validator index using the corresponding public key.
func (v *ValidatorMapHandler) Set(key [fieldparams.BLSPubkeyLength]byte, index primitives.ValidatorIndex) {
	v.Lock()
	defer v.Unlock()
	v.valIdxMap[key] = index
}
