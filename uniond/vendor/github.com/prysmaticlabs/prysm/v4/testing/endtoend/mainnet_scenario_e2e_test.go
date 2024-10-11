package endtoend

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/types"
)

func TestEndToEnd_MultiScenarioRun_Multiclient(t *testing.T) {
	runner := e2eMainnet(t, false, true, types.InitForkCfg(version.Phase0, version.Deneb, params.E2EMainnetTestConfig()), types.WithEpochs(24))
	runner.config.Evaluators = scenarioEvalsMulti()
	runner.config.EvalInterceptor = runner.multiScenarioMulticlient
	runner.scenarioRunner()
}
