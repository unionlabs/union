package node

import (
	"context"
	"flag"
	"fmt"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strconv"
	"testing"
	"time"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/builder"
	statefeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/state"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db/filesystem"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/execution"
	mockExecution "github.com/prysmaticlabs/prysm/v4/beacon-chain/execution/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/monitor"
	"github.com/prysmaticlabs/prysm/v4/cmd"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/flags"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime"
	"github.com/prysmaticlabs/prysm/v4/runtime/interop"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	logTest "github.com/sirupsen/logrus/hooks/test"
	"github.com/urfave/cli/v2"
)

// Ensure BeaconNode implements interfaces.
var _ statefeed.Notifier = (*BeaconNode)(nil)

func newCliContextWithCancel(app *cli.App, set *flag.FlagSet) (*cli.Context, context.CancelFunc) {
	context, cancel := context.WithCancel(context.Background())
	parent := &cli.Context{Context: context}
	return cli.NewContext(app, set, parent), cancel
}

// Test that beacon chain node can close.
func TestNodeClose_OK(t *testing.T) {
	hook := logTest.NewGlobal()
	tmp := fmt.Sprintf("%s/datadirtest2", t.TempDir())

	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.Bool("test-skip-pow", true, "skip pow dial")
	set.String("datadir", tmp, "node data directory")
	set.String("p2p-encoding", "ssz", "p2p encoding scheme")
	set.Bool("demo-config", true, "demo configuration")
	set.String("deposit-contract", "0x0000000000000000000000000000000000000000", "deposit contract address")
	set.String("suggested-fee-recipient", "0x6e35733c5af9B61374A128e6F85f553aF09ff89A", "fee recipient")
	require.NoError(t, set.Set("suggested-fee-recipient", "0x6e35733c5af9B61374A128e6F85f553aF09ff89A"))
	cmd.ValidatorMonitorIndicesFlag.Value = &cli.IntSlice{}
	cmd.ValidatorMonitorIndicesFlag.Value.SetInt(1)
	ctx, cancel := newCliContextWithCancel(&app, set)

	node, err := New(ctx, cancel, WithBlobStorage(filesystem.NewEphemeralBlobStorage(t)))
	require.NoError(t, err)

	node.Close()

	require.LogsContain(t, hook, "Stopping beacon node")
}

func TestNodeStart_Ok(t *testing.T) {
	hook := logTest.NewGlobal()
	app := cli.App{}
	tmp := fmt.Sprintf("%s/datadirtest2", t.TempDir())
	set := flag.NewFlagSet("test", 0)
	set.String("datadir", tmp, "node data directory")
	set.String("suggested-fee-recipient", "0x6e35733c5af9B61374A128e6F85f553aF09ff89A", "fee recipient")
	require.NoError(t, set.Set("suggested-fee-recipient", "0x6e35733c5af9B61374A128e6F85f553aF09ff89A"))

	ctx, cancel := newCliContextWithCancel(&app, set)
	node, err := New(ctx, cancel, WithBlockchainFlagOptions([]blockchain.Option{}),
		WithBuilderFlagOptions([]builder.Option{}),
		WithExecutionChainOptions([]execution.Option{}),
		WithBlobStorage(filesystem.NewEphemeralBlobStorage(t)))
	require.NoError(t, err)
	node.services = &runtime.ServiceRegistry{}
	go func() {
		node.Start()
	}()
	time.Sleep(3 * time.Second)
	node.Close()
	require.LogsContain(t, hook, "Starting beacon node")
}

func TestNodeStart_Ok_registerDeterministicGenesisService(t *testing.T) {
	numValidators := uint64(1)
	hook := logTest.NewGlobal()
	app := cli.App{}
	tmp := fmt.Sprintf("%s/datadirtest2", t.TempDir())
	set := flag.NewFlagSet("test", 0)
	set.String("datadir", tmp, "node data directory")
	set.Uint64(flags.InteropNumValidatorsFlag.Name, numValidators, "")
	set.String("suggested-fee-recipient", "0x6e35733c5af9B61374A128e6F85f553aF09ff89A", "fee recipient")
	require.NoError(t, set.Set("suggested-fee-recipient", "0x6e35733c5af9B61374A128e6F85f553aF09ff89A"))
	genesisState, _, err := interop.GenerateGenesisState(context.Background(), 0, numValidators)
	require.NoError(t, err, "Could not generate genesis beacon state")
	for i := uint64(1); i < 2; i++ {
		var someRoot [32]byte
		var someKey [fieldparams.BLSPubkeyLength]byte
		copy(someRoot[:], strconv.Itoa(int(i)))
		copy(someKey[:], strconv.Itoa(int(i)))
		genesisState.Validators = append(genesisState.Validators, &ethpb.Validator{
			PublicKey:                  someKey[:],
			WithdrawalCredentials:      someRoot[:],
			EffectiveBalance:           params.BeaconConfig().MaxEffectiveBalance,
			Slashed:                    false,
			ActivationEligibilityEpoch: 1,
			ActivationEpoch:            1,
			ExitEpoch:                  1,
			WithdrawableEpoch:          1,
		})
		genesisState.Balances = append(genesisState.Balances, params.BeaconConfig().MaxEffectiveBalance)
	}
	genesisBytes, err := genesisState.MarshalSSZ()
	require.NoError(t, err)
	require.NoError(t, os.WriteFile("genesis_ssz.json", genesisBytes, 0666))
	set.String("genesis-state", "genesis_ssz.json", "")
	ctx, cancel := newCliContextWithCancel(&app, set)
	node, err := New(ctx, cancel, WithBlockchainFlagOptions([]blockchain.Option{}),
		WithBuilderFlagOptions([]builder.Option{}),
		WithExecutionChainOptions([]execution.Option{}),
		WithBlobStorage(filesystem.NewEphemeralBlobStorage(t)))
	require.NoError(t, err)
	node.services = &runtime.ServiceRegistry{}
	go func() {
		node.Start()
	}()
	time.Sleep(3 * time.Second)
	node.Close()
	require.LogsContain(t, hook, "Starting beacon node")
	require.NoError(t, os.Remove("genesis_ssz.json"))
}

