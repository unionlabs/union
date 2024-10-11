package client

import (
	"context"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/validator/client/iface"

	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	validator2 "github.com/prysmaticlabs/prysm/v4/consensus-types/validator"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"go.opencensus.io/trace"
)

// HandleKeyReload makes sure the validator keeps operating correctly after a change to the underlying keys.
// It is also responsible for logging out information about the new state of keys.
func (v *validator) HandleKeyReload(ctx context.Context, currentKeys [][fieldparams.BLSPubkeyLength]byte) (anyActive bool, err error) {
	ctx, span := trace.StartSpan(ctx, "validator.HandleKeyReload")
	defer span.End()

	statusRequestKeys := make([][]byte, len(currentKeys))
	for i := range currentKeys {
		statusRequestKeys[i] = currentKeys[i][:]
	}
	resp, err := v.validatorClient.MultipleValidatorStatus(ctx, &eth.MultipleValidatorStatusRequest{
		PublicKeys: statusRequestKeys,
	})
	if err != nil {
		return false, err
	}
	statuses := make([]*validatorStatus, len(resp.Statuses))
	for i, s := range resp.Statuses {
		statuses[i] = &validatorStatus{
			publicKey: resp.PublicKeys[i],
			status:    s,
			index:     resp.Indices[i],
		}
	}

	// "-1" indicates that validator count endpoint is not supported by the beacon node.
	var valCount int64 = -1
	valCounts, err := v.prysmBeaconClient.GetValidatorCount(ctx, "head", []validator2.Status{validator2.Active})
	if err != nil && !errors.Is(err, iface.ErrNotSupported) {
		return false, errors.Wrap(err, "could not get active validator count")
	}

	if len(valCounts) > 0 {
		valCount = int64(valCounts[0].Count)
	}

	return v.checkAndLogValidatorStatus(statuses, valCount), nil
}
