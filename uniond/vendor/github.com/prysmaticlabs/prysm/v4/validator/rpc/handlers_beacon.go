package rpc

import (
	"encoding/base64"
	"fmt"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"go.opencensus.io/trace"
	"google.golang.org/protobuf/types/known/emptypb"
)

// GetBeaconStatus retrieves information about the beacon node gRPC connection
// and certain chain metadata, such as the genesis time, the chain head, and the
// deposit contract address.
func (s *Server) GetBeaconStatus(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "validator.web.beacon.GetBeaconStatus")
	defer span.End()
	syncStatus, err := s.beaconNodeClient.GetSyncStatus(ctx, &emptypb.Empty{})
	if err != nil {
		log.WithError(err).Error("beacon node call to get sync status failed")
		httputil.WriteJson(w, &BeaconStatusResponse{
			BeaconNodeEndpoint: s.nodeGatewayEndpoint,
			Connected:          false,
			Syncing:            false,
		})
		return
	}
	genesis, err := s.beaconNodeClient.GetGenesis(ctx, &emptypb.Empty{})
	if err != nil {
		httputil.HandleError(w, errors.Wrap(err, "GetGenesis call failed").Error(), http.StatusInternalServerError)
		return
	}
	genesisTime := uint64(time.Unix(genesis.GenesisTime.Seconds, 0).Unix())
	address := genesis.DepositContractAddress
	chainHead, err := s.beaconChainClient.GetChainHead(ctx, &emptypb.Empty{})
	if err != nil {
		httputil.HandleError(w, errors.Wrap(err, "GetChainHead").Error(), http.StatusInternalServerError)
		return
	}
	httputil.WriteJson(w, &BeaconStatusResponse{
		BeaconNodeEndpoint:     s.beaconClientEndpoint,
		Connected:              true,
		Syncing:                syncStatus.Syncing,
		GenesisTime:            fmt.Sprintf("%d", genesisTime),
		DepositContractAddress: hexutil.Encode(address),
		ChainHead:              ChainHeadResponseFromConsensus(chainHead),
	})
}

// GetValidatorPerformance is a wrapper around the /eth/v1alpha1 endpoint of the same name.
func (s *Server) GetValidatorPerformance(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "validator.web.beacon.GetValidatorPerformance")
	defer span.End()
	publicKeys := r.URL.Query()["public_keys"]
	pubkeys := make([][]byte, len(publicKeys))
	for i, key := range publicKeys {
		var pk []byte
		if strings.HasPrefix(key, "0x") {
			k, ok := shared.ValidateHex(w, fmt.Sprintf("PublicKeys[%d]", i), key, fieldparams.BLSPubkeyLength)
			if !ok {
				return
			}
			pk = bytesutil.SafeCopyBytes(k)
		} else {
			data, err := base64.StdEncoding.DecodeString(key)
			if err != nil {
				httputil.HandleError(w, errors.Wrap(err, "Failed to decode base64").Error(), http.StatusBadRequest)
				return
			}
			pk = bytesutil.SafeCopyBytes(data)
		}
		pubkeys[i] = pk
	}

	req := &ethpb.ValidatorPerformanceRequest{
		PublicKeys: pubkeys,
	}
	validatorPerformance, err := s.beaconChainClient.GetValidatorPerformance(ctx, req)
	if err != nil {
		httputil.HandleError(w, errors.Wrap(err, "GetValidatorPerformance call failed").Error(), http.StatusInternalServerError)
		return
	}
	httputil.WriteJson(w, ValidatorPerformanceResponseFromConsensus(validatorPerformance))
}

