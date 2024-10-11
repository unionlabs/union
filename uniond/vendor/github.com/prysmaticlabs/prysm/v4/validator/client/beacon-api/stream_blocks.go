package beacon_api

import (
	"bytes"
	"context"
	"encoding/json"
	"strconv"
	"time"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/events"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"google.golang.org/grpc"
)

type abstractSignedBlockResponseJson struct {
	Version             string          `json:"version" enum:"true"`
	ExecutionOptimistic bool            `json:"execution_optimistic"`
	Finalized           bool            `json:"finalized"`
	Data                json.RawMessage `json:"data"`
}

type streamSlotsClient struct {
	grpc.ClientStream
	ctx                context.Context
	beaconApiClient    beaconApiValidatorClient
	streamSlotsRequest *ethpb.StreamSlotsRequest
	pingDelay          time.Duration
	ch                 chan event
}

type streamBlocksAltairClient struct {
	grpc.ClientStream
	ctx                 context.Context
	beaconApiClient     beaconApiValidatorClient
	streamBlocksRequest *ethpb.StreamBlocksRequest
	prevBlockSlot       primitives.Slot
	pingDelay           time.Duration
}

type headSignedBeaconBlockResult struct {
	streamBlocksResponse *ethpb.StreamBlocksResponse
	executionOptimistic  bool
	slot                 primitives.Slot
}

func (c beaconApiValidatorClient) streamSlots(ctx context.Context, in *ethpb.StreamSlotsRequest, pingDelay time.Duration) ethpb.BeaconNodeValidator_StreamSlotsClient {
	ch := make(chan event, 1)
	c.eventHandler.subscribe(eventSub{name: "stream slots", ch: ch})
	return &streamSlotsClient{
		ctx:                ctx,
		beaconApiClient:    c,
		streamSlotsRequest: in,
		pingDelay:          pingDelay,
		ch:                 ch,
	}
}

func (c beaconApiValidatorClient) streamBlocks(ctx context.Context, in *ethpb.StreamBlocksRequest, pingDelay time.Duration) ethpb.BeaconNodeValidator_StreamBlocksAltairClient {
	return &streamBlocksAltairClient{
		ctx:                 ctx,
		beaconApiClient:     c,
		streamBlocksRequest: in,
		pingDelay:           pingDelay,
	}
}

func (c *streamSlotsClient) Recv() (*ethpb.StreamSlotsResponse, error) {
	for {
		select {
		case rawEvent := <-c.ch:
			if rawEvent.eventType != events.HeadTopic {
				continue
			}
			e := &events.HeadEvent{}
			if err := json.Unmarshal([]byte(rawEvent.data), e); err != nil {
				return nil, errors.Wrap(err, "failed to unmarshal head event into JSON")
			}
			uintSlot, err := strconv.ParseUint(e.Slot, 10, 64)
			if err != nil {
				return nil, errors.Wrap(err, "failed to parse slot")
			}
			return &ethpb.StreamSlotsResponse{
				Slot: primitives.Slot(uintSlot),
			}, nil
		case <-c.ctx.Done():
			return nil, errors.New("context canceled")
		}
	}
}

func (c *streamBlocksAltairClient) Recv() (*ethpb.StreamBlocksResponse, error) {
	result, err := c.beaconApiClient.getHeadSignedBeaconBlock(c.ctx)
	if err != nil {
		return nil, errors.Wrap(err, "failed to get latest signed block")
	}

	// We keep querying the beacon chain for the latest block until we receive a new slot
	for (c.streamBlocksRequest.VerifiedOnly && result.executionOptimistic) || c.prevBlockSlot == result.slot {
		select {
		case <-time.After(c.pingDelay):
			result, err = c.beaconApiClient.getHeadSignedBeaconBlock(c.ctx)
			if err != nil {
				return nil, errors.Wrap(err, "failed to get latest signed block")
			}
		case <-c.ctx.Done():
			return nil, errors.New("context canceled")
		}
	}

	c.prevBlockSlot = result.slot
	return result.streamBlocksResponse, nil
}

