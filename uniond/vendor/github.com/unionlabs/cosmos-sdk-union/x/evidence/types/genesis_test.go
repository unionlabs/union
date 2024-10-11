package types_test

import (
	"fmt"
	"testing"
	"time"

	gogoprotoany "github.com/cosmos/gogoproto/types/any"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/x/evidence/exported"
	"cosmossdk.io/x/evidence/types"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
)

func TestDefaultGenesisState(t *testing.T) {
	gs := types.DefaultGenesisState()
	require.NotNil(t, gs.Evidence)
	require.Len(t, gs.Evidence, 0)
}

func TestNewGenesisState(t *testing.T) {
	var evidence []exported.Evidence

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"can proto marshal",
			func() {
				evidence = []exported.Evidence{&TestEvidence{}}
			},
			true,
		},
	}

	for _, tc := range testCases {
		t.Run(fmt.Sprintf("Case %s", tc.msg), func(t *testing.T) {
			tc.malleate()

			if tc.expPass {
				require.NotPanics(t, func() {
					types.NewGenesisState(evidence)
				})
			} else {
				require.Panics(t, func() {
					types.NewGenesisState(evidence)
				})
			}
		})
	}
}

func TestGenesisStateValidate(t *testing.T) {
	var (
		genesisState *types.GenesisState
		testEvidence []exported.Evidence
		pk           = ed25519.GenPrivKey()
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"valid",
			func() {
				testEvidence = make([]exported.Evidence, 100)
				for i := 0; i < 100; i++ {
					testEvidence[i] = &types.Equivocation{
						Height:           int64(i + 1),
						Power:            100,
						Time:             time.Now().UTC(),
						ConsensusAddress: pk.PubKey().Address().String(),
					}
				}
				genesisState = types.NewGenesisState(testEvidence)
			},
			true,
		},
		{
			"invalid",
			func() {
				testEvidence = make([]exported.Evidence, 100)
				for i := 0; i < 100; i++ {
					testEvidence[i] = &types.Equivocation{
						Height:           int64(i),
						Power:            100,
						Time:             time.Now().UTC(),
						ConsensusAddress: pk.PubKey().Address().String(),
					}
				}
				genesisState = types.NewGenesisState(testEvidence)
			},
			false,
		},
		{
			"expected evidence",
			func() {
				genesisState = &types.GenesisState{
					Evidence: []*codectypes.Any{{}},
				}
			},
			false,
		},
	}

	for _, tc := range testCases {
		t.Run(fmt.Sprintf("Case %s", tc.msg), func(t *testing.T) {
			tc.malleate()

			if tc.expPass {
				require.NoError(t, genesisState.Validate())
			} else {
				require.Error(t, genesisState.Validate())
			}
		})
	}
}

func TestUnpackInterfaces(t *testing.T) {
	gs := types.GenesisState{
		Evidence: []*codectypes.Any{{}},
	}

	testCases := []struct {
		msg      string
		unpacker gogoprotoany.AnyUnpacker
		expPass  bool
	}{
		{
			"success",
			codectypes.NewInterfaceRegistry(),
			true,
		},
		{
			"error",
			codec.NewLegacyAmino(),
			false,
		},
	}

	for _, tc := range testCases {
		t.Run(fmt.Sprintf("Case %s", tc.msg), func(t *testing.T) {
			if tc.expPass {
				require.NoError(t, gs.UnpackInterfaces(tc.unpacker))
			} else {
				require.Error(t, gs.UnpackInterfaces(tc.unpacker))
			}
		})
	}
}

type TestEvidence struct{}

var _ exported.Evidence = &TestEvidence{}

func (*TestEvidence) String() string {
	return "test-string"
}

func (*TestEvidence) Route() string {
	return "test-route"
}

func (*TestEvidence) ProtoMessage() {}
func (*TestEvidence) Reset()        {}

func (*TestEvidence) Hash() []byte {
	return []byte("test-hash")
}

func (*TestEvidence) ValidateBasic() error {
	return nil
}

func (*TestEvidence) GetHeight() int64 {
	return 0
}