// GetValidatorBalances is a wrapper around the /eth/v1alpha1 endpoint of the same name.
func (s *Server) GetValidatorBalances(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "validator.web.beacon.GetValidatorBalances")
	defer span.End()
	pageSize := r.URL.Query().Get("page_size")
	var ps int64
	if pageSize != "" {
		psi, err := strconv.ParseInt(pageSize, 10, 32)
		if err != nil {
			httputil.HandleError(w, errors.Wrap(err, "Failed to parse page_size").Error(), http.StatusBadRequest)
			return
		}
		ps = psi
	}
	pageToken := r.URL.Query().Get("page_token")
	publicKeys := r.URL.Query()["public_keys"]
	pubkeys := make([][]byte, len(publicKeys))
	for i, key := range publicKeys {
		var pk []byte
		if strings.HasPrefix(key, "0x") {
			k, ok := shared.ValidateHex(w, fmt.Sprintf("PublicKeys[%d]", i), key, fieldparams.BLSPubkeyLength)
			if !ok {
				return
			}
			pk = bytesutil.SafeCopyBytes(k)
		} else {
			data, err := base64.StdEncoding.DecodeString(key)
			if err != nil {
				httputil.HandleError(w, errors.Wrap(err, "Failed to decode base64").Error(), http.StatusBadRequest)
				return
			}
			pk = bytesutil.SafeCopyBytes(data)
		}
		pubkeys[i] = pk
	}
	req := &ethpb.ListValidatorBalancesRequest{
		PublicKeys: pubkeys,
		PageSize:   int32(ps),
		PageToken:  pageToken,
	}
	listValidatorBalances, err := s.beaconChainClient.ListValidatorBalances(ctx, req)
	if err != nil {
		httputil.HandleError(w, errors.Wrap(err, "ListValidatorBalances call failed").Error(), http.StatusInternalServerError)
		return
	}
	response, err := ValidatorBalancesResponseFromConsensus(listValidatorBalances)
	if err != nil {
		httputil.HandleError(w, errors.Wrap(err, "Failed to convert to json").Error(), http.StatusInternalServerError)
		return
	}
	httputil.WriteJson(w, response)
}

// GetValidators is a wrapper around the /eth/v1alpha1 endpoint of the same name.
func (s *Server) GetValidators(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "validator.web.beacon.GetValidators")
	defer span.End()
	pageSize := r.URL.Query().Get("page_size")
	ps, err := strconv.ParseInt(pageSize, 10, 32)
	if err != nil {
		httputil.HandleError(w, errors.Wrap(err, "Failed to parse page_size").Error(), http.StatusBadRequest)
		return
	}
	pageToken := r.URL.Query().Get("page_token")
	publicKeys := r.URL.Query()["public_keys"]
	pubkeys := make([][]byte, len(publicKeys))
	for i, key := range publicKeys {
		var pk []byte
		if strings.HasPrefix(key, "0x") {
			k, ok := shared.ValidateHex(w, fmt.Sprintf("PublicKeys[%d]", i), key, fieldparams.BLSPubkeyLength)
			if !ok {
				return
			}
			pk = bytesutil.SafeCopyBytes(k)
		} else {
			data, err := base64.StdEncoding.DecodeString(key)
			if err != nil {
				httputil.HandleError(w, errors.Wrap(err, "Failed to decode base64").Error(), http.StatusBadRequest)
				return
			}
			pk = bytesutil.SafeCopyBytes(data)
		}
		pubkeys[i] = pk
	}
	req := &ethpb.ListValidatorsRequest{
		PublicKeys: pubkeys,
		PageSize:   int32(ps),
		PageToken:  pageToken,
	}
	validators, err := s.beaconChainClient.ListValidators(ctx, req)
	if err != nil {
		httputil.HandleError(w, errors.Wrap(err, "ListValidators call failed").Error(), http.StatusInternalServerError)
		return
	}
	response, err := ValidatorsResponseFromConsensus(validators)
	if err != nil {
		httputil.HandleError(w, errors.Wrap(err, "Failed to convert to json").Error(), http.StatusInternalServerError)
		return
	}
	httputil.WriteJson(w, response)
}

// GetPeers is a wrapper around the /eth/v1alpha1 endpoint of the same name.
func (s *Server) GetPeers(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "validator.web.beacon.GetPeers")
	defer span.End()
	peers, err := s.beaconNodeClient.ListPeers(ctx, &emptypb.Empty{})
	if err != nil {
		httputil.HandleError(w, errors.Wrap(err, "ListPeers call failed").Error(), http.StatusInternalServerError)
		return
	}
	httputil.WriteJson(w, peers)
}
