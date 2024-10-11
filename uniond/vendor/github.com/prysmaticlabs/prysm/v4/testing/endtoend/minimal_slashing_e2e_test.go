package endtoend

import (
	"fmt"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	ev "github.com/prysmaticlabs/prysm/v4/testing/endtoend/evaluators"
	e2eParams "github.com/prysmaticlabs/prysm/v4/testing/endtoend/params"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/types"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestEndToEnd_Slasher_MinimalConfig(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	params.OverrideBeaconConfig(params.E2ETestConfig().Copy())
	require.NoError(t, e2eParams.Init(t, e2eParams.StandardBeaconCount))

	tracingPort := e2eParams.TestParams.Ports.JaegerTracingPort
	tracingEndpoint := fmt.Sprintf("127.0.0.1:%d", tracingPort)

	testConfig := &types.E2EConfig{
		BeaconFlags: []string{
			"--slasher",
		},
		ValidatorFlags: []string{},
		EpochsToRun:    6,
		TestSync:       false,
		TestFeature:    false,
		TestDeposits:   false,
		Evaluators: []types.Evaluator{
			ev.PeersConnect,
			ev.HealthzCheck,
			ev.ValidatorsSlashedAfterEpoch(4),
			ev.SlashedValidatorsLoseBalanceAfterEpoch(4),
			ev.InjectDoubleVoteOnEpoch(2),
			ev.InjectDoubleBlockOnEpoch(2),
		},
		EvalInterceptor:     defaultInterceptor,
		TracingSinkEndpoint: tracingEndpoint,
	}

	newTestRunner(t, testConfig).run()
}
