package beacon_api

import (
	"context"
	"time"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/golang/protobuf/ptypes/empty"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/validator/client/iface"
)

type ValidatorClientOpt func(*beaconApiValidatorClient)

func WithEventHandler(h *EventHandler) ValidatorClientOpt {
	return func(c *beaconApiValidatorClient) {
		c.eventHandler = h
	}
}

type beaconApiValidatorClient struct {
	genesisProvider         GenesisProvider
	dutiesProvider          dutiesProvider
	stateValidatorsProvider StateValidatorsProvider
	jsonRestHandler         JsonRestHandler
	eventHandler            *EventHandler
	beaconBlockConverter    BeaconBlockConverter
	prysmBeaconChainCLient  iface.PrysmBeaconChainClient
}

func NewBeaconApiValidatorClient(jsonRestHandler JsonRestHandler, opts ...ValidatorClientOpt) iface.ValidatorClient {
	c := &beaconApiValidatorClient{
		genesisProvider:         beaconApiGenesisProvider{jsonRestHandler: jsonRestHandler},
		dutiesProvider:          beaconApiDutiesProvider{jsonRestHandler: jsonRestHandler},
		stateValidatorsProvider: beaconApiStateValidatorsProvider{jsonRestHandler: jsonRestHandler},
		jsonRestHandler:         jsonRestHandler,
		beaconBlockConverter:    beaconApiBeaconBlockConverter{},
		prysmBeaconChainCLient: prysmBeaconChainClient{
			nodeClient:      &beaconApiNodeClient{jsonRestHandler: jsonRestHandler},
			jsonRestHandler: jsonRestHandler,
		},
	}
	for _, o := range opts {
		o(c)
	}
	return c
}

func (c *beaconApiValidatorClient) GetDuties(ctx context.Context, in *ethpb.DutiesRequest) (*ethpb.DutiesResponse, error) {
	return c.getDuties(ctx, in)
}

func (c *beaconApiValidatorClient) CheckDoppelGanger(ctx context.Context, in *ethpb.DoppelGangerRequest) (*ethpb.DoppelGangerResponse, error) {
	return c.checkDoppelGanger(ctx, in)
}

func (c *beaconApiValidatorClient) DomainData(ctx context.Context, in *ethpb.DomainRequest) (*ethpb.DomainResponse, error) {
	if len(in.Domain) != 4 {
		return nil, errors.Errorf("invalid domain type: %s", hexutil.Encode(in.Domain))
	}

	domainType := bytesutil.ToBytes4(in.Domain)
	return c.getDomainData(ctx, in.Epoch, domainType)
}

func (c *beaconApiValidatorClient) GetAttestationData(ctx context.Context, in *ethpb.AttestationDataRequest) (*ethpb.AttestationData, error) {
	if in == nil {
		return nil, errors.New("GetAttestationData received nil argument `in`")
	}

	return c.getAttestationData(ctx, in.Slot, in.CommitteeIndex)
}

func (c *beaconApiValidatorClient) GetBeaconBlock(ctx context.Context, in *ethpb.BlockRequest) (*ethpb.GenericBeaconBlock, error) {
	return c.getBeaconBlock(ctx, in.Slot, in.RandaoReveal, in.Graffiti)
}

func (c *beaconApiValidatorClient) GetFeeRecipientByPubKey(_ context.Context, _ *ethpb.FeeRecipientByPubKeyRequest) (*ethpb.FeeRecipientByPubKeyResponse, error) {
	return nil, nil
}

func (c *beaconApiValidatorClient) GetSyncCommitteeContribution(ctx context.Context, in *ethpb.SyncCommitteeContributionRequest) (*ethpb.SyncCommitteeContribution, error) {
	return c.getSyncCommitteeContribution(ctx, in)
}

func (c *beaconApiValidatorClient) GetSyncMessageBlockRoot(ctx context.Context, _ *empty.Empty) (*ethpb.SyncMessageBlockRootResponse, error) {
	return c.getSyncMessageBlockRoot(ctx)
}

func (c *beaconApiValidatorClient) GetSyncSubcommitteeIndex(ctx context.Context, in *ethpb.SyncSubcommitteeIndexRequest) (*ethpb.SyncSubcommitteeIndexResponse, error) {
	return c.getSyncSubcommitteeIndex(ctx, in)
}

func (c *beaconApiValidatorClient) MultipleValidatorStatus(ctx context.Context, in *ethpb.MultipleValidatorStatusRequest) (*ethpb.MultipleValidatorStatusResponse, error) {
	return c.multipleValidatorStatus(ctx, in)
}

