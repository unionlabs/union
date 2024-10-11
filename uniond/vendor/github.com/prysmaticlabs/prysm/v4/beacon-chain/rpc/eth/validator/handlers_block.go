package validator

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"strings"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/api"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/rewards"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"go.opencensus.io/trace"
	"google.golang.org/protobuf/types/known/wrapperspb"
)

type blockType uint8

const (
	any blockType = iota
	full
	blinded
)

// DEPRECATED: Please use ProduceBlockV3 instead.
//
// ProduceBlockV2 requests the beacon node to produce a valid unsigned beacon block,
// which can then be signed by a proposer and submitted.
func (s *Server) ProduceBlockV2(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "validator.ProduceBlockV2")
	defer span.End()

	if shared.IsSyncing(ctx, w, s.SyncChecker, s.HeadFetcher, s.TimeFetcher, s.OptimisticModeFetcher) {
		return
	}

	segments := strings.Split(r.URL.Path, "/")
	rawSlot := segments[len(segments)-1]
	rawRandaoReveal := r.URL.Query().Get("randao_reveal")
	rawGraffiti := r.URL.Query().Get("graffiti")
	rawSkipRandaoVerification := r.URL.Query().Get("skip_randao_verification")

	slot, valid := shared.ValidateUint(w, "slot", rawSlot)
	if !valid {
		return
	}

	var randaoReveal []byte
	if rawSkipRandaoVerification == "true" {
		randaoReveal = primitives.PointAtInfinity
	} else {
		rr, err := bytesutil.DecodeHexWithLength(rawRandaoReveal, fieldparams.BLSSignatureLength)
		if err != nil {
			httputil.HandleError(w, errors.Wrap(err, "Unable to decode randao reveal").Error(), http.StatusBadRequest)
			return
		}
		randaoReveal = rr
	}
	var graffiti []byte
	if rawGraffiti != "" {
		g, err := bytesutil.DecodeHexWithLength(rawGraffiti, 32)
		if err != nil {
			httputil.HandleError(w, errors.Wrap(err, "Unable to decode graffiti").Error(), http.StatusBadRequest)
			return
		}
		graffiti = g
	}

	s.produceBlockV3(ctx, w, r, &eth.BlockRequest{
		Slot:         primitives.Slot(slot),
		RandaoReveal: randaoReveal,
		Graffiti:     graffiti,
		SkipMevBoost: true,
	}, full)
}

// DEPRECATED: Please use ProduceBlockV3 instead.
//
// ProduceBlindedBlock requests the beacon node to produce a valid unsigned blinded beacon block,
// which can then be signed by a proposer and submitted.
func (s *Server) ProduceBlindedBlock(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "validator.ProduceBlindedBlock")
	defer span.End()

	if shared.IsSyncing(ctx, w, s.SyncChecker, s.HeadFetcher, s.TimeFetcher, s.OptimisticModeFetcher) {
		return
	}

	segments := strings.Split(r.URL.Path, "/")
	rawSlot := segments[len(segments)-1]
	rawRandaoReveal := r.URL.Query().Get("randao_reveal")
	rawGraffiti := r.URL.Query().Get("graffiti")
	rawSkipRandaoVerification := r.URL.Query().Get("skip_randao_verification")

	slot, valid := shared.ValidateUint(w, "slot", rawSlot)
	if !valid {
		return
	}

	var randaoReveal []byte
	if rawSkipRandaoVerification == "true" {
		randaoReveal = primitives.PointAtInfinity
	} else {
		rr, err := bytesutil.DecodeHexWithLength(rawRandaoReveal, fieldparams.BLSSignatureLength)
		if err != nil {
			httputil.HandleError(w, errors.Wrap(err, "Unable to decode randao reveal").Error(), http.StatusBadRequest)
			return
		}
		randaoReveal = rr
	}
	var graffiti []byte
	if rawGraffiti != "" {
		g, err := bytesutil.DecodeHexWithLength(rawGraffiti, 32)
		if err != nil {
			httputil.HandleError(w, errors.Wrap(err, "Unable to decode graffiti").Error(), http.StatusBadRequest)
			return
		}
		graffiti = g
	}

	s.produceBlockV3(ctx, w, r, &eth.BlockRequest{
		Slot:         primitives.Slot(slot),
		RandaoReveal: randaoReveal,
		Graffiti:     graffiti,
		SkipMevBoost: false,
	}, blinded)
}

