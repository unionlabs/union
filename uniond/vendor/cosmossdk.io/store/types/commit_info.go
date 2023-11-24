package types

import (
	cmtprotocrypto "github.com/cometbft/cometbft/proto/tendermint/crypto"

	"cosmossdk.io/store/internal/maps"
)

// GetHash returns the GetHash from the CommitID.
// This is used in CommitInfo.Hash()
//
// When we commit to this in a merkle proof, we create a map of storeInfo.Name -> storeInfo.GetHash()
// and build a merkle proof from that.
// This is then chained with the substore proof, so we prove the root hash from the substore before this
// and need to pass that (unmodified) as the leaf value of the multistore proof.
func (si StoreInfo) GetHash() []byte {
	return si.CommitId.Hash
}

func (ci CommitInfo) toMap() map[string][]byte {
	m := make(map[string][]byte, len(ci.StoreInfos))
	for _, storeInfo := range ci.StoreInfos {
		m[storeInfo.Name] = storeInfo.GetHash()
	}

	return m
}

// Hash returns the simple merkle root hash of the stores sorted by name.
func (ci CommitInfo) Hash() []byte {
	// we need a special case for empty set, as SimpleProofsFromMap requires at least one entry
	if len(ci.StoreInfos) == 0 {
		return nil
	}

	rootHash, _, _ := maps.ProofsFromMap(ci.toMap())
	return rootHash
}

func (ci CommitInfo) ProofOp(storeName string) cmtprotocrypto.ProofOp {
	ret, err := ProofOpFromMap(ci.toMap(), storeName)
	if err != nil {
		panic(err)
	}
	return ret
}

func (ci CommitInfo) CommitID() CommitID {
	return CommitID{
		Version: ci.Version,
		Hash:    ci.Hash(),
	}
}
