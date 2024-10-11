package rpc

import (
	"archive/zip"
	"bytes"
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"path/filepath"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/golang/mock/gomock"
	"github.com/prysmaticlabs/prysm/v4/api"
	"github.com/prysmaticlabs/prysm/v4/cmd/validator/flags"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	validatormock "github.com/prysmaticlabs/prysm/v4/testing/validator-mock"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/iface"
	mock "github.com/prysmaticlabs/prysm/v4/validator/accounts/testing"
	"github.com/prysmaticlabs/prysm/v4/validator/client"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager/derived"
	constant "github.com/prysmaticlabs/prysm/v4/validator/testing"
	"google.golang.org/protobuf/types/known/timestamppb"
)

var (
	defaultWalletPath = filepath.Join(flags.DefaultValidatorDir(), flags.WalletDefaultDirName)
)

func TestServer_ListAccounts(t *testing.T) {
	ctx := context.Background()
	localWalletDir := setupWalletDir(t)
	defaultWalletPath = localWalletDir
	// We attempt to create the wallet.
	opts := []accounts.Option{
		accounts.WithWalletDir(defaultWalletPath),
		accounts.WithKeymanagerType(keymanager.Derived),
		accounts.WithWalletPassword(strongPass),
		accounts.WithSkipMnemonicConfirm(true),
	}
	acc, err := accounts.NewCLIManager(opts...)
	require.NoError(t, err)
	w, err := acc.WalletCreate(ctx)
	require.NoError(t, err)
	km, err := w.InitializeKeymanager(ctx, iface.InitKeymanagerConfig{ListenForChanges: false})
	require.NoError(t, err)
	vs, err := client.NewValidatorService(ctx, &client.Config{
		Wallet: w,
		Validator: &mock.Validator{
			Km: km,
		},
	})
	require.NoError(t, err)
	s := &Server{
		walletInitialized: true,
		wallet:            w,
		validatorService:  vs,
	}
	numAccounts := 50
	dr, ok := km.(*derived.Keymanager)
	require.Equal(t, true, ok)
	err = dr.RecoverAccountsFromMnemonic(ctx, constant.TestMnemonic, derived.DefaultMnemonicLanguage, "", numAccounts)
	require.NoError(t, err)
	req := httptest.NewRequest(http.MethodGet, fmt.Sprintf(api.WebUrlPrefix+"accounts?page_size=%d", int32(numAccounts)), nil)
	wr := httptest.NewRecorder()
	wr.Body = &bytes.Buffer{}
	s.ListAccounts(wr, req)
	require.Equal(t, http.StatusOK, wr.Code)
	resp := &ListAccountsResponse{}
	require.NoError(t, json.Unmarshal(wr.Body.Bytes(), resp))
	require.Equal(t, len(resp.Accounts), numAccounts)

	tests := []struct {
		PageSize  int
		PageToken string
		All       bool
		res       *ListAccountsResponse
	}{
		{

			PageSize: 5,
			res: &ListAccountsResponse{
				Accounts:      resp.Accounts[0:5],
				NextPageToken: "1",
				TotalSize:     int32(numAccounts),
			},
		},
		{

			PageSize:  5,
			PageToken: "1",
			res: &ListAccountsResponse{
				Accounts:      resp.Accounts[5:10],
				NextPageToken: "2",
				TotalSize:     int32(numAccounts),
			},
		},
		{

			All: true,
			res: &ListAccountsResponse{
				Accounts:      resp.Accounts,
				NextPageToken: "",
				TotalSize:     int32(numAccounts),
			},
		},
	}
	for _, test := range tests {
		url := api.WebUrlPrefix + "accounts"
		if test.PageSize != 0 || test.PageToken != "" || test.All {
			url = url + "?"
		}
		if test.All {
			url = url + "all=true"
		} else {
			if test.PageSize != 0 {
				url = url + fmt.Sprintf("page_size=%d", test.PageSize)
			}
			if test.PageToken != "" {
				url = url + fmt.Sprintf("&page_token=%s", test.PageToken)
			}
		}

		req = httptest.NewRequest(http.MethodGet, url, nil)
		wr = httptest.NewRecorder()
		wr.Body = &bytes.Buffer{}
		s.ListAccounts(wr, req)
		require.Equal(t, http.StatusOK, wr.Code)
		resp = &ListAccountsResponse{}
		require.NoError(t, json.Unmarshal(wr.Body.Bytes(), resp))
		assert.DeepEqual(t, resp, test.res)
	}
}

