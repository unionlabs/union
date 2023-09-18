package cli_test

import (
	"github.com/cosmos/cosmos-sdk/testutil"
	clitestutil "github.com/cosmos/cosmos-sdk/testutil/cli"
	"github.com/cosmos/cosmos-sdk/x/genutil/client/cli"
)

// An example exported genesis file from a 0.37 chain. Note that evidence
// parameters only contains `max_age`.
var v037Exported = `{
	"app_hash": "",
	"app_state": {},
	"chain_id": "test",
	"consensus_params": {
		"block": {
		"max_bytes": "22020096",
		"max_gas": "-1",
		"time_iota_ms": "1000"
		},
		"evidence": { "max_age": "100000" },
		"validator": { "pub_key_types": ["ed25519"] }
	},
	"genesis_time": "2020-09-29T20:16:29.172362037Z",
	"validators": []
}`

// An example exported genesis file that's 0.40 compatible.
// We added the following app_state:
//
// - x/gov: added votes to test ADR-037 split votes migration.
var v040Valid = `{
	"app_hash": "",
	"app_state": {
		"gov": {
			"starting_proposal_id": "0",
			"deposits": [],
			"votes": [
			  {
				"proposal_id": "5",
				"voter": "cosmos1fl48vsnmsdzcv85q5d2q4z5ajdha8yu34mf0eh",
				"option": "VOTE_OPTION_YES"
			  }
			],
			"proposals": [],
			"deposit_params": { "min_deposit": [], "max_deposit_period": "0s" },
			"voting_params": { "voting_period": "0s" },
			"tally_params": { "quorum": "0", "threshold": "0", "veto_threshold": "0" }
		}	  
	},
	"chain_id": "test",
	"consensus_params": {
		"block": {
		"max_bytes": "22020096",
		"max_gas": "-1",
		"time_iota_ms": "1000"
		},
		"evidence": {
			"max_age_num_blocks": "100000",
			"max_age_duration": "172800000000000",
			"max_bytes": "0"
		},
		"validator": { "pub_key_types": ["ed25519"] }
	},
	"genesis_time": "2020-09-29T20:16:29.172362037Z",
	"validators": []
}`

func (s *CLITestSuite) TestValidateGenesis() {
	testCases := []struct {
		name    string
		genesis string
		expErr  bool
	}{
		{
			"exported 0.37 genesis file",
			v037Exported,
			true,
		},
		{
			"valid 0.40 genesis file",
			v040Valid,
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		s.Run(tc.name, func() {
			genesisFile := testutil.WriteToNewTempFile(s.T(), tc.genesis)
			_, err := clitestutil.ExecTestCLICmd(s.clientCtx, cli.ValidateGenesisCmd(nil), []string{genesisFile.Name()})
			if tc.expErr {
				s.Require().Contains(err.Error(), "Make sure that you have correctly migrated all Tendermint consensus params")
			} else {
				s.Require().NoError(err)
			}
		})
	}
}