// ProduceBlockV3 requests a beacon node to produce a valid block, which can then be signed by a validator. The
// returned block may be blinded or unblinded, depending on the current state of the network as
// decided by the execution and beacon nodes.
// The beacon node must return an unblinded block if it obtains the execution payload from its
// paired execution node. It must only return a blinded block if it obtains the execution payload
// header from an MEV relay.
// Metadata in the response indicates the type of block produced, and the supported types of block
// will be added to as forks progress.
func (s *Server) ProduceBlockV3(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "validator.ProduceBlockV3")
	defer span.End()

	if shared.IsSyncing(r.Context(), w, s.SyncChecker, s.HeadFetcher, s.TimeFetcher, s.OptimisticModeFetcher) {
		return
	}

	segments := strings.Split(r.URL.Path, "/")
	rawSlot := segments[len(segments)-1]
	rawRandaoReveal := r.URL.Query().Get("randao_reveal")
	rawGraffiti := r.URL.Query().Get("graffiti")
	rawSkipRandaoVerification := r.URL.Query().Get("skip_randao_verification")

	var bbFactor *wrapperspb.UInt64Value // default the factor via fall back
	rawBbFactor, bbValue, ok := shared.UintFromQuery(w, r, "builder_boost_factor", false)
	if !ok {
		return
	}
	if rawBbFactor != "" {
		bbFactor = &wrapperspb.UInt64Value{Value: bbValue}
	}

	slot, valid := shared.ValidateUint(w, "slot", rawSlot)
	if !valid {
		return
	}

	var randaoReveal []byte
	if rawSkipRandaoVerification == "true" {
		randaoReveal = primitives.PointAtInfinity
	} else {
		rr, err := bytesutil.DecodeHexWithLength(rawRandaoReveal, fieldparams.BLSSignatureLength)
		if err != nil {
			httputil.HandleError(w, errors.Wrap(err, "Unable to decode randao reveal").Error(), http.StatusBadRequest)
			return
		}
		randaoReveal = rr
	}
	var graffiti []byte
	if rawGraffiti != "" {
		g, err := bytesutil.DecodeHexWithLength(rawGraffiti, 32)
		if err != nil {
			httputil.HandleError(w, errors.Wrap(err, "Unable to decode graffiti").Error(), http.StatusBadRequest)
			return
		}
		graffiti = g
	}

	s.produceBlockV3(ctx, w, r, &eth.BlockRequest{
		Slot:               primitives.Slot(slot),
		RandaoReveal:       randaoReveal,
		Graffiti:           graffiti,
		SkipMevBoost:       false,
		BuilderBoostFactor: bbFactor,
	}, any)
}

