package beacon_api

import (
	"bytes"
	"context"
	"encoding/json"
	"strconv"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

func (c *beaconApiValidatorClient) prepareBeaconProposer(ctx context.Context, recipients []*ethpb.PrepareBeaconProposerRequest_FeeRecipientContainer) error {
	jsonRecipients := make([]*shared.FeeRecipient, len(recipients))
	for index, recipient := range recipients {
		jsonRecipients[index] = &shared.FeeRecipient{
			FeeRecipient:   hexutil.Encode(recipient.FeeRecipient),
			ValidatorIndex: strconv.FormatUint(uint64(recipient.ValidatorIndex), 10),
		}
	}

	marshalledJsonRecipients, err := json.Marshal(jsonRecipients)
	if err != nil {
		return errors.Wrap(err, "failed to marshal recipients")
	}

	return c.jsonRestHandler.Post(ctx, "/eth/v1/validator/prepare_beacon_proposer", nil, bytes.NewBuffer(marshalledJsonRecipients), nil)
}
