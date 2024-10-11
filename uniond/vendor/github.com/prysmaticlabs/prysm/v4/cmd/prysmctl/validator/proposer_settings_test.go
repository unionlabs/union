package validator

import (
	"encoding/json"
	"flag"
	"fmt"
	"net"
	"net/http"
	"net/http/httptest"
	"os"
	"strings"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/rpc"
	logtest "github.com/sirupsen/logrus/hooks/test"
	"github.com/urfave/cli/v2"
)

func getValidatorHappyPathTestServer(t *testing.T) *httptest.Server {
	key1 := "0x855ae9c6184d6edd46351b375f16f541b2d33b0ed0da9be4571b13938588aee840ba606a946f0e8023ae3a4b2a43b4d4"
	key2 := "0x844ae9c6184d6edd46351b375f16f541b2d33b0ed0da9be4571b13938588aee840ba606a946f0e8023ae3a4b2a43b4d4"
	address1 := "0xb698D697092822185bF0311052215d5B5e1F3944"
	return httptest.NewUnstartedServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(200)
		w.Header().Set("Content-Type", "application/json")
		if r.Method == http.MethodGet {
			if r.RequestURI == "/eth/v1/keystores" {
				err := json.NewEncoder(w).Encode(&rpc.ListKeystoresResponse{
					Data: []*rpc.Keystore{
						{
							ValidatingPubkey: key1,
						},
						{
							ValidatingPubkey: key2,
						},
					},
				})
				require.NoError(t, err)
			} else if r.RequestURI == "/eth/v1/remotekeys" {
				err := json.NewEncoder(w).Encode(&rpc.ListRemoteKeysResponse{
					Data: []*rpc.RemoteKey{
						{
							Pubkey: key1,
						},
					},
				})
				require.NoError(t, err)
			} else if r.RequestURI[strings.LastIndex(r.RequestURI, "/")+1:] == "feerecipient" {
				pathSeg := strings.Split(r.RequestURI, "/")
				validatorKey := pathSeg[len(pathSeg)-2]
				feeMap := map[string]string{
					key1: address1,
					key2: address1,
				}
				address, ok := feeMap[validatorKey]
				require.Equal(t, ok, true)
				err := json.NewEncoder(w).Encode(&rpc.GetFeeRecipientByPubkeyResponse{
					Data: &rpc.FeeRecipient{
						Pubkey:     validatorKey,
						Ethaddress: address,
					},
				})
				require.NoError(t, err)
			}
		}
	}))
}

func TestGetProposerSettings(t *testing.T) {
	file := "./testdata/settings.json"
	baseurl := "127.0.0.1:3500"
	l, err := net.Listen("tcp", baseurl)
	require.NoError(t, err)
	srv := getValidatorHappyPathTestServer(t)
	err = srv.Listener.Close()
	require.NoError(t, err)
	srv.Listener = l
	srv.Start()
	defer srv.Close()
	hook := logtest.NewGlobal()
	defaultfeerecipient := "0xb698D697092822185bF0311052215d5B5e1F3944"
	token := "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.e30.VXjrSItV_Kmwg_XilpscyPm2SPIsstytYLtr_AuJI8I"
	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.String("validator-host", baseurl, "")
	set.String("output-proposer-settings-path", file, "")
	set.String("default-fee-recipient", defaultfeerecipient, "")
	set.String("token", token, "")
	set.Bool("with-builder", true, "")
	assert.NoError(t, set.Set("validator-host", baseurl))
	assert.NoError(t, set.Set("output-proposer-settings-path", file))
	assert.NoError(t, set.Set("default-fee-recipient", defaultfeerecipient))
	assert.NoError(t, set.Set("token", token))

	cliCtx := cli.NewContext(&app, set, nil)

	err = getProposerSettings(cliCtx, os.Stdin)
	require.NoError(t, err)
	assert.LogsContain(t, hook, fmt.Sprintf("fee recipient is set to %s", defaultfeerecipient))
	assert.LogsContain(t, hook, "Successfully created")
	// clean up created file
	err = os.Remove(file)
	require.NoError(t, err)
}

func TestValidateValidateIsExecutionAddress(t *testing.T) {
	t.Run("Happy Path", func(t *testing.T) {
		err := validateIsExecutionAddress("0xb698D697092822185bF0311052215d5B5e1F3933")
		require.NoError(t, err)
	})
	t.Run("Too Long", func(t *testing.T) {
		err := validateIsExecutionAddress("0xb698D697092822185bF0311052215d5B5e1F39331")
		require.ErrorContains(t, "no default address entered", err)
	})
	t.Run("Too Short", func(t *testing.T) {
		err := validateIsExecutionAddress("0xb698D697092822185bF0311052215d5B5e1F393")
		require.ErrorContains(t, "no default address entered", err)
	})
	t.Run("Not a hex", func(t *testing.T) {
		err := validateIsExecutionAddress("b698D697092822185bF0311052215d5B5e1F393310")
		require.ErrorContains(t, "no default address entered", err)
	})
}
