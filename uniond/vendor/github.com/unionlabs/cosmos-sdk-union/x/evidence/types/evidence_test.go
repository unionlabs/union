package types_test

import (
	"encoding/hex"
	"strings"
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/core/comet"
	"cosmossdk.io/x/evidence/types"

	"github.com/cosmos/cosmos-sdk/codec/address"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

func TestEquivocation_Valid(t *testing.T) {
	consCodec := address.NewBech32Codec("cosmosvalcons")
	n, _ := time.Parse(time.RFC3339, "2006-01-02T15:04:05Z")
	addr, err := consCodec.BytesToString(sdk.ConsAddress("foo_________________"))
	require.NoError(t, err)

	e := types.Equivocation{
		Height:           100,
		Time:             n,
		Power:            1000000,
		ConsensusAddress: addr,
	}

	consAddr, err := consCodec.BytesToString(e.GetConsensusAddress(consCodec))
	require.NoError(t, err)
	require.Equal(t, e.GetTotalPower(), int64(0))
	require.Equal(t, e.GetValidatorPower(), e.Power)
	require.Equal(t, e.GetTime(), e.Time)
	require.Equal(t, consAddr, e.ConsensusAddress)
	require.Equal(t, e.GetHeight(), e.Height)
	require.Equal(t, e.Route(), types.RouteEquivocation)
	require.Equal(t, strings.ToUpper(hex.EncodeToString(e.Hash())), "1E10F9267BEA3A9A4AB5302C2C510CC1AFD7C54E232DA5B2E3360DFAFACF7A76")
	require.Equal(t, "height:100 time:<seconds:1136214245 > power:1000000 consensus_address:\"cosmosvalcons1vehk7h6lta047h6lta047h6lta047h6l8m4r53\" ", e.String())
	require.NoError(t, e.ValidateBasic())

	require.Equal(t, int64(0), e.GetTotalPower())
	require.Equal(t, e.Power, e.GetValidatorPower())
	require.Equal(t, e.Time, e.GetTime())
	require.Equal(t, e.ConsensusAddress, consAddr)
	require.Equal(t, e.Height, e.GetHeight())
	require.Equal(t, types.RouteEquivocation, e.Route())
	require.Equal(t, "1E10F9267BEA3A9A4AB5302C2C510CC1AFD7C54E232DA5B2E3360DFAFACF7A76", strings.ToUpper(hex.EncodeToString(e.Hash())))
	require.Equal(t, "height:100 time:<seconds:1136214245 > power:1000000 consensus_address:\"cosmosvalcons1vehk7h6lta047h6lta047h6lta047h6l8m4r53\" ", e.String())
	require.NoError(t, e.ValidateBasic())
}

func TestEquivocationValidateBasic(t *testing.T) {
	var zeroTime time.Time
	addr, err := address.NewBech32Codec("cosmosvalcons").BytesToString(sdk.ConsAddress("foo_________________"))
	require.NoError(t, err)

	n, _ := time.Parse(time.RFC3339, "2006-01-02T15:04:05Z")
	testCases := []struct {
		name      string
		e         types.Equivocation
		expectErr bool
	}{
		{"valid", types.Equivocation{100, n, 1000000, addr}, false},
		{"invalid time", types.Equivocation{100, zeroTime, 1000000, addr}, true},
		{"invalid height", types.Equivocation{0, n, 1000000, addr}, true},
		{"invalid power", types.Equivocation{100, n, 0, addr}, true},
		{"invalid address", types.Equivocation{100, n, 1000000, ""}, true},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			require.Equal(t, tc.expectErr, tc.e.ValidateBasic() != nil)
		})
	}
}

func TestEvidenceAddressConversion(t *testing.T) {
	sdk.GetConfig().SetBech32PrefixForConsensusNode("testcnclcons", "testcnclconspub")
	tmEvidence := NewCometMisbehavior(1, 100, time.Now(), comet.DuplicateVote,
		comet.Validator{Address: []byte{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}, Power: 100})

	evidence := types.FromABCIEvidence(tmEvidence, address.NewBech32Codec("testcnclcons"))
	consAddr := evidence.GetConsensusAddress(address.NewBech32Codec("testcnclcons"))
	// Check the address is the same after conversion
	require.Equal(t, tmEvidence.Validator.Address, consAddr.Bytes())
	sdk.GetConfig().SetBech32PrefixForConsensusNode(sdk.Bech32PrefixConsAddr, sdk.Bech32PrefixConsPub)
}

func NewCometMisbehavior(height, tvp int64, t time.Time, tpe comet.MisbehaviorType, val comet.Validator) comet.Evidence {
	return comet.Evidence{
		Height:           height,
		Time:             t,
		TotalVotingPower: tvp,
		Type:             tpe,
		Validator:        val,
	}
}
