package iface

import (
	"context"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/validator"
)

var ErrNotSupported = errors.New("endpoint not supported")

type ValidatorCount struct {
	Status string
	Count  uint64
}

// PrysmBeaconChainClient defines an interface required to implement all the prysm specific custom endpoints.
type PrysmBeaconChainClient interface {
	GetValidatorCount(context.Context, string, []validator.Status) ([]ValidatorCount, error)
}
