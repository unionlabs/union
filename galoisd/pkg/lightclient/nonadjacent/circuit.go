package nonadjacent

import (
	"galois/pkg/emulated"
	"galois/pkg/lightclient"

	"github.com/consensys/gnark/frontend"
	gadget "github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
)

const (
	TrustedRatioNum = 1
	TrustedRatioDen = 3

	UntrustedRatioNum = 2
	UntrustedRatioDen = 3
)

type TendermintNonAdjacentLightClientInput struct {
	Sig           gadget.G2Affine
	Validators    [lightclient.MaxVal]lightclient.Validator
	NbOfVal       frontend.Variable
	NbOfSignature frontend.Variable
	Bitmap        frontend.Variable
}

type Circuit struct {
	TrustedInput             TendermintNonAdjacentLightClientInput
	UntrustedInput           TendermintNonAdjacentLightClientInput
	ExpectedTrustedValRoot   frontend.Variable `gnark:",public"`
	ExpectedUntrustedValRoot frontend.Variable `gnark:",public"`
	Message                  frontend.Variable `gnark:",public"`
}

func (circuit *Circuit) Define(api frontend.API) error {
	emulatedAPI, err := g2.NewEmulatedAPI(api)
	if err != nil {
		return err
	}
	messagePoint, err := emulatedAPI.HashToG2(circuit.Message, 0)
	if err != nil {
		return err
	}
	lc := lightclient.NewTendermintLightClientAPI(api, &lightclient.TendermintLightClientInput{
		Sig:           circuit.TrustedInput.Sig,
		Validators:    circuit.TrustedInput.Validators,
		NbOfVal:       circuit.TrustedInput.NbOfVal,
		NbOfSignature: circuit.TrustedInput.NbOfSignature,
		Bitmap:        circuit.TrustedInput.Bitmap,
	})
	res := lc.Verify(messagePoint, circuit.ExpectedTrustedValRoot, TrustedRatioNum, TrustedRatioDen)
	if res != nil {
		return res
	}
	lc = lightclient.NewTendermintLightClientAPI(api, &lightclient.TendermintLightClientInput{
		Sig:           circuit.UntrustedInput.Sig,
		Validators:    circuit.UntrustedInput.Validators,
		NbOfVal:       circuit.UntrustedInput.NbOfVal,
		NbOfSignature: circuit.UntrustedInput.NbOfSignature,
		Bitmap:        circuit.UntrustedInput.Bitmap,
	})
	return lc.Verify(messagePoint, circuit.ExpectedUntrustedValRoot, UntrustedRatioNum, UntrustedRatioDen)
}
