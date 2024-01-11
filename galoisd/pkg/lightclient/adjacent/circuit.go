package adjacent

import (
	"galois/pkg/emulated"
	"galois/pkg/lightclient"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/algebra/emulated/fields_bn254"
)

type Circuit struct {
	Input           lightclient.TendermintLightClientInput
	ExpectedValRoot frontend.Variable    `gnark:",public"`
	Message         [2]frontend.Variable `gnark:",public"`
}

func (circuit *Circuit) Define(api frontend.API) error {
	emulatedAPI, err := g2.NewEmulatedAPI(api)
	if err != nil {
		return err
	}
	var message fields_bn254.E2
	message.A0.Limbs = lightclient.Unpack(api, circuit.Message[0], 256, 64)
	message.A1.Limbs = lightclient.Unpack(api, circuit.Message[1], 256, 64)
	messagePoint := emulatedAPI.HashToG2(&message)
	lc := lightclient.NewTendermintLightClientAPI(api, &circuit.Input)
	return lc.Verify(messagePoint, circuit.ExpectedValRoot, 2, 3)
}
