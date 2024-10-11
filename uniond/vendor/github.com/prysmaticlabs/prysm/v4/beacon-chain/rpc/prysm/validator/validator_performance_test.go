package validator

import (
	"bytes"
	"context"
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/prysmaticlabs/go-bitfield"
	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/epoch/precompute"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/core"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	mockSync "github.com/prysmaticlabs/prysm/v4/beacon-chain/sync/initial-sync/testing"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestServer_GetValidatorPerformance(t *testing.T) {
	t.Run("Syncing", func(t *testing.T) {
		vs := &Server{
			CoreService: &core.Service{
				SyncChecker: &mockSync.Sync{IsSyncing: true},
			},
		}

		srv := httptest.NewServer(http.HandlerFunc(vs.GetValidatorPerformance))
		req := httptest.NewRequest("POST", "/foo", nil)

		client := &http.Client{}
		rawResp, err := client.Post(srv.URL, "application/json", req.Body)
		require.NoError(t, err)
		require.Equal(t, http.StatusServiceUnavailable, rawResp.StatusCode)
	})
	t.Run("OK", func(t *testing.T) {
		helpers.ClearCache()
		params.SetupTestConfigCleanup(t)
		params.OverrideBeaconConfig(params.MinimalSpecConfig())

		publicKeys := [][48]byte{
			bytesutil.ToBytes48([]byte{1}),
			bytesutil.ToBytes48([]byte{2}),
			bytesutil.ToBytes48([]byte{3}),
		}
		headState, err := util.NewBeaconState()
		require.NoError(t, err)
		headState = setHeadState(t, headState, publicKeys)
		require.NoError(t, headState.SetBalances([]uint64{100, 101, 102}))

		offset := int64(headState.Slot().Mul(params.BeaconConfig().SecondsPerSlot))
		vs := &Server{
			CoreService: &core.Service{
				HeadFetcher: &mock.ChainService{
					State: headState,
				},
				GenesisTimeFetcher: &mock.ChainService{Genesis: time.Now().Add(time.Duration(-1*offset) * time.Second)},
				SyncChecker:        &mockSync.Sync{IsSyncing: false},
			},
		}
		want := &PerformanceResponse{
			PublicKeys:                    [][]byte{publicKeys[1][:], publicKeys[2][:]},
			CurrentEffectiveBalances:      []uint64{params.BeaconConfig().MaxEffectiveBalance, params.BeaconConfig().MaxEffectiveBalance},
			CorrectlyVotedSource:          []bool{false, false},
			CorrectlyVotedTarget:          []bool{false, false},
			CorrectlyVotedHead:            []bool{false, false},
			BalancesBeforeEpochTransition: []uint64{101, 102},
			BalancesAfterEpochTransition:  []uint64{0, 0},
			MissingValidators:             [][]byte{publicKeys[0][:]},
		}

		request := &PerformanceRequest{
			PublicKeys: [][]byte{publicKeys[0][:], publicKeys[2][:], publicKeys[1][:]},
		}
		var buf bytes.Buffer
		err = json.NewEncoder(&buf).Encode(request)
		require.NoError(t, err)

		srv := httptest.NewServer(http.HandlerFunc(vs.GetValidatorPerformance))
		req := httptest.NewRequest("POST", "/foo", &buf)
		client := &http.Client{}
		rawResp, err := client.Post(srv.URL, "application/json", req.Body)
		require.NoError(t, err)
		defer func() {
			if err := rawResp.Body.Close(); err != nil {
				t.Fatal(err)
			}
		}()
		body, err := io.ReadAll(rawResp.Body)
		require.NoError(t, err)

		response := &PerformanceResponse{}
		require.NoError(t, json.Unmarshal(body, response))
		require.DeepEqual(t, want, response)
	})
	t.Run("Indices", func(t *testing.T) {
		ctx := context.Background()
		publicKeys := [][48]byte{
			bytesutil.ToBytes48([]byte{1}),
			bytesutil.ToBytes48([]byte{2}),
			bytesutil.ToBytes48([]byte{3}),
		}
		headState, err := util.NewBeaconState()
		require.NoError(t, err)
		headState = setHeadState(t, headState, publicKeys)

		offset := int64(headState.Slot().Mul(params.BeaconConfig().SecondsPerSlot))
		vs := &Server{
			CoreService: &core.Service{
				HeadFetcher: &mock.ChainService{
					// 10 epochs into the future.
					State: headState,
				},
				SyncChecker:        &mockSync.Sync{IsSyncing: false},
				GenesisTimeFetcher: &mock.ChainService{Genesis: time.Now().Add(time.Duration(-1*offset) * time.Second)},
			},
		}
		c := headState.Copy()
		vp, bp, err := precompute.New(ctx, c)
		require.NoError(t, err)
		vp, bp, err = precompute.ProcessAttestations(ctx, c, vp, bp)
		require.NoError(t, err)
		_, err = precompute.ProcessRewardsAndPenaltiesPrecompute(c, bp, vp, precompute.AttestationsDelta, precompute.ProposersDelta)
		require.NoError(t, err)
		extraBal := params.BeaconConfig().MaxEffectiveBalance + params.BeaconConfig().GweiPerEth

		want := &PerformanceResponse{
			PublicKeys:                    [][]byte{publicKeys[1][:], publicKeys[2][:]},
			CurrentEffectiveBalances:      []uint64{params.BeaconConfig().MaxEffectiveBalance, params.BeaconConfig().MaxEffectiveBalance},
			CorrectlyVotedSource:          []bool{false, false},
			CorrectlyVotedTarget:          []bool{false, false},
			CorrectlyVotedHead:            []bool{false, false},
			BalancesBeforeEpochTransition: []uint64{extraBal, extraBal + params.BeaconConfig().GweiPerEth},
			BalancesAfterEpochTransition:  []uint64{vp[1].AfterEpochTransitionBalance, vp[2].AfterEpochTransitionBalance},
			MissingValidators:             [][]byte{publicKeys[0][:]},
		}
		request := &PerformanceRequest{
			Indices: []primitives.ValidatorIndex{2, 1, 0},
		}
		var buf bytes.Buffer
		err = json.NewEncoder(&buf).Encode(request)
		require.NoError(t, err)

		srv := httptest.NewServer(http.HandlerFunc(vs.GetValidatorPerformance))
		req := httptest.NewRequest("POST", "/foo", &buf)
		client := &http.Client{}
		rawResp, err := client.Post(srv.URL, "application/json", req.Body)
		require.NoError(t, err)
		defer func() {
			if err := rawResp.Body.Close(); err != nil {
				t.Fatal(err)
			}
		}()
		body, err := io.ReadAll(rawResp.Body)
		require.NoError(t, err)

		response := &PerformanceResponse{}
		require.NoError(t, json.Unmarshal(body, response))
		require.DeepEqual(t, want, response)
	})
	t.Run("Indices Pubkeys", func(t *testing.T) {
		ctx := context.Background()
		publicKeys := [][48]byte{
			bytesutil.ToBytes48([]byte{1}),
			bytesutil.ToBytes48([]byte{2}),
			bytesutil.ToBytes48([]byte{3}),
		}
		headState, err := util.NewBeaconState()
		require.NoError(t, err)
		headState = setHeadState(t, headState, publicKeys)

		offset := int64(headState.Slot().Mul(params.BeaconConfig().SecondsPerSlot))
		vs := &Server{
			CoreService: &core.Service{
				HeadFetcher: &mock.ChainService{
					// 10 epochs into the future.
					State: headState,
				},
				SyncChecker:        &mockSync.Sync{IsSyncing: false},
				GenesisTimeFetcher: &mock.ChainService{Genesis: time.Now().Add(time.Duration(-1*offset) * time.Second)},
			},
		}
		c := headState.Copy()
		vp, bp, err := precompute.New(ctx, c)
		require.NoError(t, err)
		vp, bp, err = precompute.ProcessAttestations(ctx, c, vp, bp)
		require.NoError(t, err)
		_, err = precompute.ProcessRewardsAndPenaltiesPrecompute(c, bp, vp, precompute.AttestationsDelta, precompute.ProposersDelta)
		require.NoError(t, err)
		extraBal := params.BeaconConfig().MaxEffectiveBalance + params.BeaconConfig().GweiPerEth

		want := &PerformanceResponse{
			PublicKeys:                    [][]byte{publicKeys[1][:], publicKeys[2][:]},
			CurrentEffectiveBalances:      []uint64{params.BeaconConfig().MaxEffectiveBalance, params.BeaconConfig().MaxEffectiveBalance},
			CorrectlyVotedSource:          []bool{false, false},
			CorrectlyVotedTarget:          []bool{false, false},
			CorrectlyVotedHead:            []bool{false, false},
			BalancesBeforeEpochTransition: []uint64{extraBal, extraBal + params.BeaconConfig().GweiPerEth},
			BalancesAfterEpochTransition:  []uint64{vp[1].AfterEpochTransitionBalance, vp[2].AfterEpochTransitionBalance},
			MissingValidators:             [][]byte{publicKeys[0][:]},
		}
		request := &PerformanceRequest{
			PublicKeys: [][]byte{publicKeys[0][:], publicKeys[2][:]}, Indices: []primitives.ValidatorIndex{1, 2},
		}
		var buf bytes.Buffer
		err = json.NewEncoder(&buf).Encode(request)
		require.NoError(t, err)

		srv := httptest.NewServer(http.HandlerFunc(vs.GetValidatorPerformance))
		req := httptest.NewRequest("POST", "/foo", &buf)
		client := &http.Client{}
		rawResp, err := client.Post(srv.URL, "application/json", req.Body)
		require.NoError(t, err)
		defer func() {
			if err := rawResp.Body.Close(); err != nil {
				t.Fatal(err)
			}
		}()
		body, err := io.ReadAll(rawResp.Body)
		require.NoError(t, err)

		response := &PerformanceResponse{}
		require.NoError(t, json.Unmarshal(body, response))
		require.DeepEqual(t, want, response)
	})
	t.Run("Altair OK", func(t *testing.T) {
		helpers.ClearCache()
		params.SetupTestConfigCleanup(t)
		params.OverrideBeaconConfig(params.MinimalSpecConfig())

		publicKeys := [][48]byte{
			bytesutil.ToBytes48([]byte{1}),
			bytesutil.ToBytes48([]byte{2}),
			bytesutil.ToBytes48([]byte{3}),
		}
		epoch := primitives.Epoch(1)
		headState, _ := util.DeterministicGenesisStateAltair(t, 32)
		require.NoError(t, headState.SetSlot(params.BeaconConfig().SlotsPerEpoch.Mul(uint64(epoch+1))))
		headState = setHeadState(t, headState, publicKeys)

		require.NoError(t, headState.SetInactivityScores([]uint64{0, 0, 0}))
		require.NoError(t, headState.SetBalances([]uint64{100, 101, 102}))
		offset := int64(headState.Slot().Mul(params.BeaconConfig().SecondsPerSlot))
		vs := &Server{
			CoreService: &core.Service{
				HeadFetcher: &mock.ChainService{
					State: headState,
				},
				GenesisTimeFetcher: &mock.ChainService{Genesis: time.Now().Add(time.Duration(-1*offset) * time.Second)},
				SyncChecker:        &mockSync.Sync{IsSyncing: false},
			},
		}
		want := &PerformanceResponse{
			PublicKeys:                    [][]byte{publicKeys[1][:], publicKeys[2][:]},
			CurrentEffectiveBalances:      []uint64{params.BeaconConfig().MaxEffectiveBalance, params.BeaconConfig().MaxEffectiveBalance},
			CorrectlyVotedSource:          []bool{false, false},
			CorrectlyVotedTarget:          []bool{false, false},
			CorrectlyVotedHead:            []bool{false, false},
			BalancesBeforeEpochTransition: []uint64{101, 102},
			BalancesAfterEpochTransition:  []uint64{0, 0},
			MissingValidators:             [][]byte{publicKeys[0][:]},
			InactivityScores:              []uint64{0, 0},
		}
		request := &PerformanceRequest{
			PublicKeys: [][]byte{publicKeys[0][:], publicKeys[2][:], publicKeys[1][:]},
		}
		var buf bytes.Buffer
		err := json.NewEncoder(&buf).Encode(request)
		require.NoError(t, err)

		srv := httptest.NewServer(http.HandlerFunc(vs.GetValidatorPerformance))
		req := httptest.NewRequest("POST", "/foo", &buf)
		client := &http.Client{}
		rawResp, err := client.Post(srv.URL, "application/json", req.Body)
		require.NoError(t, err)
		defer func() {
			if err := rawResp.Body.Close(); err != nil {
				t.Fatal(err)
			}
		}()
		body, err := io.ReadAll(rawResp.Body)
		require.NoError(t, err)

		response := &PerformanceResponse{}
		require.NoError(t, json.Unmarshal(body, response))
		require.DeepEqual(t, want, response)
	})
	t.Run("Bellatrix OK", func(t *testing.T) {
		helpers.ClearCache()
		params.SetupTestConfigCleanup(t)
		params.OverrideBeaconConfig(params.MinimalSpecConfig())

		publicKeys := [][48]byte{
			bytesutil.ToBytes48([]byte{1}),
			bytesutil.ToBytes48([]byte{2}),
			bytesutil.ToBytes48([]byte{3}),
		}
		epoch := primitives.Epoch(1)
		headState, _ := util.DeterministicGenesisStateBellatrix(t, 32)
		require.NoError(t, headState.SetSlot(params.BeaconConfig().SlotsPerEpoch.Mul(uint64(epoch+1))))
		headState = setHeadState(t, headState, publicKeys)

		require.NoError(t, headState.SetInactivityScores([]uint64{0, 0, 0}))
		require.NoError(t, headState.SetBalances([]uint64{100, 101, 102}))
		offset := int64(headState.Slot().Mul(params.BeaconConfig().SecondsPerSlot))
		vs := &Server{
			CoreService: &core.Service{
				HeadFetcher: &mock.ChainService{
					State: headState,
				},
				GenesisTimeFetcher: &mock.ChainService{Genesis: time.Now().Add(time.Duration(-1*offset) * time.Second)},
				SyncChecker:        &mockSync.Sync{IsSyncing: false},
			},
		}
		want := &PerformanceResponse{
			PublicKeys:                    [][]byte{publicKeys[1][:], publicKeys[2][:]},
			CurrentEffectiveBalances:      []uint64{params.BeaconConfig().MaxEffectiveBalance, params.BeaconConfig().MaxEffectiveBalance},
			CorrectlyVotedSource:          []bool{false, false},
			CorrectlyVotedTarget:          []bool{false, false},
			CorrectlyVotedHead:            []bool{false, false},
			BalancesBeforeEpochTransition: []uint64{101, 102},
			BalancesAfterEpochTransition:  []uint64{0, 0},
			MissingValidators:             [][]byte{publicKeys[0][:]},
			InactivityScores:              []uint64{0, 0},
		}
		request := &PerformanceRequest{
			PublicKeys: [][]byte{publicKeys[0][:], publicKeys[2][:], publicKeys[1][:]},
		}
		var buf bytes.Buffer
		err := json.NewEncoder(&buf).Encode(request)
		require.NoError(t, err)

		srv := httptest.NewServer(http.HandlerFunc(vs.GetValidatorPerformance))
		req := httptest.NewRequest("POST", "/foo", &buf)
		client := &http.Client{}
		rawResp, err := client.Post(srv.URL, "application/json", req.Body)
		require.NoError(t, err)
		defer func() {
			if err := rawResp.Body.Close(); err != nil {
				t.Fatal(err)
			}
		}()
		body, err := io.ReadAll(rawResp.Body)
		require.NoError(t, err)

		response := &PerformanceResponse{}
		require.NoError(t, json.Unmarshal(body, response))
		require.DeepEqual(t, want, response)
	})
	t.Run("Capella OK", func(t *testing.T) {
		helpers.ClearCache()
		params.SetupTestConfigCleanup(t)
		params.OverrideBeaconConfig(params.MinimalSpecConfig())

		publicKeys := [][48]byte{
			bytesutil.ToBytes48([]byte{1}),
			bytesutil.ToBytes48([]byte{2}),
			bytesutil.ToBytes48([]byte{3}),
		}
		epoch := primitives.Epoch(1)
		headState, _ := util.DeterministicGenesisStateCapella(t, 32)
		require.NoError(t, headState.SetSlot(params.BeaconConfig().SlotsPerEpoch.Mul(uint64(epoch+1))))
		headState = setHeadState(t, headState, publicKeys)

		require.NoError(t, headState.SetInactivityScores([]uint64{0, 0, 0}))
		require.NoError(t, headState.SetBalances([]uint64{100, 101, 102}))
		offset := int64(headState.Slot().Mul(params.BeaconConfig().SecondsPerSlot))
		vs := &Server{
			CoreService: &core.Service{
				HeadFetcher: &mock.ChainService{
					State: headState,
				},
				GenesisTimeFetcher: &mock.ChainService{Genesis: time.Now().Add(time.Duration(-1*offset) * time.Second)},
				SyncChecker:        &mockSync.Sync{IsSyncing: false},
			},
		}
		want := &PerformanceResponse{
			PublicKeys:                    [][]byte{publicKeys[1][:], publicKeys[2][:]},
			CurrentEffectiveBalances:      []uint64{params.BeaconConfig().MaxEffectiveBalance, params.BeaconConfig().MaxEffectiveBalance},
			CorrectlyVotedSource:          []bool{false, false},
			CorrectlyVotedTarget:          []bool{false, false},
			CorrectlyVotedHead:            []bool{false, false},
			BalancesBeforeEpochTransition: []uint64{101, 102},
			BalancesAfterEpochTransition:  []uint64{0, 0},
			MissingValidators:             [][]byte{publicKeys[0][:]},
			InactivityScores:              []uint64{0, 0},
		}
		request := &PerformanceRequest{
			PublicKeys: [][]byte{publicKeys[0][:], publicKeys[2][:], publicKeys[1][:]},
		}
		var buf bytes.Buffer
		err := json.NewEncoder(&buf).Encode(request)
		require.NoError(t, err)

		srv := httptest.NewServer(http.HandlerFunc(vs.GetValidatorPerformance))
		req := httptest.NewRequest("POST", "/foo", &buf)
		client := &http.Client{}
		rawResp, err := client.Post(srv.URL, "application/json", req.Body)
		require.NoError(t, err)
		defer func() {
			if err := rawResp.Body.Close(); err != nil {
				t.Fatal(err)
			}
		}()
		body, err := io.ReadAll(rawResp.Body)
		require.NoError(t, err)

		response := &PerformanceResponse{}
		require.NoError(t, json.Unmarshal(body, response))
		require.DeepEqual(t, want, response)
	})
}

func setHeadState(t *testing.T, headState state.BeaconState, publicKeys [][48]byte) state.BeaconState {
	epoch := primitives.Epoch(1)
	require.NoError(t, headState.SetSlot(params.BeaconConfig().SlotsPerEpoch.Mul(uint64(epoch+1))))
	if headState.Version() < version.Altair {
		atts := make([]*ethpb.PendingAttestation, 3)
		for i := 0; i < len(atts); i++ {
			atts[i] = &ethpb.PendingAttestation{
				Data: &ethpb.AttestationData{
					Target: &ethpb.Checkpoint{Root: make([]byte, 32)},
					Source: &ethpb.Checkpoint{Root: make([]byte, 32)},
				},
				AggregationBits: bitfield.Bitlist{},
				InclusionDelay:  1,
			}
			require.NoError(t, headState.AppendPreviousEpochAttestations(atts[i]))
		}
	}

	defaultBal := params.BeaconConfig().MaxEffectiveBalance
	extraBal := params.BeaconConfig().MaxEffectiveBalance + params.BeaconConfig().GweiPerEth
	balances := []uint64{defaultBal, extraBal, extraBal + params.BeaconConfig().GweiPerEth}
	require.NoError(t, headState.SetBalances(balances))

	validators := []*ethpb.Validator{
		{
			PublicKey:       publicKeys[0][:],
			ActivationEpoch: 5,
			ExitEpoch:       params.BeaconConfig().FarFutureEpoch,
		},
		{
			PublicKey:        publicKeys[1][:],
			EffectiveBalance: defaultBal,
			ActivationEpoch:  0,
			ExitEpoch:        params.BeaconConfig().FarFutureEpoch,
		},
		{
			PublicKey:        publicKeys[2][:],
			EffectiveBalance: defaultBal,
			ActivationEpoch:  0,
			ExitEpoch:        params.BeaconConfig().FarFutureEpoch,
		},
	}
	require.NoError(t, headState.SetValidators(validators))
	return headState
}
