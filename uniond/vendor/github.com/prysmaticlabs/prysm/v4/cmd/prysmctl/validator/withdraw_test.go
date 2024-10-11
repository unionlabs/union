package validator

import (
	"encoding/json"
	"flag"
	"fmt"
	"net"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/prysmaticlabs/prysm/v4/api/server"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/beacon"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/config"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	logtest "github.com/sirupsen/logrus/hooks/test"
	"github.com/urfave/cli/v2"
)

func getHappyPathTestServer(file string, t *testing.T) *httptest.Server {
	return httptest.NewUnstartedServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(200)
		w.Header().Set("Content-Type", "application/json")
		if r.Method == http.MethodGet {
			fmt.Println(r.RequestURI)
			if r.RequestURI == "/eth/v1/beacon/pool/bls_to_execution_changes" {
				b, err := os.ReadFile(filepath.Clean(file))
				require.NoError(t, err)
				var to []*shared.SignedBLSToExecutionChange
				err = json.Unmarshal(b, &to)
				require.NoError(t, err)
				err = json.NewEncoder(w).Encode(&beacon.BLSToExecutionChangesPoolResponse{
					Data: to,
				})
				require.NoError(t, err)
			} else if r.RequestURI == "/eth/v1/beacon/states/head/fork" {
				err := json.NewEncoder(w).Encode(&beacon.GetStateForkResponse{
					Data: &shared.Fork{
						PreviousVersion: hexutil.Encode(params.BeaconConfig().CapellaForkVersion),
						CurrentVersion:  hexutil.Encode(params.BeaconConfig().CapellaForkVersion),
						Epoch:           "1350",
					},
					ExecutionOptimistic: false,
					Finalized:           true,
				})
				require.NoError(t, err)
			} else if r.RequestURI == "/eth/v1/config/spec" {
				m := make(map[string]string)
				m["CAPELLA_FORK_EPOCH"] = "1350"
				err := json.NewEncoder(w).Encode(&config.GetSpecResponse{
					Data: m,
				})
				require.NoError(t, err)
			}

		}
	}))
}

func TestCallWithdrawalEndpoint(t *testing.T) {
	file := "./testdata/change-operations.json"
	baseurl := "127.0.0.1:3500"
	l, err := net.Listen("tcp", baseurl)
	require.NoError(t, err)
	srv := getHappyPathTestServer(file, t)
	err = srv.Listener.Close()
	require.NoError(t, err)
	srv.Listener = l
	srv.Start()
	defer srv.Close()
	hook := logtest.NewGlobal()

	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.String("beacon-node-host", baseurl, "")
	set.String("path", file, "")
	set.Bool("confirm", true, "")
	set.Bool("accept-terms-of-use", true, "")
	assert.NoError(t, set.Set("beacon-node-host", baseurl))
	assert.NoError(t, set.Set("path", file))
	cliCtx := cli.NewContext(&app, set, nil)

	err = setWithdrawalAddresses(cliCtx)
	require.NoError(t, err)

	assert.LogsContain(t, hook, "Successfully published")
}

func TestCallWithdrawalEndpoint_Multiple(t *testing.T) {
	file := "./testdata/change-operations-multiple.json"
	baseurl := "127.0.0.1:3500"
	l, err := net.Listen("tcp", baseurl)
	require.NoError(t, err)
	srv := getHappyPathTestServer(file, t)
	err = srv.Listener.Close()
	require.NoError(t, err)
	srv.Listener = l
	srv.Start()
	defer srv.Close()
	hook := logtest.NewGlobal()

	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.String("beacon-node-host", baseurl, "")
	set.String("path", file, "")
	set.Bool("confirm", true, "")
	set.Bool("accept-terms-of-use", true, "")
	assert.NoError(t, set.Set("beacon-node-host", baseurl))
	assert.NoError(t, set.Set("path", file))
	cliCtx := cli.NewContext(&app, set, nil)

	err = setWithdrawalAddresses(cliCtx)
	require.NoError(t, err)
	assert.LogsContain(t, hook, "Successfully published")
	assert.LogsContain(t, hook, "to update 2 withdrawal")
	assert.LogsContain(t, hook, "All (total:2) signed withdrawal messages were found in the pool.")
	assert.LogsDoNotContain(t, hook, "Set withdrawal address message not found in the node's operations pool.")
}

