package events

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	time2 "time"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/prysmaticlabs/prysm/v4/api"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/operation"
	statefeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/state"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/eth/v1"
	ethpbv2 "github.com/prysmaticlabs/prysm/v4/proto/eth/v2"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	log "github.com/sirupsen/logrus"
	"go.opencensus.io/trace"
)

const (
	// HeadTopic represents a new chain head event topic.
	HeadTopic = "head"
	// BlockTopic represents a new produced block event topic.
	BlockTopic = "block"
	// AttestationTopic represents a new submitted attestation event topic.
	AttestationTopic = "attestation"
	// VoluntaryExitTopic represents a new performed voluntary exit event topic.
	VoluntaryExitTopic = "voluntary_exit"
	// FinalizedCheckpointTopic represents a new finalized checkpoint event topic.
	FinalizedCheckpointTopic = "finalized_checkpoint"
	// ChainReorgTopic represents a chain reorganization event topic.
	ChainReorgTopic = "chain_reorg"
	// SyncCommitteeContributionTopic represents a new sync committee contribution event topic.
	SyncCommitteeContributionTopic = "contribution_and_proof"
	// BLSToExecutionChangeTopic represents a new received BLS to execution change event topic.
	BLSToExecutionChangeTopic = "bls_to_execution_change"
	// PayloadAttributesTopic represents a new payload attributes for execution payload building event topic.
	PayloadAttributesTopic = "payload_attributes"
	// BlobSidecarTopic represents a new blob sidecar event topic
	BlobSidecarTopic = "blob_sidecar"
	// ProposerSlashingTopic represents a new proposer slashing event topic
	ProposerSlashingTopic = "proposer_slashing"
	// AttesterSlashingTopic represents a new attester slashing event topic
	AttesterSlashingTopic = "attester_slashing"
	// LightClientFinalityUpdateTopic represents a new light client finality update event topic.
	LightClientFinalityUpdateTopic = "light_client_finality_update"
	// LightClientOptimisticUpdateTopic represents a new light client optimistic update event topic.
	LightClientOptimisticUpdateTopic = "light_client_optimistic_update"
)

const topicDataMismatch = "Event data type %T does not correspond to event topic %s"

const chanBuffer = 1000

var casesHandled = map[string]bool{
	HeadTopic:                        true,
	BlockTopic:                       true,
	AttestationTopic:                 true,
	VoluntaryExitTopic:               true,
	FinalizedCheckpointTopic:         true,
	ChainReorgTopic:                  true,
	SyncCommitteeContributionTopic:   true,
	BLSToExecutionChangeTopic:        true,
	PayloadAttributesTopic:           true,
	BlobSidecarTopic:                 true,
	ProposerSlashingTopic:            true,
	AttesterSlashingTopic:            true,
	LightClientFinalityUpdateTopic:   true,
	LightClientOptimisticUpdateTopic: true,
}

// StreamEvents provides an endpoint to subscribe to the beacon node Server-Sent-Events stream.
// Consumers should use the eventsource implementation to listen for those events.
// Servers may send SSE comments beginning with ':' for any purpose,
// including to keep the event stream connection alive in the presence of proxy servers.
func (s *Server) StreamEvents(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "events.StreamEvents")
	defer span.End()

	flusher, ok := w.(http.Flusher)
	if !ok {
		httputil.HandleError(w, "Streaming unsupported!", http.StatusInternalServerError)
		return
	}

	topics := r.URL.Query()["topics"]
	if len(topics) == 0 {
		httputil.HandleError(w, "No topics specified to subscribe to", http.StatusBadRequest)
		return
	}
	topicsMap := make(map[string]bool)
	for _, topic := range topics {
		if _, ok := casesHandled[topic]; !ok {
			httputil.HandleError(w, fmt.Sprintf("Invalid topic: %s", topic), http.StatusBadRequest)
			return
		}
		topicsMap[topic] = true
	}

	// Subscribe to event feeds from information received in the beacon node runtime.
	opsChan := make(chan *feed.Event, chanBuffer)
	opsSub := s.OperationNotifier.OperationFeed().Subscribe(opsChan)
	stateChan := make(chan *feed.Event, chanBuffer)
	stateSub := s.StateNotifier.StateFeed().Subscribe(stateChan)
	defer opsSub.Unsubscribe()
	defer stateSub.Unsubscribe()

	// Set up SSE response headers
	w.Header().Set("Content-Type", api.EventStreamMediaType)
	w.Header().Set("Connection", api.KeepAlive)

	// Handle each event received and context cancellation.
	// We send a keepalive dummy message immediately to prevent clients
	// stalling while waiting for the first response chunk.
	// After that we send a keepalive dummy message every SECONDS_PER_SLOT
	// to prevent anyone (e.g. proxy servers) from closing connections.
	sendKeepalive(w, flusher)
	keepaliveTicker := time2.NewTicker(time2.Duration(params.BeaconConfig().SecondsPerSlot) * time2.Second)
	for {
		select {
		case event := <-opsChan:
			handleBlockOperationEvents(w, flusher, topicsMap, event)
		case event := <-stateChan:
			s.handleStateEvents(ctx, w, flusher, topicsMap, event)
		case <-keepaliveTicker.C:
			sendKeepalive(w, flusher)
		case <-ctx.Done():
			return
		}
	}
}