func (c *beaconApiValidatorClient) PrepareBeaconProposer(ctx context.Context, in *ethpb.PrepareBeaconProposerRequest) (*empty.Empty, error) {
	return new(empty.Empty), c.prepareBeaconProposer(ctx, in.Recipients)
}

func (c *beaconApiValidatorClient) ProposeAttestation(ctx context.Context, in *ethpb.Attestation) (*ethpb.AttestResponse, error) {
	return c.proposeAttestation(ctx, in)
}

func (c *beaconApiValidatorClient) ProposeBeaconBlock(ctx context.Context, in *ethpb.GenericSignedBeaconBlock) (*ethpb.ProposeResponse, error) {
	return c.proposeBeaconBlock(ctx, in)
}

func (c *beaconApiValidatorClient) ProposeExit(ctx context.Context, in *ethpb.SignedVoluntaryExit) (*ethpb.ProposeExitResponse, error) {
	return c.proposeExit(ctx, in)
}

func (c *beaconApiValidatorClient) StreamSlots(ctx context.Context, in *ethpb.StreamSlotsRequest) (ethpb.BeaconNodeValidator_StreamSlotsClient, error) {
	return c.streamSlots(ctx, in, time.Second), nil
}

func (c *beaconApiValidatorClient) StreamBlocksAltair(ctx context.Context, in *ethpb.StreamBlocksRequest) (ethpb.BeaconNodeValidator_StreamBlocksAltairClient, error) {
	return c.streamBlocks(ctx, in, time.Second), nil
}

func (c *beaconApiValidatorClient) SubmitAggregateSelectionProof(ctx context.Context, in *ethpb.AggregateSelectionRequest) (*ethpb.AggregateSelectionResponse, error) {
	return c.submitAggregateSelectionProof(ctx, in)
}

func (c *beaconApiValidatorClient) SubmitSignedAggregateSelectionProof(ctx context.Context, in *ethpb.SignedAggregateSubmitRequest) (*ethpb.SignedAggregateSubmitResponse, error) {
	return c.submitSignedAggregateSelectionProof(ctx, in)
}

func (c *beaconApiValidatorClient) SubmitSignedContributionAndProof(ctx context.Context, in *ethpb.SignedContributionAndProof) (*empty.Empty, error) {
	return new(empty.Empty), c.submitSignedContributionAndProof(ctx, in)
}

func (c *beaconApiValidatorClient) SubmitSyncMessage(ctx context.Context, in *ethpb.SyncCommitteeMessage) (*empty.Empty, error) {
	return new(empty.Empty), c.submitSyncMessage(ctx, in)
}

func (c *beaconApiValidatorClient) SubmitValidatorRegistrations(ctx context.Context, in *ethpb.SignedValidatorRegistrationsV1) (*empty.Empty, error) {
	return new(empty.Empty), c.submitValidatorRegistrations(ctx, in.Messages)
}

func (c *beaconApiValidatorClient) SubscribeCommitteeSubnets(ctx context.Context, in *ethpb.CommitteeSubnetsSubscribeRequest, validatorIndices []primitives.ValidatorIndex) (*empty.Empty, error) {
	return new(empty.Empty), c.subscribeCommitteeSubnets(ctx, in, validatorIndices)
}

func (c *beaconApiValidatorClient) ValidatorIndex(ctx context.Context, in *ethpb.ValidatorIndexRequest) (*ethpb.ValidatorIndexResponse, error) {
	return c.validatorIndex(ctx, in)
}

func (c *beaconApiValidatorClient) ValidatorStatus(ctx context.Context, in *ethpb.ValidatorStatusRequest) (*ethpb.ValidatorStatusResponse, error) {
	return c.validatorStatus(ctx, in)
}

func (c *beaconApiValidatorClient) WaitForActivation(ctx context.Context, in *ethpb.ValidatorActivationRequest) (ethpb.BeaconNodeValidator_WaitForActivationClient, error) {
	return c.waitForActivation(ctx, in)
}

// Deprecated: Do not use.
func (c *beaconApiValidatorClient) WaitForChainStart(ctx context.Context, _ *empty.Empty) (*ethpb.ChainStartResponse, error) {
	return c.waitForChainStart(ctx)
}

func (c *beaconApiValidatorClient) StartEventStream(ctx context.Context) error {
	if c.eventHandler != nil {
		if err := c.eventHandler.get(ctx, []string{"head"}); err != nil {
			return errors.Wrapf(err, "could not invoke event handler")
		}
	}
	return nil
}

func (c *beaconApiValidatorClient) EventStreamIsRunning() bool {
	return c.eventHandler.running
}
