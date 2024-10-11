package rpc

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/golang/mock/gomock"
	"github.com/pkg/errors"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	validatormock "github.com/prysmaticlabs/prysm/v4/testing/validator-mock"
	"google.golang.org/protobuf/types/known/timestamppb"
)

func TestGetBeaconStatus_NotConnected(t *testing.T) {
	ctrl := gomock.NewController(t)
	nodeClient := validatormock.NewMockNodeClient(ctrl)
	nodeClient.EXPECT().GetSyncStatus(
		gomock.Any(), // ctx
		gomock.Any(),
	).Return(nil /*response*/, errors.New("uh oh"))
	srv := &Server{
		beaconNodeClient: nodeClient,
	}
	req := httptest.NewRequest(http.MethodGet, fmt.Sprintf("/v2/validator/beacon/status"), nil)
	wr := httptest.NewRecorder()
	wr.Body = &bytes.Buffer{}
	srv.GetBeaconStatus(wr, req)
	require.Equal(t, http.StatusOK, wr.Code)
	resp := &BeaconStatusResponse{}
	require.NoError(t, json.Unmarshal(wr.Body.Bytes(), resp))
	want := &BeaconStatusResponse{
		BeaconNodeEndpoint: "",
		Connected:          false,
		Syncing:            false,
	}
	assert.DeepEqual(t, want, resp)
}

func TestGetBeaconStatus_OK(t *testing.T) {
	ctrl := gomock.NewController(t)
	nodeClient := validatormock.NewMockNodeClient(ctrl)
	beaconChainClient := validatormock.NewMockBeaconChainClient(ctrl)
	nodeClient.EXPECT().GetSyncStatus(
		gomock.Any(), // ctx
		gomock.Any(),
	).Return(&ethpb.SyncStatus{Syncing: true}, nil)
	timeStamp := timestamppb.New(time.Unix(0, 0))
	nodeClient.EXPECT().GetGenesis(
		gomock.Any(), // ctx
		gomock.Any(),
	).Return(&ethpb.Genesis{
		GenesisTime:            timeStamp,
		DepositContractAddress: []byte("hello"),
	}, nil)
	beaconChainClient.EXPECT().GetChainHead(
		gomock.Any(), // ctx
		gomock.Any(),
	).Return(&ethpb.ChainHead{
		HeadEpoch: 1,
	}, nil)
	srv := &Server{
		beaconNodeClient:  nodeClient,
		beaconChainClient: beaconChainClient,
	}

	req := httptest.NewRequest(http.MethodGet, fmt.Sprintf("/v2/validator/beacon/status"), nil)
	wr := httptest.NewRecorder()
	wr.Body = &bytes.Buffer{}
	srv.GetBeaconStatus(wr, req)
	require.Equal(t, http.StatusOK, wr.Code)
	resp := &BeaconStatusResponse{}
	require.NoError(t, json.Unmarshal(wr.Body.Bytes(), resp))

	want := &BeaconStatusResponse{
		BeaconNodeEndpoint:     "",
		Connected:              true,
		Syncing:                true,
		GenesisTime:            fmt.Sprintf("%d", time.Unix(0, 0).Unix()),
		DepositContractAddress: "0x68656c6c6f",
		ChainHead: &ChainHead{
			HeadSlot:                   "0",
			HeadEpoch:                  "1",
			HeadBlockRoot:              "0x",
			FinalizedSlot:              "0",
			FinalizedEpoch:             "0",
			FinalizedBlockRoot:         "0x",
			JustifiedSlot:              "0",
			JustifiedEpoch:             "0",
			JustifiedBlockRoot:         "0x",
			PreviousJustifiedSlot:      "0",
			PreviousJustifiedEpoch:     "0",
			PreviousJustifiedBlockRoot: "0x",
			OptimisticStatus:           false,
		},
	}
	assert.DeepEqual(t, want, resp)
}
