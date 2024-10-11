package ssz

import (
	"bytes"
	"encoding/binary"

	"github.com/pkg/errors"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// Uint64Root computes the HashTreeRoot Merkleization of
// a simple uint64 value according to the Ethereum
// Simple Serialize specification.
func Uint64Root(val uint64) [32]byte {
	buf := make([]byte, 8)
	binary.LittleEndian.PutUint64(buf, val)
	root := bytesutil.ToBytes32(buf)
	return root
}

// ForkRoot computes the HashTreeRoot Merkleization of
// a Fork struct value according to the Ethereum
// Simple Serialize specification.
func ForkRoot(fork *ethpb.Fork) ([32]byte, error) {
	fieldRoots := make([][32]byte, 3)
	if fork != nil {
		fieldRoots[0] = bytesutil.ToBytes32(fork.PreviousVersion)
		fieldRoots[1] = bytesutil.ToBytes32(fork.CurrentVersion)
		forkEpochBuf := make([]byte, 8)
		binary.LittleEndian.PutUint64(forkEpochBuf, uint64(fork.Epoch))
		fieldRoots[2] = bytesutil.ToBytes32(forkEpochBuf)
	}
	return BitwiseMerkleize(fieldRoots, uint64(len(fieldRoots)), uint64(len(fieldRoots)))
}

// CheckpointRoot computes the HashTreeRoot Merkleization of
// a InitWithReset struct value according to the Ethereum
// Simple Serialize specification.
func CheckpointRoot(checkpoint *ethpb.Checkpoint) ([32]byte, error) {
	fieldRoots := make([][32]byte, 2)
	if checkpoint != nil {
		epochBuf := make([]byte, 8)
		binary.LittleEndian.PutUint64(epochBuf, uint64(checkpoint.Epoch))
		fieldRoots[0] = bytesutil.ToBytes32(epochBuf)
		fieldRoots[1] = bytesutil.ToBytes32(checkpoint.Root)
	}
	return BitwiseMerkleize(fieldRoots, uint64(len(fieldRoots)), uint64(len(fieldRoots)))
}

// ByteArrayRootWithLimit computes the HashTreeRoot Merkleization of
// a list of [32]byte roots according to the Ethereum Simple Serialize
// specification.
func ByteArrayRootWithLimit(roots [][]byte, limit uint64) ([32]byte, error) {
	newRoots := make([][32]byte, len(roots))
	for i, r := range roots {
		copy(newRoots[i][:], r)
	}
	result, err := BitwiseMerkleize(newRoots, uint64(len(newRoots)), limit)
	if err != nil {
		return [32]byte{}, errors.Wrap(err, "could not compute byte array merkleization")
	}
	buf := new(bytes.Buffer)
	if err := binary.Write(buf, binary.LittleEndian, uint64(len(newRoots))); err != nil {
		return [32]byte{}, errors.Wrap(err, "could not marshal byte array length")
	}
	// We need to mix in the length of the slice.
	output := make([]byte, 32)
	copy(output, buf.Bytes())
	mixedLen := MixInLength(result, output)
	return mixedLen, nil
}

// SlashingsRoot computes the HashTreeRoot Merkleization of
// a list of uint64 slashing values according to the Ethereum
// Simple Serialize specification.
func SlashingsRoot(slashings []uint64) ([32]byte, error) {
	slashingMarshaling := make([][]byte, fieldparams.SlashingsLength)
	for i := 0; i < len(slashings) && i < len(slashingMarshaling); i++ {
		slashBuf := make([]byte, 8)
		binary.LittleEndian.PutUint64(slashBuf, slashings[i])
		slashingMarshaling[i] = slashBuf
	}
	slashingChunks, err := PackByChunk(slashingMarshaling)
	if err != nil {
		return [32]byte{}, errors.Wrap(err, "could not pack slashings into chunks")
	}
	return BitwiseMerkleize(slashingChunks, uint64(len(slashingChunks)), uint64(len(slashingChunks)))
}

// TransactionsRoot computes the HTR for the Transactions' property of the ExecutionPayload
// The code was largely copy/pasted from the code generated to compute the HTR of the entire
// ExecutionPayload.
func TransactionsRoot(txs [][]byte) ([32]byte, error) {
	txRoots := make([][32]byte, 0)
	for i := 0; i < len(txs); i++ {
		rt, err := ByteSliceRoot(txs[i], fieldparams.MaxBytesPerTxLength) // getting the transaction root here
		if err != nil {
			return [32]byte{}, err
		}
		txRoots = append(txRoots, rt)
	}

	bytesRoot, err := BitwiseMerkleize(txRoots, uint64(len(txRoots)), fieldparams.MaxTxsPerPayloadLength)
	if err != nil {
		return [32]byte{}, errors.Wrap(err, "could not compute  merkleization")
	}
	bytesRootBuf := new(bytes.Buffer)
	if err := binary.Write(bytesRootBuf, binary.LittleEndian, uint64(len(txs))); err != nil {
		return [32]byte{}, errors.Wrap(err, "could not marshal length")
	}
	bytesRootBufRoot := make([]byte, 32)
	copy(bytesRootBufRoot, bytesRootBuf.Bytes())
	return MixInLength(bytesRoot, bytesRootBufRoot), nil
}

// WithdrawalSliceRoot computes the HTR of a slice of withdrawals.
// The limit parameter is used as input to the bitwise merkleization algorithm.
func WithdrawalSliceRoot(withdrawals []*enginev1.Withdrawal, limit uint64) ([32]byte, error) {
	roots := make([][32]byte, len(withdrawals))
	for i := 0; i < len(withdrawals); i++ {
		r, err := withdrawalRoot(withdrawals[i])
		if err != nil {
			return [32]byte{}, err
		}
		roots[i] = r
	}

	bytesRoot, err := BitwiseMerkleize(roots, uint64(len(roots)), limit)
	if err != nil {
		return [32]byte{}, errors.Wrap(err, "could not compute  merkleization")
	}
	bytesRootBuf := new(bytes.Buffer)
	if err := binary.Write(bytesRootBuf, binary.LittleEndian, uint64(len(withdrawals))); err != nil {
		return [32]byte{}, errors.Wrap(err, "could not marshal length")
	}
	bytesRootBufRoot := make([]byte, 32)
	copy(bytesRootBufRoot, bytesRootBuf.Bytes())
	return MixInLength(bytesRoot, bytesRootBufRoot), nil
}

// ByteSliceRoot is a helper func to merkleize an arbitrary List[Byte, N]
// this func runs Chunkify + MerkleizeVector
// max length is dividable by 32 ( root length )
func ByteSliceRoot(slice []byte, maxLength uint64) ([32]byte, error) {
	chunkedRoots, err := PackByChunk([][]byte{slice})
	if err != nil {
		return [32]byte{}, err
	}
	maxRootLength := (maxLength + 31) / 32 // nearest number divisible by root length (32)
	bytesRoot, err := BitwiseMerkleize(chunkedRoots, uint64(len(chunkedRoots)), maxRootLength)
	if err != nil {
		return [32]byte{}, errors.Wrap(err, "could not compute merkleization")
	}
	bytesRootBuf := new(bytes.Buffer)
	if err := binary.Write(bytesRootBuf, binary.LittleEndian, uint64(len(slice))); err != nil {
		return [32]byte{}, errors.Wrap(err, "could not marshal length")
	}
	bytesRootBufRoot := make([]byte, 32)
	copy(bytesRootBufRoot, bytesRootBuf.Bytes())
	return MixInLength(bytesRoot, bytesRootBufRoot), nil
}

func withdrawalRoot(w *enginev1.Withdrawal) ([32]byte, error) {
	fieldRoots := make([][32]byte, 4)
	if w != nil {
		binary.LittleEndian.PutUint64(fieldRoots[0][:], w.Index)

		binary.LittleEndian.PutUint64(fieldRoots[1][:], uint64(w.ValidatorIndex))

		fieldRoots[2] = bytesutil.ToBytes32(w.Address)
		binary.LittleEndian.PutUint64(fieldRoots[3][:], w.Amount)
	}
	return BitwiseMerkleize(fieldRoots, uint64(len(fieldRoots)), uint64(len(fieldRoots)))
}