func handleBlockOperationEvents(w http.ResponseWriter, flusher http.Flusher, requestedTopics map[string]bool, event *feed.Event) {
	switch event.Type {
	case operation.AggregatedAttReceived:
		if _, ok := requestedTopics[AttestationTopic]; !ok {
			return
		}
		attData, ok := event.Data.(*operation.AggregatedAttReceivedData)
		if !ok {
			write(w, flusher, topicDataMismatch, event.Data, AttestationTopic)
			return
		}
		att := shared.AttFromConsensus(attData.Attestation.Aggregate)
		send(w, flusher, AttestationTopic, att)
	case operation.UnaggregatedAttReceived:
		if _, ok := requestedTopics[AttestationTopic]; !ok {
			return
		}
		attData, ok := event.Data.(*operation.UnAggregatedAttReceivedData)
		if !ok {
			write(w, flusher, topicDataMismatch, event.Data, AttestationTopic)
			return
		}
		att := shared.AttFromConsensus(attData.Attestation)
		send(w, flusher, AttestationTopic, att)
	case operation.ExitReceived:
		if _, ok := requestedTopics[VoluntaryExitTopic]; !ok {
			return
		}
		exitData, ok := event.Data.(*operation.ExitReceivedData)
		if !ok {
			write(w, flusher, topicDataMismatch, event.Data, VoluntaryExitTopic)
			return
		}
		exit := shared.SignedExitFromConsensus(exitData.Exit)
		send(w, flusher, VoluntaryExitTopic, exit)
	case operation.SyncCommitteeContributionReceived:
		if _, ok := requestedTopics[SyncCommitteeContributionTopic]; !ok {
			return
		}
		contributionData, ok := event.Data.(*operation.SyncCommitteeContributionReceivedData)
		if !ok {
			write(w, flusher, topicDataMismatch, event.Data, SyncCommitteeContributionTopic)
			return
		}
		contribution := shared.SignedContributionAndProofFromConsensus(contributionData.Contribution)
		send(w, flusher, SyncCommitteeContributionTopic, contribution)
	case operation.BLSToExecutionChangeReceived:
		if _, ok := requestedTopics[BLSToExecutionChangeTopic]; !ok {
			return
		}
		changeData, ok := event.Data.(*operation.BLSToExecutionChangeReceivedData)
		if !ok {
			write(w, flusher, topicDataMismatch, event.Data, BLSToExecutionChangeTopic)
			return
		}
		send(w, flusher, BLSToExecutionChangeTopic, shared.SignedBLSChangeFromConsensus(changeData.Change))
	case operation.BlobSidecarReceived:
		if _, ok := requestedTopics[BlobSidecarTopic]; !ok {
			return
		}
		blobData, ok := event.Data.(*operation.BlobSidecarReceivedData)
		if !ok {
			write(w, flusher, topicDataMismatch, event.Data, BlobSidecarTopic)
			return
		}
		versionedHash := blockchain.ConvertKzgCommitmentToVersionedHash(blobData.Blob.KzgCommitment)
		blobEvent := &BlobSidecarEvent{
			BlockRoot:     hexutil.Encode(blobData.Blob.BlockRootSlice()),
			Index:         fmt.Sprintf("%d", blobData.Blob.Index),
			Slot:          fmt.Sprintf("%d", blobData.Blob.Slot()),
			VersionedHash: versionedHash.String(),
			KzgCommitment: hexutil.Encode(blobData.Blob.KzgCommitment),
		}
		send(w, flusher, BlobSidecarTopic, blobEvent)
	case operation.AttesterSlashingReceived:
		if _, ok := requestedTopics[AttesterSlashingTopic]; !ok {
			return
		}
		attesterSlashingData, ok := event.Data.(*operation.AttesterSlashingReceivedData)
		if !ok {
			write(w, flusher, topicDataMismatch, event.Data, AttesterSlashingTopic)
			return
		}
		send(w, flusher, AttesterSlashingTopic, shared.AttesterSlashingFromConsensus(attesterSlashingData.AttesterSlashing))
	case operation.ProposerSlashingReceived:
		if _, ok := requestedTopics[ProposerSlashingTopic]; !ok {
			return
		}
		proposerSlashingData, ok := event.Data.(*operation.ProposerSlashingReceivedData)
		if !ok {
			write(w, flusher, topicDataMismatch, event.Data, ProposerSlashingTopic)
			return
		}
		send(w, flusher, ProposerSlashingTopic, shared.ProposerSlashingFromConsensus(proposerSlashingData.ProposerSlashing))
	}
}