func TestServer_BackupAccounts(t *testing.T) {
	ctx := context.Background()
	localWalletDir := setupWalletDir(t)
	defaultWalletPath = localWalletDir
	// We attempt to create the wallet.
	opts := []accounts.Option{
		accounts.WithWalletDir(defaultWalletPath),
		accounts.WithKeymanagerType(keymanager.Derived),
		accounts.WithWalletPassword(strongPass),
		accounts.WithSkipMnemonicConfirm(true),
	}
	acc, err := accounts.NewCLIManager(opts...)
	require.NoError(t, err)
	w, err := acc.WalletCreate(ctx)
	require.NoError(t, err)
	km, err := w.InitializeKeymanager(ctx, iface.InitKeymanagerConfig{ListenForChanges: false})
	require.NoError(t, err)
	vs, err := client.NewValidatorService(ctx, &client.Config{
		Wallet: w,
		Validator: &mock.Validator{
			Km: km,
		},
	})
	require.NoError(t, err)
	s := &Server{
		walletInitialized: true,
		wallet:            w,
		validatorService:  vs,
	}
	numAccounts := 50
	dr, ok := km.(*derived.Keymanager)
	require.Equal(t, true, ok)
	err = dr.RecoverAccountsFromMnemonic(ctx, constant.TestMnemonic, derived.DefaultMnemonicLanguage, "", numAccounts)
	require.NoError(t, err)
	req := httptest.NewRequest(http.MethodGet, fmt.Sprintf("/v2/validator/accounts?page_size=%d", int32(numAccounts)), nil)
	wr := httptest.NewRecorder()
	wr.Body = &bytes.Buffer{}
	s.ListAccounts(wr, req)
	require.Equal(t, http.StatusOK, wr.Code)
	resp := &ListAccountsResponse{}
	require.NoError(t, json.Unmarshal(wr.Body.Bytes(), resp))
	require.Equal(t, len(resp.Accounts), numAccounts)

	pubKeys := make([]string, numAccounts)
	for i, aa := range resp.Accounts {
		pubKeys[i] = aa.ValidatingPublicKey
	}
	request := &BackupAccountsRequest{
		PublicKeys:     pubKeys,
		BackupPassword: s.wallet.Password(),
	}
	var buf bytes.Buffer
	err = json.NewEncoder(&buf).Encode(request)
	require.NoError(t, err)
	req = httptest.NewRequest(http.MethodPost, api.WebUrlPrefix+"accounts/backup", &buf)
	wr = httptest.NewRecorder()
	wr.Body = &bytes.Buffer{}
	// We now attempt to backup all public keys from the wallet.
	s.BackupAccounts(wr, req)
	require.Equal(t, http.StatusOK, wr.Code)
	res := &BackupAccountsResponse{}
	require.NoError(t, json.Unmarshal(wr.Body.Bytes(), res))
	// decode the base64 string
	decodedBytes, err := base64.StdEncoding.DecodeString(res.ZipFile)
	require.NoError(t, err)
	// Open a zip archive for reading.
	bu := bytes.NewReader(decodedBytes)
	r, err := zip.NewReader(bu, int64(len(decodedBytes)))
	require.NoError(t, err)
	require.Equal(t, len(pubKeys), len(r.File))

	// Iterate through the files in the archive, checking they
	// match the keystores we wanted to back up.
	for i, f := range r.File {
		keystoreFile, err := f.Open()
		require.NoError(t, err)
		encoded, err := io.ReadAll(keystoreFile)
		if err != nil {
			require.NoError(t, keystoreFile.Close())
			t.Fatal(err)
		}
		keystore := &keymanager.Keystore{}
		if err := json.Unmarshal(encoded, &keystore); err != nil {
			require.NoError(t, keystoreFile.Close())
			t.Fatal(err)
		}
		assert.Equal(t, "0x"+keystore.Pubkey, pubKeys[i])
		require.NoError(t, keystoreFile.Close())
	}
}

