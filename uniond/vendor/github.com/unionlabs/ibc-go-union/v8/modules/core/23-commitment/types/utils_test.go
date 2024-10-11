package types_test

import (
	"fmt"

	"github.com/stretchr/testify/require"

	storetypes "cosmossdk.io/store/types"

	cmtprotocrypto "github.com/cometbft/cometbft/api/cometbft/crypto/v1"

	"github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
)

func (suite *MerkleTestSuite) TestConvertProofs() {
	suite.iavlStore.Set([]byte("MYKEY"), []byte("MYVALUE"))
	cid := suite.store.Commit()

	root := types.NewMerkleRoot(cid.Hash)
	existsPath := types.NewMerklePath(suite.storeKey.Name(), "MYKEY")
	nonexistPath := types.NewMerklePath(suite.storeKey.Name(), "NOTMYKEY")
	value := []byte("MYVALUE")

	var proofOps *cmtprotocrypto.ProofOps
	testcases := []struct {
		name      string
		malleate  func()
		keyExists bool
		expPass   bool
	}{
		{
			"success for ExistenceProof",
			func() {
				res, err := suite.store.Query(&storetypes.RequestQuery{
					Path:  fmt.Sprintf("/%s/key", suite.storeKey.Name()), // required path to get key/value+proof
					Data:  []byte("MYKEY"),
					Prove: true,
				})
				require.NoError(suite.T(), err)
				require.NotNil(suite.T(), res.ProofOps)

				proofOps = res.ProofOps
			},
			true, true,
		},
		{
			"success for NonexistenceProof",
			func() {
				res, err := suite.store.Query(&storetypes.RequestQuery{
					Path:  fmt.Sprintf("/%s/key", suite.storeKey.Name()), // required path to get key/value+proof
					Data:  []byte("NOTMYKEY"),
					Prove: true,
				})
				require.NoError(suite.T(), err)
				require.NotNil(suite.T(), res.ProofOps)

				proofOps = res.ProofOps
			},
			false, true,
		},
		{
			"nil proofOps",
			func() {
				proofOps = nil
			},
			true, false,
		},
		{
			"proof op data is nil",
			func() {
				res, err := suite.store.Query(&storetypes.RequestQuery{
					Path:  fmt.Sprintf("/%s/key", suite.storeKey.Name()), // required path to get key/value+proof
					Data:  []byte("MYKEY"),
					Prove: true,
				})
				require.NoError(suite.T(), err)
				require.NotNil(suite.T(), res.ProofOps)

				proofOps = res.ProofOps
				proofOps.Ops[0].Data = nil
			},
			true, false,
		},
	}

	for _, tc := range testcases {
		tc := tc

		tc.malleate()

		proof, err := types.ConvertProofs(proofOps)
		if tc.expPass {
			suite.Require().NoError(err, "ConvertProofs unexpectedly returned error for case: %s", tc.name)
			if tc.keyExists {
				err := proof.VerifyMembership(types.GetSDKSpecs(), &root, existsPath, value)
				suite.Require().NoError(err, "converted proof failed to verify membership for case: %s", tc.name)
			} else {
				err := proof.VerifyNonMembership(types.GetSDKSpecs(), &root, nonexistPath)
				suite.Require().NoError(err, "converted proof failed to verify membership for case: %s", tc.name)
			}
		} else {
			suite.Require().Error(err, "ConvertProofs passed on invalid case for case: %s", tc.name)
		}
	}
}
