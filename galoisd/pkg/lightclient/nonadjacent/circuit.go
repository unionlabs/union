package nonadjacent

import (
	"github.com/unionlabs/union/galoisd/pkg/lightclient"

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
	DomainSeparationTag frontend.Variable
	TrustedInput        TendermintNonAdjacentLightClientInput
	TrustedValRoot      frontend.Variable
	UntrustedInput      TendermintNonAdjacentLightClientInput
	Vote                lightclient.BlockVote
	Header              lightclient.BlockHeader
	InputsHash          frontend.Variable `gnark:",public"`
}

// Union whitepaper: Algorithm 2. procedure Main
func (circuit *Circuit) Define(api frontend.API) error {
	bhapi, err := lightclient.NewBlockHeaderAPI(api, circuit.Header, circuit.Vote)
	if err != nil {
		return err
	}
	bhapi.VerifyInputs(circuit.InputsHash, circuit.TrustedValRoot)
	hashedMessage, err := bhapi.HashToCurve(circuit.DomainSeparationTag)
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
	res := lc.Verify(hashedMessage, circuit.TrustedValRoot, TrustedRatioNum, TrustedRatioDen)
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
	return lc.Verify(hashedMessage, circuit.Header.ValidatorsHash, UntrustedRatioNum, UntrustedRatioDen)
}