// TestClearDB tests clearing the database
func TestClearDB(t *testing.T) {
	hook := logTest.NewGlobal()
	srv, endpoint, err := mockExecution.SetupRPCServer()
	require.NoError(t, err)
	t.Cleanup(func() {
		srv.Stop()
	})

	tmp := filepath.Join(t.TempDir(), "datadirtest")

	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.String("datadir", tmp, "node data directory")
	set.Bool(cmd.ForceClearDB.Name, true, "force clear db")
	set.String("suggested-fee-recipient", "0x6e35733c5af9B61374A128e6F85f553aF09ff89A", "fee recipient")
	require.NoError(t, set.Set("suggested-fee-recipient", "0x6e35733c5af9B61374A128e6F85f553aF09ff89A"))
	context, cancel := newCliContextWithCancel(&app, set)
	options := []Option{
		WithExecutionChainOptions([]execution.Option{execution.WithHttpEndpoint(endpoint)}),
		WithBlobStorage(filesystem.NewEphemeralBlobStorage(t)),
	}
	_, err = New(context, cancel, options...)
	require.NoError(t, err)
	require.LogsContain(t, hook, "Removing database")
}

func TestMonitor_RegisteredCorrectly(t *testing.T) {
	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	require.NoError(t, cmd.ValidatorMonitorIndicesFlag.Apply(set))
	cliCtx := cli.NewContext(&app, set, nil)
	require.NoError(t, cliCtx.Set(cmd.ValidatorMonitorIndicesFlag.Name, "1,2"))
	n := &BeaconNode{ctx: context.Background(), cliCtx: cliCtx, services: runtime.NewServiceRegistry()}
	require.NoError(t, n.services.RegisterService(&blockchain.Service{}))
	require.NoError(t, n.registerValidatorMonitorService(make(chan struct{})))

	var mService *monitor.Service
	require.NoError(t, n.services.FetchService(&mService))
	require.Equal(t, true, mService.TrackedValidators[1])
	require.Equal(t, true, mService.TrackedValidators[2])
	require.Equal(t, false, mService.TrackedValidators[100])
}

func Test_hasNetworkFlag(t *testing.T) {
	tests := []struct {
		name         string
		networkName  string
		networkValue string
		want         bool
	}{
		{
			name:         "Prater testnet",
			networkName:  features.PraterTestnet.Name,
			networkValue: "prater",
			want:         true,
		},
		{
			name:         "Mainnet",
			networkName:  features.Mainnet.Name,
			networkValue: "mainnet",
			want:         true,
		},
		{
			name:         "No network flag",
			networkName:  "",
			networkValue: "",
			want:         false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			set := flag.NewFlagSet("test", 0)
			set.String(tt.networkName, tt.networkValue, tt.name)

			cliCtx := cli.NewContext(&cli.App{}, set, nil)
			err := cliCtx.Set(tt.networkName, tt.networkValue)
			require.NoError(t, err)

			if got := hasNetworkFlag(cliCtx); got != tt.want {
				t.Errorf("hasNetworkFlag() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestCORS(t *testing.T) {
	// Mock CLI context with a test CORS domain
	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.String(flags.GPRCGatewayCorsDomain.Name, "http://allowed-example.com", "")
	cliCtx := cli.NewContext(&app, set, nil)
	require.NoError(t, cliCtx.Set(flags.GPRCGatewayCorsDomain.Name, "http://allowed-example.com"))

	router := newRouter(cliCtx)

	// Ensure a test route exists
	router.HandleFunc("/some-path", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	}).Methods(http.MethodGet)

	// Define test cases
	tests := []struct {
		name        string
		origin      string
		expectAllow bool
	}{
		{"AllowedOrigin", "http://allowed-example.com", true},
		{"DisallowedOrigin", "http://disallowed-example.com", false},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {

			// Create a request and response recorder
			req := httptest.NewRequest("GET", "http://example.com/some-path", nil)
			req.Header.Set("Origin", tc.origin)
			rr := httptest.NewRecorder()

			// Serve HTTP
			router.ServeHTTP(rr, req)

			// Check the CORS headers based on the expected outcome
			if tc.expectAllow && rr.Header().Get("Access-Control-Allow-Origin") != tc.origin {
				t.Errorf("Expected Access-Control-Allow-Origin header to be %v, got %v", tc.origin, rr.Header().Get("Access-Control-Allow-Origin"))
			}
			if !tc.expectAllow && rr.Header().Get("Access-Control-Allow-Origin") != "" {
				t.Errorf("Expected Access-Control-Allow-Origin header to be empty for disallowed origin, got %v", rr.Header().Get("Access-Control-Allow-Origin"))
			}
		})
	}
}