func (c beaconApiValidatorClient) getHeadSignedBeaconBlock(ctx context.Context) (*headSignedBeaconBlockResult, error) {
	// Since we don't know yet what the json looks like, we unmarshal into an abstract structure that has only a version
	// and a blob of data
	signedBlockResponseJson := abstractSignedBlockResponseJson{}
	if err := c.jsonRestHandler.Get(ctx, "/eth/v2/beacon/blocks/head", &signedBlockResponseJson); err != nil {
		return nil, err
	}

	// Once we know what the consensus version is, we can go ahead and unmarshal into the specific structs unique to each version
	decoder := json.NewDecoder(bytes.NewReader(signedBlockResponseJson.Data))

	response := &ethpb.StreamBlocksResponse{}
	var slot primitives.Slot

	switch signedBlockResponseJson.Version {
	case "phase0":
		jsonPhase0Block := shared.SignedBeaconBlock{}
		if err := decoder.Decode(&jsonPhase0Block); err != nil {
			return nil, errors.Wrap(err, "failed to decode signed phase0 block response json")
		}

		phase0Block, err := c.beaconBlockConverter.ConvertRESTPhase0BlockToProto(jsonPhase0Block.Message)
		if err != nil {
			return nil, errors.Wrap(err, "failed to get signed phase0 block")
		}

		decodedSignature, err := hexutil.Decode(jsonPhase0Block.Signature)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to decode phase0 block signature `%s`", jsonPhase0Block.Signature)
		}

		response.Block = &ethpb.StreamBlocksResponse_Phase0Block{
			Phase0Block: &ethpb.SignedBeaconBlock{
				Signature: decodedSignature,
				Block:     phase0Block,
			},
		}

		slot = phase0Block.Slot

	case "altair":
		jsonAltairBlock := shared.SignedBeaconBlockAltair{}
		if err := decoder.Decode(&jsonAltairBlock); err != nil {
			return nil, errors.Wrap(err, "failed to decode signed altair block response json")
		}

		altairBlock, err := c.beaconBlockConverter.ConvertRESTAltairBlockToProto(jsonAltairBlock.Message)
		if err != nil {
			return nil, errors.Wrap(err, "failed to get signed altair block")
		}

		decodedSignature, err := hexutil.Decode(jsonAltairBlock.Signature)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to decode altair block signature `%s`", jsonAltairBlock.Signature)
		}

		response.Block = &ethpb.StreamBlocksResponse_AltairBlock{
			AltairBlock: &ethpb.SignedBeaconBlockAltair{
				Signature: decodedSignature,
				Block:     altairBlock,
			},
		}

		slot = altairBlock.Slot

	case "bellatrix":
		jsonBellatrixBlock := shared.SignedBeaconBlockBellatrix{}
		if err := decoder.Decode(&jsonBellatrixBlock); err != nil {
			return nil, errors.Wrap(err, "failed to decode signed bellatrix block response json")
		}

		bellatrixBlock, err := c.beaconBlockConverter.ConvertRESTBellatrixBlockToProto(jsonBellatrixBlock.Message)
		if err != nil {
			return nil, errors.Wrap(err, "failed to get signed bellatrix block")
		}

		decodedSignature, err := hexutil.Decode(jsonBellatrixBlock.Signature)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to decode bellatrix block signature `%s`", jsonBellatrixBlock.Signature)
		}

		response.Block = &ethpb.StreamBlocksResponse_BellatrixBlock{
			BellatrixBlock: &ethpb.SignedBeaconBlockBellatrix{
				Signature: decodedSignature,
				Block:     bellatrixBlock,
			},
		}

		slot = bellatrixBlock.Slot

	case "capella":
		jsonCapellaBlock := shared.SignedBeaconBlockCapella{}
		if err := decoder.Decode(&jsonCapellaBlock); err != nil {
			return nil, errors.Wrap(err, "failed to decode signed capella block response json")
		}

		capellaBlock, err := c.beaconBlockConverter.ConvertRESTCapellaBlockToProto(jsonCapellaBlock.Message)
		if err != nil {
			return nil, errors.Wrap(err, "failed to get signed capella block")
		}

		decodedSignature, err := hexutil.Decode(jsonCapellaBlock.Signature)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to decode capella block signature `%s`", jsonCapellaBlock.Signature)
		}

		response.Block = &ethpb.StreamBlocksResponse_CapellaBlock{
			CapellaBlock: &ethpb.SignedBeaconBlockCapella{
				Signature: decodedSignature,
				Block:     capellaBlock,
			},
		}

		slot = capellaBlock.Slot
	case "deneb":
		jsonDenebBlock := shared.SignedBeaconBlockDeneb{}
		if err := decoder.Decode(&jsonDenebBlock); err != nil {
			return nil, errors.Wrap(err, "failed to decode signed deneb block response json")
		}

		denebBlock, err := jsonDenebBlock.ToConsensus()
		if err != nil {
			return nil, errors.Wrap(err, "failed to get signed deneb block")
		}

		response.Block = &ethpb.StreamBlocksResponse_DenebBlock{
			DenebBlock: &ethpb.SignedBeaconBlockDeneb{
				Signature: denebBlock.Signature,
				Block:     denebBlock.Block,
			},
		}

		slot = denebBlock.Block.Slot

	default:
		return nil, errors.Errorf("unsupported consensus version `%s`", signedBlockResponseJson.Version)
	}

	return &headSignedBeaconBlockResult{
		streamBlocksResponse: response,
		executionOptimistic:  signedBlockResponseJson.ExecutionOptimistic,
		slot:                 slot,
	}, nil
}