func TestCallWithdrawalEndpoint_Multiple_stakingcli(t *testing.T) {
	stakingcliFile := "./testdata/staking-cli-change-operations-multiple.json"
	file := "./testdata/change-operations-multiple.json"
	baseurl := "127.0.0.1:3500"
	l, err := net.Listen("tcp", baseurl)
	require.NoError(t, err)
	srv := getHappyPathTestServer(file, t)
	err = srv.Listener.Close()
	require.NoError(t, err)
	srv.Listener = l
	srv.Start()
	defer srv.Close()
	hook := logtest.NewGlobal()

	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.String("beacon-node-host", baseurl, "")
	set.String("path", stakingcliFile, "")
	set.Bool("confirm", true, "")
	set.Bool("accept-terms-of-use", true, "")
	assert.NoError(t, set.Set("beacon-node-host", baseurl))
	assert.NoError(t, set.Set("path", stakingcliFile))
	cliCtx := cli.NewContext(&app, set, nil)

	err = setWithdrawalAddresses(cliCtx)
	require.NoError(t, err)
	assert.LogsContain(t, hook, "Successfully published")
	assert.LogsContain(t, hook, "to update 2 withdrawal")
	assert.LogsContain(t, hook, "All (total:2) signed withdrawal messages were found in the pool.")
	assert.LogsDoNotContain(t, hook, "Set withdrawal address message not found in the node's operations pool.")
}

func TestCallWithdrawalEndpoint_Multiple_notfound(t *testing.T) {
	respFile := "./testdata/change-operations-multiple_notfound.json"
	file := "./testdata/change-operations-multiple.json"
	baseurl := "127.0.0.1:3500"
	l, err := net.Listen("tcp", baseurl)
	require.NoError(t, err)
	srv := getHappyPathTestServer(respFile, t)
	err = srv.Listener.Close()
	require.NoError(t, err)
	srv.Listener = l
	srv.Start()
	defer srv.Close()
	hook := logtest.NewGlobal()

	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.String("beacon-node-host", baseurl, "")
	set.String("path", file, "")
	set.Bool("confirm", true, "")
	set.Bool("accept-terms-of-use", true, "")
	assert.NoError(t, set.Set("beacon-node-host", baseurl))
	assert.NoError(t, set.Set("path", file))
	cliCtx := cli.NewContext(&app, set, nil)

	err = setWithdrawalAddresses(cliCtx)
	require.NoError(t, err)
	assert.LogsContain(t, hook, "Successfully published")
	assert.LogsContain(t, hook, "to update 2 withdrawal")
	assert.LogsContain(t, hook, "Set withdrawal address message not found in the node's operations pool.")
	assert.LogsContain(t, hook, "Please check before resubmitting. Set withdrawal address messages that were not found in the pool may have been already included into a block.")
	assert.LogsDoNotContain(t, hook, "Set withdrawal address message found in the node's operations pool.")
}

func TestCallWithdrawalEndpoint_Empty(t *testing.T) {
	baseurl := "127.0.0.1:3500"
	content := []byte("[]")
	tmpfile, err := os.CreateTemp("./testdata", "*.json")
	require.NoError(t, err)
	_, err = tmpfile.Write(content)
	require.NoError(t, err)
	defer func() {
		err := os.Remove(tmpfile.Name())
		require.NoError(t, err)
	}()
	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.String("beacon-node-host", baseurl, "")
	set.String("path", tmpfile.Name(), "")
	set.Bool("confirm", true, "")
	set.Bool("accept-terms-of-use", true, "")
	assert.NoError(t, set.Set("beacon-node-host", baseurl))
	assert.NoError(t, set.Set("path", tmpfile.Name()))
	cliCtx := cli.NewContext(&app, set, nil)
	err = setWithdrawalAddresses(cliCtx)
	assert.ErrorContains(t, "the list of signed requests is empty", err)
}

func TestCallWithdrawalEndpoint_Errors(t *testing.T) {
	file := "./testdata/change-operations.json"
	baseurl := "127.0.0.1:3500"
	l, err := net.Listen("tcp", baseurl)
	require.NoError(t, err)
	srv := httptest.NewUnstartedServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method == http.MethodPost && r.RequestURI == "/eth/v1/beacon/pool/bls_to_execution_changes" {
			w.WriteHeader(400)
			w.Header().Set("Content-Type", "application/json")
			err = json.NewEncoder(w).Encode(&server.IndexedVerificationFailureError{
				Failures: []*server.IndexedVerificationFailure{
					{Index: 0, Message: "Could not validate SignedBLSToExecutionChange"},
				},
			})
			require.NoError(t, err)
		} else if r.Method == http.MethodGet {
			if r.RequestURI == "/eth/v1/beacon/states/head/fork" {
				w.WriteHeader(200)
				w.Header().Set("Content-Type", "application/json")
				err := json.NewEncoder(w).Encode(&beacon.GetStateForkResponse{
					Data: &shared.Fork{
						PreviousVersion: hexutil.Encode(params.BeaconConfig().CapellaForkVersion),
						CurrentVersion:  hexutil.Encode(params.BeaconConfig().CapellaForkVersion),
						Epoch:           fmt.Sprintf("%d", params.BeaconConfig().CapellaForkEpoch),
					},
					ExecutionOptimistic: false,
					Finalized:           true,
				})
				require.NoError(t, err)
			} else if r.RequestURI == "/eth/v1/config/spec" {
				w.WriteHeader(200)
				w.Header().Set("Content-Type", "application/json")
				m := make(map[string]string)
				m["CAPELLA_FORK_EPOCH"] = "1350"
				err := json.NewEncoder(w).Encode(&config.GetSpecResponse{
					Data: m,
				})
				require.NoError(t, err)
			} else {
				w.WriteHeader(400)
				w.Header().Set("Content-Type", "application/json")
			}
		}
	}))
	err = srv.Listener.Close()
	require.NoError(t, err)
	srv.Listener = l
	srv.Start()
	defer srv.Close()
	hook := logtest.NewGlobal()

	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.String("beacon-node-host", baseurl, "")
	set.String("path", file, "")
	set.Bool("confirm", true, "")
	set.Bool("accept-terms-of-use", true, "")
	assert.NoError(t, set.Set("beacon-node-host", baseurl))
	assert.NoError(t, set.Set("path", file))
	cliCtx := cli.NewContext(&app, set, nil)

	err = setWithdrawalAddresses(cliCtx)
	require.ErrorContains(t, "did not receive 2xx response from API", err)

	assert.LogsContain(t, hook, "Could not validate SignedBLSToExecutionChange")
}

