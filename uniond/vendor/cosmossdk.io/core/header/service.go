package header

import (
	"context"
	"crypto/sha256"
	"encoding/binary"
	"errors"
	"time"
)

// Service defines the interface in which you can get header information
type Service interface {
	HeaderInfo(context.Context) Info
}

// Info defines a struct that contains information about the header
type Info struct {
	Height  int64     // Height returns the height of the block
	Hash    []byte    // Hash returns the hash of the block header
	Time    time.Time // Time returns the time of the block
	AppHash []byte    // AppHash used in the current block header
	ChainID string    // ChainId returns the chain ID of the block
}

const hashSize = sha256.Size

// Bytes encodes the Info struct into a byte slice using little-endian encoding
func (i *Info) Bytes() ([]byte, error) {
	buf := make([]byte, 0)

	// Encode Height
	heightBytes := make([]byte, 8)
	binary.LittleEndian.PutUint64(heightBytes, uint64(i.Height))
	buf = append(buf, heightBytes...)

	// Encode Hash
	if len(i.Hash) != hashSize {
		return nil, errors.New("invalid hash size")
	}

	buf = append(buf, i.Hash...)

	// Encode Time
	timeBytes := make([]byte, 8)
	binary.LittleEndian.PutUint64(timeBytes, uint64(i.Time.Unix()))
	buf = append(buf, timeBytes...)

	// Encode AppHash
	if len(i.AppHash) != hashSize {
		return nil, errors.New("invalid hash size")
	}
	buf = append(buf, i.AppHash...)

	// Encode ChainID
	chainIDLen := len(i.ChainID)
	buf = append(buf, byte(chainIDLen))
	buf = append(buf, []byte(i.ChainID)...)

	return buf, nil
}

// FromBytes decodes the byte slice into an Info struct using little-endian encoding
func (i *Info) FromBytes(bytes []byte) error {
	// Decode Height
	i.Height = int64(binary.LittleEndian.Uint64(bytes[:8]))
	bytes = bytes[8:]

	// Decode Hash
	i.Hash = make([]byte, hashSize)
	copy(i.Hash, bytes[:hashSize])
	bytes = bytes[hashSize:]

	// Decode Time
	unixTime := int64(binary.LittleEndian.Uint64(bytes[:8]))
	i.Time = time.Unix(unixTime, 0).UTC()
	bytes = bytes[8:]

	// Decode AppHash
	i.AppHash = make([]byte, hashSize)
	copy(i.AppHash, bytes[:hashSize])
	bytes = bytes[hashSize:]

	// Decode ChainID
	chainIDLen := int(bytes[0])
	bytes = bytes[1:]
	if len(bytes) < chainIDLen {
		return errors.New("invalid byte slice length")
	}
	i.ChainID = string(bytes[:chainIDLen])

	return nil
}