func (s *Server) handleStateEvents(ctx context.Context, w http.ResponseWriter, flusher http.Flusher, requestedTopics map[string]bool, event *feed.Event) {
	switch event.Type {
	case statefeed.NewHead:
		if _, ok := requestedTopics[HeadTopic]; ok {
			headData, ok := event.Data.(*ethpb.EventHead)
			if !ok {
				write(w, flusher, topicDataMismatch, event.Data, HeadTopic)
				return
			}
			head := &HeadEvent{
				Slot:                      fmt.Sprintf("%d", headData.Slot),
				Block:                     hexutil.Encode(headData.Block),
				State:                     hexutil.Encode(headData.State),
				EpochTransition:           headData.EpochTransition,
				ExecutionOptimistic:       headData.ExecutionOptimistic,
				PreviousDutyDependentRoot: hexutil.Encode(headData.PreviousDutyDependentRoot),
				CurrentDutyDependentRoot:  hexutil.Encode(headData.CurrentDutyDependentRoot),
			}
			send(w, flusher, HeadTopic, head)
		}
		if _, ok := requestedTopics[PayloadAttributesTopic]; ok {
			s.sendPayloadAttributes(ctx, w, flusher)
		}
	case statefeed.MissedSlot:
		if _, ok := requestedTopics[PayloadAttributesTopic]; ok {
			s.sendPayloadAttributes(ctx, w, flusher)
		}
	case statefeed.FinalizedCheckpoint:
		if _, ok := requestedTopics[FinalizedCheckpointTopic]; !ok {
			return
		}
		checkpointData, ok := event.Data.(*ethpb.EventFinalizedCheckpoint)
		if !ok {
			write(w, flusher, topicDataMismatch, event.Data, FinalizedCheckpointTopic)
			return
		}
		checkpoint := &FinalizedCheckpointEvent{
			Block:               hexutil.Encode(checkpointData.Block),
			State:               hexutil.Encode(checkpointData.State),
			Epoch:               fmt.Sprintf("%d", checkpointData.Epoch),
			ExecutionOptimistic: checkpointData.ExecutionOptimistic,
		}
		send(w, flusher, FinalizedCheckpointTopic, checkpoint)
	case statefeed.LightClientFinalityUpdate:
		if _, ok := requestedTopics[LightClientFinalityUpdateTopic]; !ok {
			return
		}
		updateData, ok := event.Data.(*ethpbv2.LightClientFinalityUpdateWithVersion)
		if !ok {
			write(w, flusher, topicDataMismatch, event.Data, LightClientFinalityUpdateTopic)
			return
		}

		var finalityBranch []string
		for _, b := range updateData.Data.FinalityBranch {
			finalityBranch = append(finalityBranch, hexutil.Encode(b))
		}
		update := &LightClientFinalityUpdateEvent{
			Version: version.String(int(updateData.Version)),
			Data: &LightClientFinalityUpdate{
				AttestedHeader: &shared.BeaconBlockHeader{
					Slot:          fmt.Sprintf("%d", updateData.Data.AttestedHeader.Slot),
					ProposerIndex: fmt.Sprintf("%d", updateData.Data.AttestedHeader.ProposerIndex),
					ParentRoot:    hexutil.Encode(updateData.Data.AttestedHeader.ParentRoot),
					StateRoot:     hexutil.Encode(updateData.Data.AttestedHeader.StateRoot),
					BodyRoot:      hexutil.Encode(updateData.Data.AttestedHeader.BodyRoot),
				},
				FinalizedHeader: &shared.BeaconBlockHeader{
					Slot:          fmt.Sprintf("%d", updateData.Data.FinalizedHeader.Slot),
					ProposerIndex: fmt.Sprintf("%d", updateData.Data.FinalizedHeader.ProposerIndex),
					ParentRoot:    hexutil.Encode(updateData.Data.FinalizedHeader.ParentRoot),
					StateRoot:     hexutil.Encode(updateData.Data.FinalizedHeader.StateRoot),
				},
				FinalityBranch: finalityBranch,
				SyncAggregate: &shared.SyncAggregate{
					SyncCommitteeBits:      hexutil.Encode(updateData.Data.SyncAggregate.SyncCommitteeBits),
					SyncCommitteeSignature: hexutil.Encode(updateData.Data.SyncAggregate.SyncCommitteeSignature),
				},
				SignatureSlot: fmt.Sprintf("%d", updateData.Data.SignatureSlot),
			},
		}
		send(w, flusher, LightClientFinalityUpdateTopic, update)
	case statefeed.LightClientOptimisticUpdate:
		if _, ok := requestedTopics[LightClientOptimisticUpdateTopic]; !ok {
			return
		}
		updateData, ok := event.Data.(*ethpbv2.LightClientOptimisticUpdateWithVersion)
		if !ok {
			write(w, flusher, topicDataMismatch, event.Data, LightClientOptimisticUpdateTopic)
			return
		}
		update := &LightClientOptimisticUpdateEvent{
			Version: version.String(int(updateData.Version)),
			Data: &LightClientOptimisticUpdate{
				AttestedHeader: &shared.BeaconBlockHeader{
					Slot:          fmt.Sprintf("%d", updateData.Data.AttestedHeader.Slot),
					ProposerIndex: fmt.Sprintf("%d", updateData.Data.AttestedHeader.ProposerIndex),
					ParentRoot:    hexutil.Encode(updateData.Data.AttestedHeader.ParentRoot),
					StateRoot:     hexutil.Encode(updateData.Data.AttestedHeader.StateRoot),
					BodyRoot:      hexutil.Encode(updateData.Data.AttestedHeader.BodyRoot),
				},
				SyncAggregate: &shared.SyncAggregate{
					SyncCommitteeBits:      hexutil.Encode(updateData.Data.SyncAggregate.SyncCommitteeBits),
					SyncCommitteeSignature: hexutil.Encode(updateData.Data.SyncAggregate.SyncCommitteeSignature),
				},
				SignatureSlot: fmt.Sprintf("%d", updateData.Data.SignatureSlot),
			},
		}
		send(w, flusher, LightClientOptimisticUpdateTopic, update)
	case statefeed.Reorg:
		if _, ok := requestedTopics[ChainReorgTopic]; !ok {
			return
		}
		reorgData, ok := event.Data.(*ethpb.EventChainReorg)
		if !ok {
			write(w, flusher, topicDataMismatch, event.Data, ChainReorgTopic)
			return
		}
		reorg := &ChainReorgEvent{
			Slot:                fmt.Sprintf("%d", reorgData.Slot),
			Depth:               fmt.Sprintf("%d", reorgData.Depth),
			OldHeadBlock:        hexutil.Encode(reorgData.OldHeadBlock),
			NewHeadBlock:        hexutil.Encode(reorgData.NewHeadBlock),
			OldHeadState:        hexutil.Encode(reorgData.OldHeadState),
			NewHeadState:        hexutil.Encode(reorgData.NewHeadState),
			Epoch:               fmt.Sprintf("%d", reorgData.Epoch),
			ExecutionOptimistic: reorgData.ExecutionOptimistic,
		}
		send(w, flusher, ChainReorgTopic, reorg)
	case statefeed.BlockProcessed:
		if _, ok := requestedTopics[BlockTopic]; !ok {
			return
		}
		blkData, ok := event.Data.(*statefeed.BlockProcessedData)
		if !ok {
			write(w, flusher, topicDataMismatch, event.Data, BlockTopic)
			return
		}
		blockRoot, err := blkData.SignedBlock.Block().HashTreeRoot()
		if err != nil {
			write(w, flusher, "Could not get block root: "+err.Error())
			return
		}
		blk := &BlockEvent{
			Slot:                fmt.Sprintf("%d", blkData.Slot),
			Block:               hexutil.Encode(blockRoot[:]),
			ExecutionOptimistic: blkData.Optimistic,
		}
		send(w, flusher, BlockTopic, blk)
	}
}

