package rpc

import (
	"context"
	"errors"
	"io"
	"net/http"
	"testing"
	"time"

	"github.com/gorilla/mux"
	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	mockExecution "github.com/prysmaticlabs/prysm/v4/beacon-chain/execution/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/beacon"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/blob"
	rpcBuilder "github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/builder"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/debug"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/events"
	lightclient "github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/light-client"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/node"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/rewards"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/validator"
	beaconprysm "github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/prysm/beacon"
	nodeprysm "github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/prysm/node"
	validatorprysm "github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/prysm/validator"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	mockSync "github.com/prysmaticlabs/prysm/v4/beacon-chain/sync/initial-sync/testing"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/sirupsen/logrus"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

func init() {
	logrus.SetLevel(logrus.DebugLevel)
	logrus.SetOutput(io.Discard)
}

func combineMaps(maps ...map[string][]string) map[string][]string {
	combinedMap := make(map[string][]string)

	for _, m := range maps {
		for k, v := range m {
			combinedMap[k] = v
		}
	}

	return combinedMap
}

func TestServer_InitializeRoutes(t *testing.T) {
	s := Service{
		cfg: &Config{
			Router: mux.NewRouter(),
		},
	}
	s.initializeRewardServerRoutes(&rewards.Server{})
	s.initializeBuilderServerRoutes(&rpcBuilder.Server{})
	s.initializeBlobServerRoutes(&blob.Server{})
	s.initializeValidatorServerRoutes(&validator.Server{})
	s.initializeNodeServerRoutes(&node.Server{})
	s.initializeBeaconServerRoutes(&beacon.Server{})
	s.initializeConfigRoutes()
	s.initializeEventsServerRoutes(&events.Server{})
	s.initializeLightClientServerRoutes(&lightclient.Server{})
	s.initializeDebugServerRoutes(&debug.Server{})

	//prysm internal
	s.initializePrysmBeaconServerRoutes(&beaconprysm.Server{})
	s.initializePrysmNodeServerRoutes(&nodeprysm.Server{})
	s.initializePrysmValidatorServerRoutes(&validatorprysm.Server{})

	beaconRoutes := map[string][]string{
		"/eth/v1/beacon/genesis":                                     {http.MethodGet},
		"/eth/v1/beacon/states/{state_id}/root":                      {http.MethodGet},
		"/eth/v1/beacon/states/{state_id}/fork":                      {http.MethodGet},
		"/eth/v1/beacon/states/{state_id}/finality_checkpoints":      {http.MethodGet},
		"/eth/v1/beacon/states/{state_id}/validators":                {http.MethodGet, http.MethodPost},
		"/eth/v1/beacon/states/{state_id}/validators/{validator_id}": {http.MethodGet},
		"/eth/v1/beacon/states/{state_id}/validator_balances":        {http.MethodGet, http.MethodPost},
		"/eth/v1/beacon/states/{state_id}/committees":                {http.MethodGet},
		"/eth/v1/beacon/states/{state_id}/sync_committees":           {http.MethodGet},
		"/eth/v1/beacon/states/{state_id}/randao":                    {http.MethodGet},
		"/eth/v1/beacon/headers":                                     {http.MethodGet},
		"/eth/v1/beacon/headers/{block_id}":                          {http.MethodGet},
		"/eth/v1/beacon/blinded_blocks":                              {http.MethodPost},
		"/eth/v2/beacon/blinded_blocks":                              {http.MethodPost},
		"/eth/v1/beacon/blocks":                                      {http.MethodPost},
		"/eth/v2/beacon/blocks":                                      {http.MethodPost},
		"/eth/v1/beacon/blocks/{block_id}":                           {http.MethodGet}, //deprecated
		"/eth/v2/beacon/blocks/{block_id}":                           {http.MethodGet},
		"/eth/v1/beacon/blocks/{block_id}/root":                      {http.MethodGet},
		"/eth/v1/beacon/blocks/{block_id}/attestations":              {http.MethodGet},
		"/eth/v1/beacon/blob_sidecars/{block_id}":                    {http.MethodGet},
		"/eth/v1/beacon/rewards/sync_committee/{block_id}":           {http.MethodPost},
		//"/eth/v1/beacon/deposit_snapshot": {http.MethodGet}, not implemented
		"/eth/v1/beacon/rewards/blocks/{block_id}":           {http.MethodGet},
		"/eth/v1/beacon/rewards/attestations/{epoch}":        {http.MethodPost},
		"/eth/v1/beacon/blinded_blocks/{block_id}":           {http.MethodGet},
		"/eth/v1/beacon/light_client/bootstrap/{block_root}": {http.MethodGet},
		"/eth/v1/beacon/light_client/updates":                {http.MethodGet},
		"/eth/v1/beacon/light_client/finality_update":        {http.MethodGet},
		"/eth/v1/beacon/light_client/optimistic_update":      {http.MethodGet},
		"/eth/v1/beacon/pool/attestations":                   {http.MethodGet, http.MethodPost},
		"/eth/v1/beacon/pool/attester_slashings":             {http.MethodGet, http.MethodPost},
		"/eth/v1/beacon/pool/proposer_slashings":             {http.MethodGet, http.MethodPost},
		"/eth/v1/beacon/pool/sync_committees":                {http.MethodPost},
		"/eth/v1/beacon/pool/voluntary_exits":                {http.MethodGet, http.MethodPost},
		"/eth/v1/beacon/pool/bls_to_execution_changes":       {http.MethodGet, http.MethodPost},
	}

	builderRoutes := map[string][]string{
		"/eth/v1/builder/states/{state_id}/expected_withdrawals": {http.MethodGet},
	}

	configRoutes := map[string][]string{
		"/eth/v1/config/fork_schedule":    {http.MethodGet},
		"/eth/v1/config/spec":             {http.MethodGet},
		"/eth/v1/config/deposit_contract": {http.MethodGet},
	}

	debugRoutes := map[string][]string{
		"/eth/v1/debug/beacon/states/{state_id}": {http.MethodGet}, //deprecated
		"/eth/v2/debug/beacon/states/{state_id}": {http.MethodGet},
		"/eth/v2/debug/beacon/heads":             {http.MethodGet},
		"/eth/v1/debug/fork_choice":              {http.MethodGet},
	}

	eventsRoutes := map[string][]string{
		"/eth/v1/events": {http.MethodGet},
	}

	nodeRoutes := map[string][]string{
		"/eth/v1/node/identity":        {http.MethodGet},
		"/eth/v1/node/peers":           {http.MethodGet},
		"/eth/v1/node/peers/{peer_id}": {http.MethodGet},
		"/eth/v1/node/peer_count":      {http.MethodGet},
		"/eth/v1/node/version":         {http.MethodGet},
		"/eth/v1/node/syncing":         {http.MethodGet},
		"/eth/v1/node/health":          {http.MethodGet},
	}

	validatorRoutes := map[string][]string{
		"/eth/v1/validator/duties/attester/{epoch}":        {http.MethodPost},
		"/eth/v1/validator/duties/proposer/{epoch}":        {http.MethodGet},
		"/eth/v1/validator/duties/sync/{epoch}":            {http.MethodPost},
		"/eth/v2/validator/blocks/{slot}":                  {http.MethodGet}, //deprecated
		"/eth/v3/validator/blocks/{slot}":                  {http.MethodGet},
		"/eth/v1/validator/blinded_blocks/{slot}":          {http.MethodGet}, //deprecated
		"/eth/v1/validator/attestation_data":               {http.MethodGet},
		"/eth/v1/validator/aggregate_attestation":          {http.MethodGet},
		"/eth/v1/validator/aggregate_and_proofs":           {http.MethodPost},
		"/eth/v1/validator/beacon_committee_subscriptions": {http.MethodPost},
		"/eth/v1/validator/sync_committee_subscriptions":   {http.MethodPost},
		//"/eth/v1/validator/beacon_committee_selections":   {http.MethodPost}, // not implemented
		"/eth/v1/validator/sync_committee_contribution": {http.MethodGet},
		//"/eth/v1/validator/sync_committee_selections":  {http.MethodPost}, // not implemented
		"/eth/v1/validator/contribution_and_proofs": {http.MethodPost},
		"/eth/v1/validator/prepare_beacon_proposer": {http.MethodPost},
		"/eth/v1/validator/register_validator":      {http.MethodPost},
		"/eth/v1/validator/liveness/{epoch}":        {http.MethodPost},
	}

	prysmCustomRoutes := map[string][]string{
		"/prysm/v1/beacon/weak_subjectivity":               {http.MethodGet},
		"/prysm/node/trusted_peers":                        {http.MethodGet, http.MethodPost},
		"/prysm/node/trusted_peers/{peer_id}":              {http.MethodDelete},
		"/prysm/validators/performance":                    {http.MethodPost},
		"/eth/v1/beacon/states/{state_id}/validator_count": {http.MethodGet},
	}

	wantRouteList := combineMaps(beaconRoutes, builderRoutes, configRoutes, debugRoutes, eventsRoutes, nodeRoutes, validatorRoutes, prysmCustomRoutes)
	gotRouteList := make(map[string][]string)
	err := s.cfg.Router.Walk(func(route *mux.Route, router *mux.Router, ancestors []*mux.Route) error {
		tpl, err1 := route.GetPathTemplate()
		require.NoError(t, err1)
		met, err2 := route.GetMethods()
		require.NoError(t, err2)
		methods, ok := gotRouteList[tpl]
		if !ok {
			gotRouteList[tpl] = met
		} else {
			gotRouteList[tpl] = append(methods, met...)
		}
		return nil
	})
	require.NoError(t, err)
	require.DeepEqual(t, wantRouteList, gotRouteList)
}

func TestLifecycle_OK(t *testing.T) {
	hook := logTest.NewGlobal()
	chainService := &mock.ChainService{
		Genesis: time.Now(),
	}
	rpcService := NewService(context.Background(), &Config{
		Port:                  "7348",
		SyncService:           &mockSync.Sync{IsSyncing: false},
		BlockReceiver:         chainService,
		AttestationReceiver:   chainService,
		HeadFetcher:           chainService,
		GenesisTimeFetcher:    chainService,
		ExecutionChainService: &mockExecution.Chain{},
		StateNotifier:         chainService.StateNotifier(),
		Router:                mux.NewRouter(),
		ClockWaiter:           startup.NewClockSynchronizer(),
	})

	rpcService.Start()

	require.LogsContain(t, hook, "listening on port")
	assert.NoError(t, rpcService.Stop())
}

func TestStatus_CredentialError(t *testing.T) {
	credentialErr := errors.New("credentialError")
	s := &Service{
		cfg: &Config{SyncService: &mockSync.Sync{IsSyncing: false},
			OptimisticModeFetcher: &mock.ChainService{Optimistic: false}},
		credentialError: credentialErr,
	}

	assert.ErrorContains(t, s.credentialError.Error(), s.Status())
}

func TestStatus_Optimistic(t *testing.T) {
	s := &Service{
		cfg: &Config{SyncService: &mockSync.Sync{IsSyncing: false},
			OptimisticModeFetcher: &mock.ChainService{Optimistic: true}},
	}

	assert.ErrorContains(t, "service is optimistic", s.Status())
}

func TestRPC_InsecureEndpoint(t *testing.T) {
	hook := logTest.NewGlobal()
	chainService := &mock.ChainService{Genesis: time.Now()}
	rpcService := NewService(context.Background(), &Config{
		Port:                  "7777",
		SyncService:           &mockSync.Sync{IsSyncing: false},
		BlockReceiver:         chainService,
		GenesisTimeFetcher:    chainService,
		AttestationReceiver:   chainService,
		HeadFetcher:           chainService,
		ExecutionChainService: &mockExecution.Chain{},
		StateNotifier:         chainService.StateNotifier(),
		Router:                mux.NewRouter(),
		ClockWaiter:           startup.NewClockSynchronizer(),
	})

	rpcService.Start()

	require.LogsContain(t, hook, "listening on port")
	require.LogsContain(t, hook, "You are using an insecure gRPC server")
	assert.NoError(t, rpcService.Stop())
}