func (s *Server) produceBlockV3(ctx context.Context, w http.ResponseWriter, r *http.Request, v1alpha1req *eth.BlockRequest, requiredType blockType) {
	isSSZ := httputil.RespondWithSsz(r)
	v1alpha1resp, err := s.V1Alpha1Server.GetBeaconBlock(ctx, v1alpha1req)
	if err != nil {
		httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
		return
	}

	if requiredType == blinded && !v1alpha1resp.IsBlinded {
		httputil.HandleError(w, "Prepared block is not blinded", http.StatusInternalServerError)
		return
	} else if requiredType == full && v1alpha1resp.IsBlinded {
		httputil.HandleError(w, "Prepared block is blinded", http.StatusInternalServerError)
		return
	}

	consensusBlockValue, httpError := getConsensusBlockValue(ctx, s.BlockRewardFetcher, v1alpha1resp.Block)
	if httpError != nil {
		httputil.WriteError(w, httpError)
		return
	}

	w.Header().Set(api.ExecutionPayloadBlindedHeader, fmt.Sprintf("%v", v1alpha1resp.IsBlinded))
	w.Header().Set(api.ExecutionPayloadValueHeader, v1alpha1resp.PayloadValue)
	w.Header().Set(api.ConsensusBlockValueHeader, consensusBlockValue)

	phase0Block, ok := v1alpha1resp.Block.(*eth.GenericBeaconBlock_Phase0)
	if ok {
		w.Header().Set(api.VersionHeader, version.String(version.Phase0))
		// rewards aren't used in phase 0
		handleProducePhase0V3(w, isSSZ, phase0Block, v1alpha1resp.PayloadValue)
		return
	}
	altairBlock, ok := v1alpha1resp.Block.(*eth.GenericBeaconBlock_Altair)
	if ok {
		w.Header().Set(api.VersionHeader, version.String(version.Altair))
		handleProduceAltairV3(w, isSSZ, altairBlock, v1alpha1resp.PayloadValue, consensusBlockValue)
		return
	}
	optimistic, err := s.OptimisticModeFetcher.IsOptimistic(ctx)
	if err != nil {
		httputil.HandleError(w, errors.Wrap(err, "Could not determine if the node is a optimistic node").Error(), http.StatusInternalServerError)
		return
	}
	if optimistic {
		httputil.HandleError(w, "The node is currently optimistic and cannot serve validators", http.StatusServiceUnavailable)
		return
	}
	blindedBellatrixBlock, ok := v1alpha1resp.Block.(*eth.GenericBeaconBlock_BlindedBellatrix)
	if ok {
		w.Header().Set(api.VersionHeader, version.String(version.Bellatrix))
		handleProduceBlindedBellatrixV3(w, isSSZ, blindedBellatrixBlock, v1alpha1resp.PayloadValue, consensusBlockValue)
		return
	}
	bellatrixBlock, ok := v1alpha1resp.Block.(*eth.GenericBeaconBlock_Bellatrix)
	if ok {
		w.Header().Set(api.VersionHeader, version.String(version.Bellatrix))
		handleProduceBellatrixV3(w, isSSZ, bellatrixBlock, v1alpha1resp.PayloadValue, consensusBlockValue)
		return
	}
	blindedCapellaBlock, ok := v1alpha1resp.Block.(*eth.GenericBeaconBlock_BlindedCapella)
	if ok {
		w.Header().Set(api.VersionHeader, version.String(version.Capella))
		handleProduceBlindedCapellaV3(w, isSSZ, blindedCapellaBlock, v1alpha1resp.PayloadValue, consensusBlockValue)
		return
	}
	capellaBlock, ok := v1alpha1resp.Block.(*eth.GenericBeaconBlock_Capella)
	if ok {
		w.Header().Set(api.VersionHeader, version.String(version.Capella))
		handleProduceCapellaV3(w, isSSZ, capellaBlock, v1alpha1resp.PayloadValue, consensusBlockValue)
		return
	}
	blindedDenebBlockContents, ok := v1alpha1resp.Block.(*eth.GenericBeaconBlock_BlindedDeneb)
	if ok {
		w.Header().Set(api.VersionHeader, version.String(version.Deneb))
		handleProduceBlindedDenebV3(w, isSSZ, blindedDenebBlockContents, v1alpha1resp.PayloadValue, consensusBlockValue)
		return
	}
	denebBlockContents, ok := v1alpha1resp.Block.(*eth.GenericBeaconBlock_Deneb)
	if ok {
		w.Header().Set(api.VersionHeader, version.String(version.Deneb))
		handleProduceDenebV3(w, isSSZ, denebBlockContents, v1alpha1resp.PayloadValue, consensusBlockValue)
		return
	}
}

func getConsensusBlockValue(ctx context.Context, blockRewardsFetcher rewards.BlockRewardsFetcher, i interface{} /* block as argument */) (string, *httputil.DefaultJsonError) {
	bb, err := blocks.NewBeaconBlock(i)
	if err != nil {
		return "", &httputil.DefaultJsonError{
			Message: err.Error(),
			Code:    http.StatusInternalServerError,
		}
	}
	if bb.Version() == version.Phase0 {
		// ignore for phase 0
		return "", nil
	}
	//get consensus payload value which is the same as the total from the block rewards api
	blockRewards, httpError := blockRewardsFetcher.GetBlockRewardsData(ctx, bb)
	if httpError != nil {
		return "", httpError
	}
	return blockRewards.Total, nil
}