// This event stream is intended to be used by builders and relays.
// Parent fields are based on state at N_{current_slot}, while the rest of fields are based on state of N_{current_slot + 1}
func (s *Server) sendPayloadAttributes(ctx context.Context, w http.ResponseWriter, flusher http.Flusher) {
	headRoot, err := s.HeadFetcher.HeadRoot(ctx)
	if err != nil {
		write(w, flusher, "Could not get head root: "+err.Error())
		return
	}
	st, err := s.HeadFetcher.HeadState(ctx)
	if err != nil {
		write(w, flusher, "Could not get head state: "+err.Error())
		return
	}
	// advance the head state
	headState, err := transition.ProcessSlotsIfPossible(ctx, st, s.ChainInfoFetcher.CurrentSlot()+1)
	if err != nil {
		write(w, flusher, "Could not advance head state: "+err.Error())
		return
	}

	headBlock, err := s.HeadFetcher.HeadBlock(ctx)
	if err != nil {
		write(w, flusher, "Could not get head block: "+err.Error())
		return
	}

	headPayload, err := headBlock.Block().Body().Execution()
	if err != nil {
		write(w, flusher, "Could not get execution payload: "+err.Error())
		return
	}

	t, err := slots.ToTime(headState.GenesisTime(), headState.Slot())
	if err != nil {
		write(w, flusher, "Could not get head state slot time: "+err.Error())
		return
	}

	prevRando, err := helpers.RandaoMix(headState, time.CurrentEpoch(headState))
	if err != nil {
		write(w, flusher, "Could not get head state randao mix: "+err.Error())
		return
	}

	proposerIndex, err := helpers.BeaconProposerIndex(ctx, headState)
	if err != nil {
		write(w, flusher, "Could not get head state proposer index: "+err.Error())
		return
	}

	var attributes interface{}
	switch headState.Version() {
	case version.Bellatrix:
		attributes = &PayloadAttributesV1{
			Timestamp:             fmt.Sprintf("%d", t.Unix()),
			PrevRandao:            hexutil.Encode(prevRando),
			SuggestedFeeRecipient: hexutil.Encode(headPayload.FeeRecipient()),
		}
	case version.Capella:
		withdrawals, err := headState.ExpectedWithdrawals()
		if err != nil {
			write(w, flusher, "Could not get head state expected withdrawals: "+err.Error())
			return
		}
		attributes = &PayloadAttributesV2{
			Timestamp:             fmt.Sprintf("%d", t.Unix()),
			PrevRandao:            hexutil.Encode(prevRando),
			SuggestedFeeRecipient: hexutil.Encode(headPayload.FeeRecipient()),
			Withdrawals:           shared.WithdrawalsFromConsensus(withdrawals),
		}
	case version.Deneb:
		withdrawals, err := headState.ExpectedWithdrawals()
		if err != nil {
			write(w, flusher, "Could not get head state expected withdrawals: "+err.Error())
			return
		}
		parentRoot, err := headBlock.Block().HashTreeRoot()
		if err != nil {
			write(w, flusher, "Could not get head block root: "+err.Error())
			return
		}
		attributes = &PayloadAttributesV3{
			Timestamp:             fmt.Sprintf("%d", t.Unix()),
			PrevRandao:            hexutil.Encode(prevRando),
			SuggestedFeeRecipient: hexutil.Encode(headPayload.FeeRecipient()),
			Withdrawals:           shared.WithdrawalsFromConsensus(withdrawals),
			ParentBeaconBlockRoot: hexutil.Encode(parentRoot[:]),
		}
	default:
		write(w, flusher, "Payload version %s is not supported", version.String(headState.Version()))
		return
	}

	attributesBytes, err := json.Marshal(attributes)
	if err != nil {
		write(w, flusher, err.Error())
		return
	}
	eventData := PayloadAttributesEventData{
		ProposerIndex:     fmt.Sprintf("%d", proposerIndex),
		ProposalSlot:      fmt.Sprintf("%d", headState.Slot()),
		ParentBlockNumber: fmt.Sprintf("%d", headPayload.BlockNumber()),
		ParentBlockRoot:   hexutil.Encode(headRoot),
		ParentBlockHash:   hexutil.Encode(headPayload.BlockHash()),
		PayloadAttributes: attributesBytes,
	}
	eventDataBytes, err := json.Marshal(eventData)
	if err != nil {
		write(w, flusher, err.Error())
		return
	}
	send(w, flusher, PayloadAttributesTopic, &PayloadAttributesEvent{
		Version: version.String(headState.Version()),
		Data:    eventDataBytes,
	})
}

func send(w http.ResponseWriter, flusher http.Flusher, name string, data interface{}) {
	j, err := json.Marshal(data)
	if err != nil {
		write(w, flusher, "Could not marshal event to JSON: "+err.Error())
		return
	}
	write(w, flusher, "event: %s\ndata: %s\n\n", name, string(j))
}

func sendKeepalive(w http.ResponseWriter, flusher http.Flusher) {
	write(w, flusher, ":\n\n")
}

func write(w http.ResponseWriter, flusher http.Flusher, format string, a ...any) {
	_, err := fmt.Fprintf(w, format, a...)
	if err != nil {
		log.WithError(err).Error("Could not write to response writer")
	}
	flusher.Flush()
}
