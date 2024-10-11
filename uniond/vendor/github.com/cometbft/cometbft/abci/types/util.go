package types

import (
	"bytes"
	"sort"
	"strings"
)

// ------------------------------------------------------------------------------

// ValidatorUpdates is a list of validators that implements the Sort interface.
type ValidatorUpdates []ValidatorUpdate

var _ sort.Interface = (ValidatorUpdates)(nil)

// All these methods for ValidatorUpdates:
//    Len, Less and Swap
// are for ValidatorUpdates to implement sort.Interface
// which will be used by the sort package.
// See Issue https://github.com/tendermint/abci/issues/212

func (v ValidatorUpdates) Len() int {
	return len(v)
}

// XXX: doesn't distinguish same validator with different power.
func (v ValidatorUpdates) Less(i, j int) bool {
	return strings.Compare(v[i].PubKeyType, v[j].PubKeyType) <= 0 && bytes.Compare(v[i].PubKeyBytes, v[j].PubKeyBytes) <= 0
}

func (v ValidatorUpdates) Swap(i, j int) {
	v[i], v[j] = v[j], v[i]
}
