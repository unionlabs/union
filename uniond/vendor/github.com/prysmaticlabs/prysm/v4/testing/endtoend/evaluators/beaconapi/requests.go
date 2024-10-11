package beaconapi

import (
	"fmt"
	"strings"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/beacon"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/config"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/debug"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/node"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/validator"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
)

var requests = map[string]endpoint{
	"/beacon/genesis": newMetadata[beacon.GetGenesisResponse](v1PathTemplate),
	"/beacon/states/{param1}/root": newMetadata[beacon.GetStateRootResponse](v1PathTemplate,
		withParams(func(_ primitives.Epoch) []string {
			return []string{"head"}
		})),
	"/beacon/states/{param1}/fork": newMetadata[beacon.GetStateForkResponse](v1PathTemplate,
		withParams(func(_ primitives.Epoch) []string {
			return []string{"head"}
		})),
	"/beacon/states/{param1}/finality_checkpoints": newMetadata[beacon.GetFinalityCheckpointsResponse](v1PathTemplate,
		withParams(func(_ primitives.Epoch) []string {
			return []string{"head"}
		})),
	// we want to test comma-separated query params
	"/beacon/states/{param1}/validators?id=0,1": newMetadata[beacon.GetValidatorsResponse](v1PathTemplate,
		withParams(func(_ primitives.Epoch) []string {
			return []string{"head"}
		})),
	"/beacon/states/{param1}/validators/{param2}": newMetadata[beacon.GetValidatorResponse](v1PathTemplate,
		withParams(func(_ primitives.Epoch) []string {
			return []string{"head", "0"}
		})),
	"/beacon/states/{param1}/validator_balances?id=0,1": newMetadata[beacon.GetValidatorBalancesResponse](v1PathTemplate,
		withParams(func(_ primitives.Epoch) []string {
			return []string{"head"}
		})),
	"/beacon/states/{param1}/committees?index=0": newMetadata[beacon.GetCommitteesResponse](v1PathTemplate,
		withParams(func(_ primitives.Epoch) []string {
			return []string{"head"}
		})),
	"/beacon/states/{param1}/sync_committees": newMetadata[beacon.GetSyncCommitteeResponse](v1PathTemplate,
		withStart(params.BeaconConfig().AltairForkEpoch),
		withParams(func(_ primitives.Epoch) []string {
			return []string{"head"}
		})),
	"/beacon/states/{param1}/randao": newMetadata[beacon.GetRandaoResponse](v1PathTemplate,
		withParams(func(_ primitives.Epoch) []string {
			return []string{"head"}
		})),
	"/beacon/headers": newMetadata[beacon.GetBlockHeadersResponse](v1PathTemplate),
	"/beacon/headers/{param1}": newMetadata[beacon.GetBlockHeaderResponse](v1PathTemplate,
		withParams(func(e primitives.Epoch) []string {
			slot := uint64(0)
			if e > 0 {
				slot = (uint64(e) * uint64(params.BeaconConfig().SlotsPerEpoch)) - 1
			}
			return []string{fmt.Sprintf("%v", slot)}
		})),
	"/beacon/blocks/{param1}": newMetadata[beacon.GetBlockV2Response](v2PathTemplate,
		withSsz(),
		withParams(func(_ primitives.Epoch) []string {
			return []string{"head"}
		})),
	"/beacon/blocks/{param1}/root": newMetadata[beacon.BlockRootResponse](v1PathTemplate,
		withParams(func(_ primitives.Epoch) []string {
			return []string{"head"}
		})),
	"/beacon/blocks/{param1}/attestations": newMetadata[beacon.GetBlockAttestationsResponse](v1PathTemplate,
		withParams(func(_ primitives.Epoch) []string {
			return []string{"head"}
		})),
	"/beacon/blinded_blocks/{param1}": newMetadata[beacon.GetBlockV2Response](v1PathTemplate,
		withSsz(),
		withParams(func(_ primitives.Epoch) []string {
			return []string{"head"}
		})),
	"/beacon/pool/attestations": newMetadata[beacon.ListAttestationsResponse](v1PathTemplate,
		withCustomEval(func(p interface{}, _ interface{}) error {
			pResp, ok := p.(*beacon.ListAttestationsResponse)
			if !ok {
				return fmt.Errorf(msgWrongJson, &beacon.ListAttestationsResponse{}, p)
			}
			if pResp.Data == nil {
				return errEmptyPrysmData
			}
			return nil
		})),
	"/beacon/pool/attester_slashings": newMetadata[beacon.GetAttesterSlashingsResponse](v1PathTemplate,
		withCustomEval(func(p interface{}, _ interface{}) error {
			pResp, ok := p.(*beacon.GetAttesterSlashingsResponse)
			if !ok {
				return fmt.Errorf(msgWrongJson, &beacon.GetAttesterSlashingsResponse{}, p)
			}
			if pResp.Data == nil {
				return errEmptyPrysmData
			}
			return nil
		})),
	"/beacon/pool/proposer_slashings": newMetadata[beacon.GetProposerSlashingsResponse](v1PathTemplate,
		withCustomEval(func(p interface{}, _ interface{}) error {
			pResp, ok := p.(*beacon.GetProposerSlashingsResponse)
			if !ok {
				return fmt.Errorf(msgWrongJson, &beacon.GetProposerSlashingsResponse{}, p)
			}
			if pResp.Data == nil {
				return errEmptyPrysmData
			}
			return nil
		})),
	"/beacon/pool/voluntary_exits": newMetadata[beacon.ListVoluntaryExitsResponse](v1PathTemplate,
		withCustomEval(func(p interface{}, _ interface{}) error {
			pResp, ok := p.(*beacon.ListVoluntaryExitsResponse)
			if !ok {
				return fmt.Errorf(msgWrongJson, &beacon.ListVoluntaryExitsResponse{}, p)
			}
			if pResp.Data == nil {
				return errEmptyPrysmData
			}
			return nil
		})),
	"/beacon/pool/bls_to_execution_changes": newMetadata[beacon.BLSToExecutionChangesPoolResponse](v1PathTemplate,
		withCustomEval(func(p interface{}, _ interface{}) error {
			pResp, ok := p.(*beacon.BLSToExecutionChangesPoolResponse)
			if !ok {
				return fmt.Errorf(msgWrongJson, &beacon.BLSToExecutionChangesPoolResponse{}, p)
			}
			if pResp.Data == nil {
				return errEmptyPrysmData
			}
			return nil
		})),
	"/config/fork_schedule": newMetadata[config.GetForkScheduleResponse](v1PathTemplate,
		withCustomEval(func(p interface{}, lh interface{}) error {
			pResp, ok := p.(*config.GetForkScheduleResponse)
			if !ok {
				return fmt.Errorf(msgWrongJson, &config.GetForkScheduleResponse{}, p)
			}
			lhResp, ok := lh.(*config.GetForkScheduleResponse)
			if !ok {
				return fmt.Errorf(msgWrongJson, &config.GetForkScheduleResponse{}, lh)
			}
			if pResp.Data == nil {
				return errEmptyPrysmData
			}
			if lhResp.Data == nil {
				return errEmptyLighthouseData
			}
			// remove all forks with far-future epoch
			for i := len(pResp.Data) - 1; i >= 0; i-- {
				if pResp.Data[i].Epoch == fmt.Sprintf("%d", params.BeaconConfig().FarFutureEpoch) {
					pResp.Data = append(pResp.Data[:i], pResp.Data[i+1:]...)
				}
			}
			for i := len(lhResp.Data) - 1; i >= 0; i-- {
				if lhResp.Data[i].Epoch == fmt.Sprintf("%d", params.BeaconConfig().FarFutureEpoch) {
					lhResp.Data = append(lhResp.Data[:i], lhResp.Data[i+1:]...)
				}
			}
			return compareJSON(pResp, lhResp)
		})),
	"/config/deposit_contract": newMetadata[config.GetDepositContractResponse](v1PathTemplate),
	"/debug/beacon/heads":      newMetadata[debug.GetForkChoiceHeadsV2Response](v2PathTemplate),
	"/node/identity": newMetadata[node.GetIdentityResponse](v1PathTemplate,
		withCustomEval(func(p interface{}, _ interface{}) error {
			pResp, ok := p.(*node.GetIdentityResponse)
			if !ok {
				return fmt.Errorf(msgWrongJson, &node.GetIdentityResponse{}, p)
			}
			if pResp.Data == nil {
				return errEmptyPrysmData
			}
			return nil
		})),
	"/node/peers": newMetadata[node.GetPeersResponse](v1PathTemplate,
		withCustomEval(func(p interface{}, _ interface{}) error {
			pResp, ok := p.(*node.GetPeersResponse)
			if !ok {
				return fmt.Errorf(msgWrongJson, &node.GetPeersResponse{}, p)
			}
			if pResp.Data == nil {
				return errEmptyPrysmData
			}
			return nil
		})),
	"/node/peer_count": newMetadata[node.GetPeerCountResponse](v1PathTemplate,
		withCustomEval(func(p interface{}, _ interface{}) error {
			pResp, ok := p.(*node.GetPeerCountResponse)
			if !ok {
				return fmt.Errorf(msgWrongJson, &node.GetPeerCountResponse{}, p)
			}
			if pResp.Data == nil {
				return errEmptyPrysmData
			}
			return nil
		})),
	"/node/version": newMetadata[node.GetVersionResponse](v1PathTemplate,
		withCustomEval(func(p interface{}, lh interface{}) error {
			pResp, ok := p.(*node.GetVersionResponse)
			if !ok {
				return fmt.Errorf(msgWrongJson, &beacon.ListAttestationsResponse{}, p)
			}
			lhResp, ok := lh.(*node.GetVersionResponse)
			if !ok {
				return fmt.Errorf(msgWrongJson, &beacon.ListAttestationsResponse{}, p)
			}
			if pResp.Data == nil {
				return errEmptyPrysmData
			}
			if !strings.Contains(pResp.Data.Version, "Prysm") {
				return errors.New("version response does not contain Prysm client name")
			}
			if lhResp.Data == nil {
				return errEmptyLighthouseData
			}
			if !strings.Contains(lhResp.Data.Version, "Lighthouse") {
				return errors.New("version response does not contain Lighthouse client name")
			}
			return nil
		})),
	"/node/syncing": newMetadata[node.SyncStatusResponse](v1PathTemplate),
	"/validator/duties/proposer/{param1}": newMetadata[validator.GetProposerDutiesResponse](v1PathTemplate,
		withParams(func(e primitives.Epoch) []string {
			return []string{fmt.Sprintf("%v", e)}
		}),
		withCustomEval(func(p interface{}, lh interface{}) error {
			pResp, ok := p.(*validator.GetProposerDutiesResponse)
			if !ok {
				return fmt.Errorf(msgWrongJson, &validator.GetProposerDutiesResponse{}, p)
			}
			lhResp, ok := lh.(*validator.GetProposerDutiesResponse)
			if !ok {
				return fmt.Errorf(msgWrongJson, &validator.GetProposerDutiesResponse{}, lh)
			}
			if pResp.Data == nil {
				return errEmptyPrysmData
			}
			if lhResp.Data == nil {
				return errEmptyLighthouseData
			}
			if lhResp.Data[0].Slot == "0" {
				// remove the first item from lighthouse data since lighthouse is returning a value despite no proposer
				// there is no proposer on slot 0 so prysm don't return anything for slot 0
				lhResp.Data = lhResp.Data[1:]
			}
			return compareJSON(pResp, lhResp)
		})),
	"/validator/duties/attester/{param1}": newMetadata[validator.GetAttesterDutiesResponse](v1PathTemplate,
		withParams(func(e primitives.Epoch) []string {
			//ask for a future epoch to test this case
			return []string{fmt.Sprintf("%v", e+1)}
		}),
		withReq(func() []string {
			validatorIndices := make([]string, 64)
			for key := range validatorIndices {
				validatorIndices[key] = fmt.Sprintf("%d", key)
			}
			return validatorIndices
		}()),
		withCustomEval(func(p interface{}, lh interface{}) error {
			pResp, ok := p.(*validator.GetAttesterDutiesResponse)
			if !ok {
				return fmt.Errorf(msgWrongJson, &validator.GetAttesterDutiesResponse{}, p)
			}
			lhResp, ok := lh.(*validator.GetAttesterDutiesResponse)
			if !ok {
				return fmt.Errorf(msgWrongJson, &validator.GetAttesterDutiesResponse{}, lh)
			}
			if pResp.Data == nil {
				return errEmptyPrysmData
			}
			if lhResp.Data == nil {
				return errEmptyLighthouseData
			}
			return compareJSON(pResp, lhResp)
		})),
}