func handleProducePhase0V3(
	w http.ResponseWriter,
	isSSZ bool,
	blk *eth.GenericBeaconBlock_Phase0,
	payloadValue string,
) {
	if isSSZ {
		sszResp, err := blk.Phase0.MarshalSSZ()
		if err != nil {
			httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
			return
		}
		httputil.WriteSsz(w, sszResp, "phase0Block.ssz")
		return
	}
	jsonBytes, err := json.Marshal(shared.BeaconBlockFromConsensus(blk.Phase0))
	if err != nil {
		httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	httputil.WriteJson(w, &ProduceBlockV3Response{
		Version:                 version.String(version.Phase0),
		ExecutionPayloadBlinded: false,
		ExecutionPayloadValue:   payloadValue, // mev not available at this point
		ConsensusBlockValue:     "",           // rewards not applicable before altair
		Data:                    jsonBytes,
	})
}

func handleProduceAltairV3(
	w http.ResponseWriter,
	isSSZ bool,
	blk *eth.GenericBeaconBlock_Altair,
	executionPayloadValue string,
	consensusPayloadValue string,
) {
	if isSSZ {
		sszResp, err := blk.Altair.MarshalSSZ()
		if err != nil {
			httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
			return
		}
		httputil.WriteSsz(w, sszResp, "altairBlock.ssz")
		return
	}
	jsonBytes, err := json.Marshal(shared.BeaconBlockAltairFromConsensus(blk.Altair))
	if err != nil {
		httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	httputil.WriteJson(w, &ProduceBlockV3Response{
		Version:                 version.String(version.Altair),
		ExecutionPayloadBlinded: false,
		ExecutionPayloadValue:   executionPayloadValue, // mev not available at this point
		ConsensusBlockValue:     consensusPayloadValue,
		Data:                    jsonBytes,
	})
}

func handleProduceBellatrixV3(
	w http.ResponseWriter,
	isSSZ bool,
	blk *eth.GenericBeaconBlock_Bellatrix,
	executionPayloadValue string,
	consensusPayloadValue string,
) {
	if isSSZ {
		sszResp, err := blk.Bellatrix.MarshalSSZ()
		if err != nil {
			httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
			return
		}
		httputil.WriteSsz(w, sszResp, "bellatrixBlock.ssz")
		return
	}
	block, err := shared.BeaconBlockBellatrixFromConsensus(blk.Bellatrix)
	if err != nil {
		httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	jsonBytes, err := json.Marshal(block)
	if err != nil {
		httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	httputil.WriteJson(w, &ProduceBlockV3Response{
		Version:                 version.String(version.Bellatrix),
		ExecutionPayloadBlinded: false,
		ExecutionPayloadValue:   executionPayloadValue, // mev not available at this point
		ConsensusBlockValue:     consensusPayloadValue,
		Data:                    jsonBytes,
	})
}

func handleProduceBlindedBellatrixV3(
	w http.ResponseWriter,
	isSSZ bool,
	blk *eth.GenericBeaconBlock_BlindedBellatrix,
	executionPayloadValue string,
	consensusPayloadValue string,
) {
	if isSSZ {
		sszResp, err := blk.BlindedBellatrix.MarshalSSZ()
		if err != nil {
			httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
			return
		}
		httputil.WriteSsz(w, sszResp, "blindedBellatrixBlock.ssz")
		return
	}
	block, err := shared.BlindedBeaconBlockBellatrixFromConsensus(blk.BlindedBellatrix)
	if err != nil {
		httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	jsonBytes, err := json.Marshal(block)
	if err != nil {
		httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	httputil.WriteJson(w, &ProduceBlockV3Response{
		Version:                 version.String(version.Bellatrix),
		ExecutionPayloadBlinded: true,
		ExecutionPayloadValue:   executionPayloadValue,
		ConsensusBlockValue:     consensusPayloadValue,
		Data:                    jsonBytes,
	})
}

func handleProduceBlindedCapellaV3(
	w http.ResponseWriter,
	isSSZ bool,
	blk *eth.GenericBeaconBlock_BlindedCapella,
	executionPayloadValue string,
	consensusPayloadValue string,
) {
	if isSSZ {
		sszResp, err := blk.BlindedCapella.MarshalSSZ()
		if err != nil {
			httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
			return
		}
		httputil.WriteSsz(w, sszResp, "blindedCapellaBlock.ssz")
		return
	}
	block, err := shared.BlindedBeaconBlockCapellaFromConsensus(blk.BlindedCapella)
	if err != nil {
		httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	jsonBytes, err := json.Marshal(block)
	if err != nil {
		httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	httputil.WriteJson(w, &ProduceBlockV3Response{
		Version:                 version.String(version.Capella),
		ExecutionPayloadBlinded: true,
		ExecutionPayloadValue:   executionPayloadValue,
		ConsensusBlockValue:     consensusPayloadValue,
		Data:                    jsonBytes,
	})
}

func handleProduceCapellaV3(
	w http.ResponseWriter,
	isSSZ bool,
	blk *eth.GenericBeaconBlock_Capella,
	executionPayloadValue string,
	consensusPayloadValue string,
) {
	if isSSZ {
		sszResp, err := blk.Capella.MarshalSSZ()
		if err != nil {
			httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
			return
		}
		httputil.WriteSsz(w, sszResp, "capellaBlock.ssz")
		return
	}
	block, err := shared.BeaconBlockCapellaFromConsensus(blk.Capella)
	if err != nil {
		httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	jsonBytes, err := json.Marshal(block)
	if err != nil {
		httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	httputil.WriteJson(w, &ProduceBlockV3Response{
		Version:                 version.String(version.Capella),
		ExecutionPayloadBlinded: false,
		ExecutionPayloadValue:   executionPayloadValue, // mev not available at this point
		ConsensusBlockValue:     consensusPayloadValue,
		Data:                    jsonBytes,
	})
}

func handleProduceBlindedDenebV3(
	w http.ResponseWriter,
	isSSZ bool,
	blk *eth.GenericBeaconBlock_BlindedDeneb,
	executionPayloadValue string,
	consensusPayloadValue string,
) {
	if isSSZ {
		sszResp, err := blk.BlindedDeneb.MarshalSSZ()
		if err != nil {
			httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
			return
		}
		httputil.WriteSsz(w, sszResp, "blindedDenebBlockContents.ssz")
		return
	}
	blindedBlock, err := shared.BlindedBeaconBlockDenebFromConsensus(blk.BlindedDeneb)
	if err != nil {
		httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	jsonBytes, err := json.Marshal(blindedBlock)
	if err != nil {
		httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	httputil.WriteJson(w, &ProduceBlockV3Response{
		Version:                 version.String(version.Deneb),
		ExecutionPayloadBlinded: true,
		ExecutionPayloadValue:   executionPayloadValue,
		ConsensusBlockValue:     consensusPayloadValue,
		Data:                    jsonBytes,
	})
}

func handleProduceDenebV3(
	w http.ResponseWriter,
	isSSZ bool,
	blk *eth.GenericBeaconBlock_Deneb,
	executionPayloadValue string,
	consensusBlockValue string,
) {
	if isSSZ {
		sszResp, err := blk.Deneb.MarshalSSZ()
		if err != nil {
			httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
			return
		}
		httputil.WriteSsz(w, sszResp, "denebBlockContents.ssz")
		return
	}

	blockContents, err := shared.BeaconBlockContentsDenebFromConsensus(blk.Deneb)
	if err != nil {
		httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	jsonBytes, err := json.Marshal(blockContents)
	if err != nil {
		httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	httputil.WriteJson(w, &ProduceBlockV3Response{
		Version:                 version.String(version.Deneb),
		ExecutionPayloadBlinded: false,
		ExecutionPayloadValue:   executionPayloadValue, // mev not available at this point
		ConsensusBlockValue:     consensusBlockValue,
		Data:                    jsonBytes,
	})
}
