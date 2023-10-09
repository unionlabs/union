package proto

import (
	"math/rand"
	"testing"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/test"
	"github.com/gogo/protobuf/proto"
)

type protoCircuit struct {
	ProtoValue     frontend.Variable
	ExpectedValue  frontend.Variable
	ExpectedLength frontend.Variable
}

func Unpack(api frontend.API, packed frontend.Variable, sizeOfInput int, sizeOfElem int) []frontend.Variable {
	nbOfElems := sizeOfInput / sizeOfElem
	if sizeOfElem == 1 {
		return api.ToBinary(packed, nbOfElems)
	} else {
		unpacked := api.ToBinary(packed, sizeOfInput)
		elems := make([]frontend.Variable, nbOfElems)
		for i := 0; i < nbOfElems; i++ {
			elems[i] = api.FromBinary(unpacked[i*sizeOfElem : (i+1)*sizeOfElem]...)
		}
		return elems
	}
}

func (c *protoCircuit) Define(api frontend.API) error {
	protoValue := Unpack(api, c.ProtoValue, MaxVarintSize*8, 1)
	value, length := NewProtoAPI(api).DecodeVarint64(protoValue)
	api.AssertIsEqual(value, c.ExpectedValue)
	api.AssertIsEqual(length, c.ExpectedLength)
	return nil
}

func TestProto(t *testing.T) {
	reverseBytes := func(numbers []byte) []byte {
		newNumbers := make([]byte, 0, len(numbers))
		for i := len(numbers) - 1; i >= 0; i-- {
			newNumbers = append(newNumbers, numbers[i])
		}
		return newNumbers
	}
	value := rand.Uint64()
	protoValue := proto.EncodeVarint(value)
	circuit := protoCircuit{}
	assignment := protoCircuit{
		ProtoValue:     reverseBytes(protoValue),
		ExpectedValue:  value,
		ExpectedLength: len(protoValue),
	}
	test.NewAssert(t).ProverSucceeded(
		&circuit,
		&assignment,
		test.WithCurves(ecc.BN254),
		test.NoFuzzing(),
		test.WithCurves(ecc.BN254),
		test.WithBackends(backend.GROTH16),
	)
}
