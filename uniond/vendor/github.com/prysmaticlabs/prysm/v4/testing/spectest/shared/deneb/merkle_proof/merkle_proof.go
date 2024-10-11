package merkle_proof

import (
	"testing"

	common "github.com/prysmaticlabs/prysm/v4/testing/spectest/shared/common/merkle_proof"
	"github.com/prysmaticlabs/prysm/v4/testing/spectest/shared/deneb/ssz_static"
)

func RunMerkleProofTests(t *testing.T, config string) {
	common.RunMerkleProofTests(t, config, "deneb", ssz_static.UnmarshalledSSZ)
}
