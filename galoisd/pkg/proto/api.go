package proto

import "github.com/consensys/gnark/frontend"

const MaxVarintSize = 10

type ProtoAPI struct {
	api frontend.API
}

func NewProtoAPI(api frontend.API) *ProtoAPI {
	return &ProtoAPI{
		api: api,
	}
}

// Max varint is encoded in size(t)+1 bytes
// Return the number of bytes the varint was originally encoded with
func (p *ProtoAPI) DecodeVarint64(b []frontend.Variable) (frontend.Variable, frontend.Variable) {
	value := [MaxVarintSize * 8]frontend.Variable{}
	for i := 0; i < MaxVarintSize*8; i++ {
		value[i] = 0
	}
	done := frontend.Variable(0)
	size := frontend.Variable(0)
	for i := 0; i < MaxVarintSize; i++ {
		for j := 0; j < 7; j++ {
			value[(8*i)+j-i] = p.api.Select(done, 0, b[(8*i)+j])
		}
		size = p.api.Select(done, size, p.api.Add(size, 1))
		done = p.api.Or(done, p.api.IsZero(b[(8*i)+7]))
	}
	return p.api.FromBinary(value[:]...), size
}
