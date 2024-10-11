package interop

import (
	"encoding/binary"
	"math/big"
	"sync"

	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/async"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/crypto/hash"
)

const (
	blsWithdrawalPrefixByte = byte(0)
)

// DeterministicallyGenerateKeys creates BLS private keys using a fixed curve order according to
// the algorithm specified in the Ethereum beacon chain specification interop mock start section found here:
// https://github.com/ethereum/eth2.0-pm/blob/a085c9870f3956d6228ed2a40cd37f0c6580ecd7/interop/mocked_start/README.md
func DeterministicallyGenerateKeys(startIndex, numKeys uint64) ([]bls.SecretKey, []bls.PublicKey, error) {
	privKeys := make([]bls.SecretKey, numKeys)
	pubKeys := make([]bls.PublicKey, numKeys)
	type keys struct {
		secrets []bls.SecretKey
		publics []bls.PublicKey
	}
	// lint:ignore uintcast -- this is safe because we can reasonably expect that the number of keys is less than max int64.
	results, err := async.Scatter(int(numKeys), func(offset int, entries int, _ *sync.RWMutex) (interface{}, error) {
		secs, pubs, err := deterministicallyGenerateKeys(uint64(offset)+startIndex, uint64(entries))
		return &keys{secrets: secs, publics: pubs}, err
	})
	if err != nil {
		return nil, nil, errors.Wrap(err, "failed to generate keys")
	}
	for _, result := range results {
		if keysExtent, ok := result.Extent.(*keys); ok {
			copy(privKeys[result.Offset:], keysExtent.secrets)
			copy(pubKeys[result.Offset:], keysExtent.publics)
		} else {
			return nil, nil, errors.New("extent not of expected type")
		}
	}
	return privKeys, pubKeys, nil
}

func deterministicallyGenerateKeys(startIndex, numKeys uint64) ([]bls.SecretKey, []bls.PublicKey, error) {
	privKeys := make([]bls.SecretKey, numKeys)
	pubKeys := make([]bls.PublicKey, numKeys)
	for i := startIndex; i < startIndex+numKeys; i++ {
		enc := make([]byte, 32)
		binary.LittleEndian.PutUint32(enc, uint32(i))
		h := hash.Hash(enc)
		// Reverse byte order to big endian for use with big ints.
		num := bytesutil.LittleEndianBytesToBigInt(h[:])
		order := new(big.Int)
		var ok bool
		order, ok = order.SetString(bls.CurveOrder, 10)
		if !ok {
			return nil, nil, errors.New("could not set bls curve order as big int")
		}
		num = num.Mod(num, order)
		numBytes := num.Bytes()
		// pad key at the start with zero bytes to make it into a 32 byte key
		if len(numBytes) < 32 {
			emptyBytes := make([]byte, 32-len(numBytes))
			numBytes = append(emptyBytes, numBytes...)
		}
		priv, err := bls.SecretKeyFromBytes(numBytes)
		if err != nil {
			return nil, nil, errors.Wrapf(err, "could not create bls secret key at index %d from raw bytes", i)
		}
		privKeys[i-startIndex] = priv
		pubKeys[i-startIndex] = priv.PublicKey()
	}
	return privKeys, pubKeys, nil
}