func TestCallWithdrawalEndpoint_ForkBeforeCapella(t *testing.T) {
	file := "./testdata/change-operations.json"
	baseurl := "127.0.0.1:3500"
	l, err := net.Listen("tcp", baseurl)
	require.NoError(t, err)
	srv := httptest.NewUnstartedServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(200)
		w.Header().Set("Content-Type", "application/json")
		if r.RequestURI == "/eth/v1/beacon/states/head/fork" {

			err := json.NewEncoder(w).Encode(&beacon.GetStateForkResponse{
				Data: &shared.Fork{
					PreviousVersion: hexutil.Encode(params.BeaconConfig().BellatrixForkVersion),
					CurrentVersion:  hexutil.Encode(params.BeaconConfig().BellatrixForkVersion),
					Epoch:           "1000",
				},
				ExecutionOptimistic: false,
				Finalized:           true,
			})
			require.NoError(t, err)
		} else if r.RequestURI == "/eth/v1/config/spec" {
			m := make(map[string]string)
			m["CAPELLA_FORK_EPOCH"] = "1350"
			err := json.NewEncoder(w).Encode(&config.GetSpecResponse{
				Data: m,
			})
			require.NoError(t, err)
		}
	}))
	err = srv.Listener.Close()
	require.NoError(t, err)
	srv.Listener = l
	srv.Start()
	defer srv.Close()

	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.String("beacon-node-host", baseurl, "")
	set.String("path", file, "")
	set.Bool("confirm", true, "")
	set.Bool("accept-terms-of-use", true, "")
	assert.NoError(t, set.Set("beacon-node-host", baseurl))
	assert.NoError(t, set.Set("path", file))
	cliCtx := cli.NewContext(&app, set, nil)

	err = setWithdrawalAddresses(cliCtx)
	require.ErrorContains(t, "setting withdrawals using the BLStoExecutionChange endpoint is only available after the Capella/Shanghai hard fork", err)
}

func TestVerifyWithdrawal_Multiple(t *testing.T) {
	file := "./testdata/change-operations-multiple.json"
	baseurl := "127.0.0.1:3500"
	l, err := net.Listen("tcp", baseurl)
	require.NoError(t, err)
	srv := httptest.NewUnstartedServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(200)
		w.Header().Set("Content-Type", "application/json")
		if r.Method == http.MethodGet {
			b, err := os.ReadFile(filepath.Clean(file))
			require.NoError(t, err)
			var to []*shared.SignedBLSToExecutionChange
			err = json.Unmarshal(b, &to)
			require.NoError(t, err)
			err = json.NewEncoder(w).Encode(&beacon.BLSToExecutionChangesPoolResponse{
				Data: to,
			})
			require.NoError(t, err)
		}
	}))
	err = srv.Listener.Close()
	require.NoError(t, err)
	srv.Listener = l
	srv.Start()
	defer srv.Close()
	hook := logtest.NewGlobal()

	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.String("beacon-node-host", baseurl, "")
	set.String("path", file, "")
	set.Bool("confirm", true, "")
	set.Bool("accept-terms-of-use", true, "")
	set.Bool("verify-only", true, "")
	assert.NoError(t, set.Set("beacon-node-host", baseurl))
	assert.NoError(t, set.Set("path", file))
	cliCtx := cli.NewContext(&app, set, nil)

	err = verifyWithdrawalsInPool(cliCtx)
	require.NoError(t, err)
	assert.LogsContain(t, hook, "All (total:2) signed withdrawal messages were found in the pool.")
	assert.LogsDoNotContain(t, hook, "set withdrawal address message not found in the node's operations pool.")
}
