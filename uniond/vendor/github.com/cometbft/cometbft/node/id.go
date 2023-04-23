package node

import (
	"time"

	"github.com/cometbft/cometbft/crypto"
)

type ID struct {
	Name   string
	PubKey crypto.PubKey
}

type PrivNodeID struct {
	ID
	PrivKey crypto.PrivKey
}

type Greeting struct {
	ID
	Version string
	ChainID string
	Message string
	Time    time.Time
}

type SignedNodeGreeting struct {
	Greeting
	Signature []byte
}

func (pnid *PrivNodeID) SignGreeting() *SignedNodeGreeting {
	// greeting := NodeGreeting{}
	return nil
}