func TestServer_VoluntaryExit(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	ctx := context.Background()
	mockValidatorClient := validatormock.NewMockValidatorClient(ctrl)
	mockNodeClient := validatormock.NewMockNodeClient(ctrl)

	mockValidatorClient.EXPECT().
		ValidatorIndex(gomock.Any(), gomock.Any()).
		Return(&ethpb.ValidatorIndexResponse{Index: 0}, nil)

	mockValidatorClient.EXPECT().
		ValidatorIndex(gomock.Any(), gomock.Any()).
		Return(&ethpb.ValidatorIndexResponse{Index: 1}, nil)

	// Any time in the past will suffice
	genesisTime := &timestamppb.Timestamp{
		Seconds: time.Date(2020, 1, 1, 0, 0, 0, 0, time.UTC).Unix(),
	}

	mockNodeClient.EXPECT().
		GetGenesis(gomock.Any(), gomock.Any()).
		Return(&ethpb.Genesis{GenesisTime: genesisTime}, nil)

	mockValidatorClient.EXPECT().
		DomainData(gomock.Any(), gomock.Any()).
		Times(2).
		Return(&ethpb.DomainResponse{SignatureDomain: make([]byte, 32)}, nil)

	mockValidatorClient.EXPECT().
		ProposeExit(gomock.Any(), gomock.AssignableToTypeOf(&ethpb.SignedVoluntaryExit{})).
		Times(2).
		Return(&ethpb.ProposeExitResponse{}, nil)

	localWalletDir := setupWalletDir(t)
	defaultWalletPath = localWalletDir
	// We attempt to create the wallet.
	opts := []accounts.Option{
		accounts.WithWalletDir(defaultWalletPath),
		accounts.WithKeymanagerType(keymanager.Derived),
		accounts.WithWalletPassword(strongPass),
		accounts.WithSkipMnemonicConfirm(true),
	}
	acc, err := accounts.NewCLIManager(opts...)
	require.NoError(t, err)
	w, err := acc.WalletCreate(ctx)
	require.NoError(t, err)
	km, err := w.InitializeKeymanager(ctx, iface.InitKeymanagerConfig{ListenForChanges: false})
	require.NoError(t, err)
	require.NoError(t, err)
	vs, err := client.NewValidatorService(ctx, &client.Config{
		Wallet: w,
		Validator: &mock.Validator{
			Km: km,
		},
	})
	require.NoError(t, err)
	s := &Server{
		walletInitialized:         true,
		wallet:                    w,
		beaconNodeClient:          mockNodeClient,
		beaconNodeValidatorClient: mockValidatorClient,
		validatorService:          vs,
	}
	numAccounts := 2
	dr, ok := km.(*derived.Keymanager)
	require.Equal(t, true, ok)
	err = dr.RecoverAccountsFromMnemonic(ctx, constant.TestMnemonic, derived.DefaultMnemonicLanguage, "", numAccounts)
	require.NoError(t, err)
	pubKeys, err := dr.FetchValidatingPublicKeys(ctx)
	require.NoError(t, err)

	rawPubKeys := make([]string, len(pubKeys))
	for i, key := range pubKeys {
		rawPubKeys[i] = hexutil.Encode(key[:])
	}
	request := &VoluntaryExitRequest{
		PublicKeys: rawPubKeys,
	}
	var buf bytes.Buffer
	err = json.NewEncoder(&buf).Encode(request)
	require.NoError(t, err)
	req := httptest.NewRequest(http.MethodPost, api.WebUrlPrefix+"accounts/voluntary-exit", &buf)
	wr := httptest.NewRecorder()
	wr.Body = &bytes.Buffer{}
	s.VoluntaryExit(wr, req)
	require.Equal(t, http.StatusOK, wr.Code)
	res := &VoluntaryExitResponse{}
	require.NoError(t, json.Unmarshal(wr.Body.Bytes(), res))
	for i := range res.ExitedKeys {
		require.Equal(t, rawPubKeys[i], hexutil.Encode(res.ExitedKeys[i]))
	}

}
