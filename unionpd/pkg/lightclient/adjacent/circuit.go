package adjacent

import (
	g2 "cometbls-prover/pkg/emulated"
	"cometbls-prover/pkg/lightclient"

	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/algebra/emulated/fields_bn254"
)

type Circuit struct {
	Input lightclient.TendermintLightClientInput
	ExpectedValRoot [2]frontend.Variable `gnark:",public"`
	Message                  fields_bn254.E2      `gnark:",public"`
}

func (circuit *Circuit) Define(api frontend.API) error {
	messagePoint := g2.MapToG2(api, &circuit.Message)
	lc := lightclient.NewTendermintLightClientAPI(api, &circuit.Input)
	return lc.Verify(messagePoint, circuit.ExpectedValRoot, 2, 3)
}
