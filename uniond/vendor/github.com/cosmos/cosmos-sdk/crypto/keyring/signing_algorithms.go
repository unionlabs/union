package keyring

import (
	"fmt"
	"strings"

	"github.com/cosmos/cosmos-sdk/crypto/hd"
)

// SignatureAlgo defines the interface for a keyring supported algorithm.
type SignatureAlgo interface {
	Name() hd.PubKeyType
	Derive() hd.DeriveFn
	Generate() hd.GenerateFn
}

// NewSigningAlgoFromString creates a supported SignatureAlgo.
func NewSigningAlgoFromString(str string, algoList SigningAlgoList) (SignatureAlgo, error) {
	for _, algo := range algoList {
		if str == string(algo.Name()) {
			return algo, nil
		}
	}
	return nil, fmt.Errorf("provided algorithm %q is not supported", str)
}

// SigningAlgoList is a slice of signature algorithms
type SigningAlgoList []SignatureAlgo

// Contains returns true if the SigningAlgoList the given SignatureAlgo.
func (sal SigningAlgoList) Contains(algo SignatureAlgo) bool {
	for _, cAlgo := range sal {
		if cAlgo.Name() == algo.Name() {
			return true
		}
	}

	return false
}

// String returns a comma separated string of the signature algorithm names in the list.
func (sal SigningAlgoList) String() string {
	names := make([]string, len(sal))
	for i := range sal {
		names[i] = string(sal[i].Name())
	}

	return strings.Join(names, ",")
}
