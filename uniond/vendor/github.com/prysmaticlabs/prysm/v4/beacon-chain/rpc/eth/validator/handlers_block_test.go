package validator

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/golang/mock/gomock"
	"github.com/prysmaticlabs/prysm/v4/api"
	blockchainTesting "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/rewards"
	rewardtesting "github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/rewards/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	rpctesting "github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared/testing"
	mockSync "github.com/prysmaticlabs/prysm/v4/beacon-chain/sync/initial-sync/testing"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	mock2 "github.com/prysmaticlabs/prysm/v4/testing/mock"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestProduceBlockV2(t *testing.T) {
	ctrl := gomock.NewController(t)
	randao := "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
	graffiti := "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2"
	bRandao, err := hexutil.Decode(randao)
	require.NoError(t, err)
	bGraffiti, err := hexutil.Decode(graffiti)
	require.NoError(t, err)
	chainService := &blockchainTesting.ChainService{}
	syncChecker := &mockSync.Sync{IsSyncing: false}
	rewardFetcher := &rewardtesting.MockBlockRewardFetcher{Rewards: &rewards.BlockRewards{Total: "10"}}

	t.Run("Phase 0", func(t *testing.T) {
		var block *shared.SignedBeaconBlock
		err = json.Unmarshal([]byte(rpctesting.Phase0Block), &block)
		require.NoError(t, err)
		jsonBytes, err := json.Marshal(block.Message)
		require.NoError(t, err)

		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: true,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				return block.Message.ToGeneric()
			}())
		server := &Server{
			V1Alpha1Server: v1alpha1Server,
			SyncChecker:    syncChecker,
		}

		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v2/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV2(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		want := fmt.Sprintf(`{"version":"phase0","execution_payload_blinded":false,"execution_payload_value":"","consensus_block_value":"","data":%s}`, string(jsonBytes))
		body := strings.ReplaceAll(writer.Body.String(), "\n", "")
		require.Equal(t, want, body)
		require.Equal(t, "phase0", writer.Header().Get(api.VersionHeader))
	})
	t.Run("Altair", func(t *testing.T) {
		var block *shared.SignedBeaconBlockAltair
		err = json.Unmarshal([]byte(rpctesting.AltairBlock), &block)
		require.NoError(t, err)
		jsonBytes, err := json.Marshal(block.Message)
		require.NoError(t, err)

		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: true,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				return block.Message.ToGeneric()
			}())
		server := &Server{
			V1Alpha1Server:     v1alpha1Server,
			SyncChecker:        syncChecker,
			BlockRewardFetcher: rewardFetcher,
		}

		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v2/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV2(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		want := fmt.Sprintf(`{"version":"altair","execution_payload_blinded":false,"execution_payload_value":"","consensus_block_value":"10","data":%s}`, string(jsonBytes))
		body := strings.ReplaceAll(writer.Body.String(), "\n", "")
		require.Equal(t, want, body)
		require.Equal(t, "altair", writer.Header().Get(api.VersionHeader))
	})
	t.Run("Bellatrix", func(t *testing.T) {
		var block *shared.SignedBeaconBlockBellatrix
		err = json.Unmarshal([]byte(rpctesting.BellatrixBlock), &block)
		require.NoError(t, err)
		jsonBytes, err := json.Marshal(block.Message)
		require.NoError(t, err)

		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: true,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				b, err := block.Message.ToGeneric()
				require.NoError(t, err)
				b.PayloadValue = "2000"
				return b, nil
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}

		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v2/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV2(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		want := fmt.Sprintf(`{"version":"bellatrix","execution_payload_blinded":false,"execution_payload_value":"2000","consensus_block_value":"10","data":%s}`, string(jsonBytes))
		body := strings.ReplaceAll(writer.Body.String(), "\n", "")
		require.Equal(t, want, body)
		require.Equal(t, "bellatrix", writer.Header().Get(api.VersionHeader))
	})
	t.Run("BlindedBellatrix", func(t *testing.T) {
		var block *shared.SignedBlindedBeaconBlockBellatrix
		err = json.Unmarshal([]byte(rpctesting.BlindedBellatrixBlock), &block)
		require.NoError(t, err)

		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: true,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				return block.Message.ToGeneric()
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}

		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v2/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV2(writer, request)
		assert.Equal(t, http.StatusInternalServerError, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusInternalServerError, e.Code)
		assert.StringContains(t, "Prepared block is blinded", e.Message)
	})
	t.Run("Capella", func(t *testing.T) {
		var block *shared.SignedBeaconBlockCapella
		err = json.Unmarshal([]byte(rpctesting.CapellaBlock), &block)
		require.NoError(t, err)
		jsonBytes, err := json.Marshal(block.Message)
		require.NoError(t, err)

		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: true,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				b, err := block.Message.ToGeneric()
				require.NoError(t, err)
				b.PayloadValue = "2000"
				return b, nil
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}

		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v2/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV2(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		want := fmt.Sprintf(`{"version":"capella","execution_payload_blinded":false,"execution_payload_value":"2000","consensus_block_value":"10","data":%s}`, string(jsonBytes))
		body := strings.ReplaceAll(writer.Body.String(), "\n", "")
		require.Equal(t, want, body)
		require.Equal(t, "capella", writer.Header().Get(api.VersionHeader))
	})
	t.Run("Blinded Capella", func(t *testing.T) {
		var block *shared.SignedBlindedBeaconBlockCapella
		err = json.Unmarshal([]byte(rpctesting.BlindedCapellaBlock), &block)
		require.NoError(t, err)

		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: true,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				return block.Message.ToGeneric()
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}

		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v2/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV2(writer, request)
		assert.Equal(t, http.StatusInternalServerError, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusInternalServerError, e.Code)
		assert.StringContains(t, "Prepared block is blinded", e.Message)
	})
	t.Run("Deneb", func(t *testing.T) {
		var block *shared.SignedBeaconBlockContentsDeneb
		err = json.Unmarshal([]byte(rpctesting.DenebBlockContents), &block)
		require.NoError(t, err)
		jsonBytes, err := json.Marshal(block.ToUnsigned())
		require.NoError(t, err)

		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: true,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				b, err := block.ToUnsigned().ToGeneric()
				require.NoError(t, err)
				b.PayloadValue = "2000"
				return b, nil
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}

		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v2/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV2(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		want := fmt.Sprintf(`{"version":"deneb","execution_payload_blinded":false,"execution_payload_value":"2000","consensus_block_value":"10","data":%s}`, string(jsonBytes))
		body := strings.ReplaceAll(writer.Body.String(), "\n", "")
		require.Equal(t, want, body)
		require.Equal(t, "deneb", writer.Header().Get(api.VersionHeader))
	})
	t.Run("Blinded Deneb", func(t *testing.T) {
		var block *shared.SignedBlindedBeaconBlockDeneb
		err = json.Unmarshal([]byte(rpctesting.BlindedDenebBlock), &block)
		require.NoError(t, err)

		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: true,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				return block.Message.ToGeneric()
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}

		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v2/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV2(writer, request)
		assert.Equal(t, http.StatusInternalServerError, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusInternalServerError, e.Code)
		assert.StringContains(t, "Prepared block is blinded", e.Message)
	})
	t.Run("invalid query parameter slot empty", func(t *testing.T) {
		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		server := &Server{
			V1Alpha1Server: v1alpha1Server,
			SyncChecker:    syncChecker,
		}
		request := httptest.NewRequest(http.MethodGet, "http://foo.example/eth/v2/validator/blocks/", nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV2(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		assert.Equal(t, true, strings.Contains(writer.Body.String(), "slot is required"))
	})
	t.Run("invalid query parameter slot invalid", func(t *testing.T) {
		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		server := &Server{
			V1Alpha1Server: v1alpha1Server,
			SyncChecker:    syncChecker,
		}
		request := httptest.NewRequest(http.MethodGet, "http://foo.example/eth/v2/validator/blocks/asdfsad", nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV2(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		assert.Equal(t, true, strings.Contains(writer.Body.String(), "slot is invalid"))
	})
	t.Run("invalid query parameter randao_reveal invalid", func(t *testing.T) {
		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		server := &Server{
			V1Alpha1Server: v1alpha1Server,
			SyncChecker:    syncChecker,
		}
		request := httptest.NewRequest(http.MethodGet, "http://foo.example/eth/v2/validator/blocks/1?randao_reveal=0x213123", nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV2(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
	})
	t.Run("syncing", func(t *testing.T) {
		chainService := &blockchainTesting.ChainService{}
		server := &Server{
			SyncChecker:           &mockSync.Sync{IsSyncing: true},
			HeadFetcher:           chainService,
			TimeFetcher:           chainService,
			OptimisticModeFetcher: chainService,
		}

		request := httptest.NewRequest(http.MethodPost, "http://foo.example", bytes.NewReader([]byte("foo")))
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV2(writer, request)
		assert.Equal(t, http.StatusServiceUnavailable, writer.Code)
		assert.Equal(t, true, strings.Contains(writer.Body.String(), "Beacon node is currently syncing and not serving request on that endpoint"))
	})
}

func TestProduceBlockV2SSZ(t *testing.T) {
	ctrl := gomock.NewController(t)
	randao := "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
	graffiti := "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2"
	bRandao, err := hexutil.Decode(randao)
	require.NoError(t, err)
	bGraffiti, err := hexutil.Decode(graffiti)
	require.NoError(t, err)
	chainService := &blockchainTesting.ChainService{}
	syncChecker := &mockSync.Sync{IsSyncing: false}
	rewardFetcher := &rewardtesting.MockBlockRewardFetcher{Rewards: &rewards.BlockRewards{Total: "10"}}

	t.Run("Phase 0", func(t *testing.T) {
		var block *shared.SignedBeaconBlock
		err = json.Unmarshal([]byte(rpctesting.Phase0Block), &block)
		require.NoError(t, err)

		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: true,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				return block.Message.ToGeneric()
			}())
		server := &Server{
			V1Alpha1Server: v1alpha1Server,
			SyncChecker:    syncChecker,
		}

		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v2/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		request.Header.Set("Accept", api.OctetStreamMediaType)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV2(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		g, err := block.ToGeneric()
		require.NoError(t, err)
		bl, ok := g.Block.(*eth.GenericSignedBeaconBlock_Phase0)
		require.Equal(t, true, ok)
		ssz, err := bl.Phase0.Block.MarshalSSZ()
		require.NoError(t, err)
		require.Equal(t, string(ssz), writer.Body.String())
		require.Equal(t, "phase0", writer.Header().Get(api.VersionHeader))
	})
	t.Run("Altair", func(t *testing.T) {
		var block *shared.SignedBeaconBlockAltair
		err = json.Unmarshal([]byte(rpctesting.AltairBlock), &block)
		require.NoError(t, err)

		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: true,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				return block.Message.ToGeneric()
			}())

		server := &Server{
			V1Alpha1Server:     v1alpha1Server,
			SyncChecker:        syncChecker,
			BlockRewardFetcher: rewardFetcher,
		}

		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v2/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		request.Header.Set("Accept", api.OctetStreamMediaType)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV2(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		g, err := block.ToGeneric()
		require.NoError(t, err)
		bl, ok := g.Block.(*eth.GenericSignedBeaconBlock_Altair)
		require.Equal(t, true, ok)
		ssz, err := bl.Altair.Block.MarshalSSZ()
		require.NoError(t, err)
		require.Equal(t, string(ssz), writer.Body.String())
		require.Equal(t, "altair", writer.Header().Get(api.VersionHeader))
	})
	t.Run("Bellatrix", func(t *testing.T) {
		var block *shared.SignedBeaconBlockBellatrix
		err = json.Unmarshal([]byte(rpctesting.BellatrixBlock), &block)
		require.NoError(t, err)

		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: true,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				return block.Message.ToGeneric()
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}

		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v2/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		request.Header.Set("Accept", api.OctetStreamMediaType)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV2(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		g, err := block.ToGeneric()
		require.NoError(t, err)
		bl, ok := g.Block.(*eth.GenericSignedBeaconBlock_Bellatrix)
		require.Equal(t, true, ok)
		ssz, err := bl.Bellatrix.Block.MarshalSSZ()
		require.NoError(t, err)
		require.Equal(t, string(ssz), writer.Body.String())
		require.Equal(t, "bellatrix", writer.Header().Get(api.VersionHeader))
	})
	t.Run("BlindedBellatrix", func(t *testing.T) {
		var block *shared.SignedBlindedBeaconBlockBellatrix
		err = json.Unmarshal([]byte(rpctesting.BlindedBellatrixBlock), &block)
		require.NoError(t, err)

		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: true,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				return block.Message.ToGeneric()
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}

		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v2/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		request.Header.Set("Accept", api.OctetStreamMediaType)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV2(writer, request)
		assert.Equal(t, http.StatusInternalServerError, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusInternalServerError, e.Code)
		assert.StringContains(t, "Prepared block is blinded", e.Message)
	})
	t.Run("Capella", func(t *testing.T) {
		var block *shared.SignedBeaconBlockCapella
		err = json.Unmarshal([]byte(rpctesting.CapellaBlock), &block)
		require.NoError(t, err)

		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: true,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				return block.Message.ToGeneric()
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}

		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v2/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		request.Header.Set("Accept", api.OctetStreamMediaType)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV2(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		g, err := block.ToGeneric()
		require.NoError(t, err)
		bl, ok := g.Block.(*eth.GenericSignedBeaconBlock_Capella)
		require.Equal(t, true, ok)
		ssz, err := bl.Capella.Block.MarshalSSZ()
		require.NoError(t, err)
		require.Equal(t, string(ssz), writer.Body.String())
		require.Equal(t, "capella", writer.Header().Get(api.VersionHeader))
	})
	t.Run("Blinded Capella", func(t *testing.T) {
		var block *shared.SignedBlindedBeaconBlockCapella
		err = json.Unmarshal([]byte(rpctesting.BlindedCapellaBlock), &block)
		require.NoError(t, err)

		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: true,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				g, err := block.Message.ToGeneric()
				require.NoError(t, err)
				g.PayloadValue = "2000"
				return g, err
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}

		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v2/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		request.Header.Set("Accept", api.OctetStreamMediaType)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV2(writer, request)
		assert.Equal(t, http.StatusInternalServerError, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusInternalServerError, e.Code)
		assert.StringContains(t, "Prepared block is blinded", e.Message)
	})
	t.Run("Deneb", func(t *testing.T) {
		var block *shared.SignedBeaconBlockContentsDeneb
		err = json.Unmarshal([]byte(rpctesting.DenebBlockContents), &block)
		require.NoError(t, err)

		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: true,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				return block.ToUnsigned().ToGeneric()
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}

		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v2/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		request.Header.Set("Accept", api.OctetStreamMediaType)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV2(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		g, err := block.ToUnsigned().ToGeneric()
		require.NoError(t, err)
		bl, ok := g.Block.(*eth.GenericBeaconBlock_Deneb)
		require.Equal(t, true, ok)
		ssz, err := bl.Deneb.MarshalSSZ()
		require.NoError(t, err)
		require.Equal(t, string(ssz), writer.Body.String())
		require.Equal(t, "deneb", writer.Header().Get(api.VersionHeader))
	})
	t.Run("Blinded Deneb", func(t *testing.T) {
		var block *shared.SignedBlindedBeaconBlockDeneb
		err = json.Unmarshal([]byte(rpctesting.BlindedDenebBlock), &block)
		require.NoError(t, err)

		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: true,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				return block.Message.ToGeneric()
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}

		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v2/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		request.Header.Set("Accept", api.OctetStreamMediaType)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV2(writer, request)
		assert.Equal(t, http.StatusInternalServerError, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusInternalServerError, e.Code)
		assert.StringContains(t, "Prepared block is blinded", e.Message)
	})
}

func TestProduceBlindedBlock(t *testing.T) {
	ctrl := gomock.NewController(t)
	randao := "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
	graffiti := "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2"
	bRandao, err := hexutil.Decode(randao)
	require.NoError(t, err)
	bGraffiti, err := hexutil.Decode(graffiti)
	require.NoError(t, err)
	chainService := &blockchainTesting.ChainService{}
	syncChecker := &mockSync.Sync{IsSyncing: false}
	rewardFetcher := &rewardtesting.MockBlockRewardFetcher{Rewards: &rewards.BlockRewards{Total: "10"}}

	t.Run("Phase 0", func(t *testing.T) {
		var block *shared.SignedBeaconBlock
		err = json.Unmarshal([]byte(rpctesting.Phase0Block), &block)
		require.NoError(t, err)

		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: false,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				return block.Message.ToGeneric()
			}())
		server := &Server{
			V1Alpha1Server: v1alpha1Server,
			SyncChecker:    syncChecker,
		}

		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v1/validator/blinded_blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlindedBlock(writer, request)
		assert.Equal(t, http.StatusInternalServerError, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusInternalServerError, e.Code)
		assert.StringContains(t, "Prepared block is not blinded", e.Message)
	})
	t.Run("Altair", func(t *testing.T) {
		var block *shared.SignedBeaconBlockAltair
		err = json.Unmarshal([]byte(rpctesting.AltairBlock), &block)
		require.NoError(t, err)

		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: false,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {

				return block.Message.ToGeneric()
			}())
		server := &Server{
			V1Alpha1Server:     v1alpha1Server,
			SyncChecker:        &mockSync.Sync{IsSyncing: false},
			BlockRewardFetcher: rewardFetcher,
		}

		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v1/validator/blinded_blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlindedBlock(writer, request)
		assert.Equal(t, http.StatusInternalServerError, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusInternalServerError, e.Code)
		assert.StringContains(t, "Prepared block is not blinded", e.Message)
	})
	t.Run("Bellatrix", func(t *testing.T) {
		var block *shared.SignedBeaconBlockBellatrix
		err = json.Unmarshal([]byte(rpctesting.BellatrixBlock), &block)
		require.NoError(t, err)

		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: false,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				return block.Message.ToGeneric()
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}

		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v1/validator/blinded_blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlindedBlock(writer, request)
		assert.Equal(t, http.StatusInternalServerError, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusInternalServerError, e.Code)
		assert.StringContains(t, "Prepared block is not blinded", e.Message)
	})
	t.Run("BlindedBellatrix", func(t *testing.T) {
		var block *shared.SignedBlindedBeaconBlockBellatrix
		err = json.Unmarshal([]byte(rpctesting.BlindedBellatrixBlock), &block)
		require.NoError(t, err)
		jsonBytes, err := json.Marshal(block.Message)
		require.NoError(t, err)

		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: false,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				b, err := block.Message.ToGeneric()
				require.NoError(t, err)
				b.PayloadValue = "2000"
				return b, nil
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}

		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v1/validator/blinded_blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlindedBlock(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		want := fmt.Sprintf(`{"version":"bellatrix","execution_payload_blinded":true,"execution_payload_value":"2000","consensus_block_value":"10","data":%s}`, string(jsonBytes))
		body := strings.ReplaceAll(writer.Body.String(), "\n", "")
		require.Equal(t, want, body)
		require.Equal(t, "bellatrix", writer.Header().Get(api.VersionHeader))
	})
	t.Run("Capella", func(t *testing.T) {
		var block *shared.SignedBeaconBlockCapella
		err = json.Unmarshal([]byte(rpctesting.CapellaBlock), &block)
		require.NoError(t, err)

		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: false,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				return block.Message.ToGeneric()
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}

		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v1/validator/blinded_blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlindedBlock(writer, request)
		assert.Equal(t, http.StatusInternalServerError, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusInternalServerError, e.Code)
		assert.StringContains(t, "Prepared block is not blinded", e.Message)
	})
	t.Run("Blinded Capella", func(t *testing.T) {
		var block *shared.SignedBlindedBeaconBlockCapella
		err = json.Unmarshal([]byte(rpctesting.BlindedCapellaBlock), &block)
		require.NoError(t, err)
		jsonBytes, err := json.Marshal(block.Message)
		require.NoError(t, err)

		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: false,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				g, err := block.Message.ToGeneric()
				require.NoError(t, err)
				g.PayloadValue = "2000"
				return g, err
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}

		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v1/validator/blinded_blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlindedBlock(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		want := fmt.Sprintf(`{"version":"capella","execution_payload_blinded":true,"execution_payload_value":"2000","consensus_block_value":"10","data":%s}`, string(jsonBytes))
		body := strings.ReplaceAll(writer.Body.String(), "\n", "")
		require.Equal(t, want, body)
		require.Equal(t, "capella", writer.Header().Get(api.VersionHeader))
	})
	t.Run("Deneb", func(t *testing.T) {
		var block *shared.SignedBeaconBlockContentsDeneb
		err = json.Unmarshal([]byte(rpctesting.DenebBlockContents), &block)
		require.NoError(t, err)

		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: false,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				return block.ToUnsigned().ToGeneric()
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}

		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v1/validator/blinded_blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlindedBlock(writer, request)
		assert.Equal(t, http.StatusInternalServerError, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusInternalServerError, e.Code)
		assert.StringContains(t, "Prepared block is not blinded", e.Message)
	})
	t.Run("Blinded Deneb", func(t *testing.T) {
		var block *shared.SignedBlindedBeaconBlockDeneb
		err = json.Unmarshal([]byte(rpctesting.BlindedDenebBlock), &block)
		require.NoError(t, err)
		jsonBytes, err := json.Marshal(block.Message)
		require.NoError(t, err)

		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: false,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				b, err := block.Message.ToGeneric()
				require.NoError(t, err)
				b.PayloadValue = "2000"
				return b, nil
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}

		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v1/validator/blinded_blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlindedBlock(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		want := fmt.Sprintf(`{"version":"deneb","execution_payload_blinded":true,"execution_payload_value":"2000","consensus_block_value":"10","data":%s}`, string(jsonBytes))
		body := strings.ReplaceAll(writer.Body.String(), "\n", "")
		require.Equal(t, want, body)
		require.Equal(t, "deneb", writer.Header().Get(api.VersionHeader))
	})
	t.Run("invalid query parameter slot empty", func(t *testing.T) {
		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		server := &Server{
			V1Alpha1Server: v1alpha1Server,
			SyncChecker:    syncChecker,
		}
		request := httptest.NewRequest(http.MethodGet, "http://foo.example/eth/v1/validator/blinded_blocks/", nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlindedBlock(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		assert.Equal(t, true, strings.Contains(writer.Body.String(), "slot is required"))
	})
	t.Run("invalid query parameter slot invalid", func(t *testing.T) {
		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		server := &Server{
			V1Alpha1Server: v1alpha1Server,
			SyncChecker:    syncChecker,
		}
		request := httptest.NewRequest(http.MethodGet, "http://foo.example/eth/v1/validator/blinded_blocks/asdfsad", nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlindedBlock(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		assert.Equal(t, true, strings.Contains(writer.Body.String(), "slot is invalid"))
	})
	t.Run("invalid query parameter randao_reveal invalid", func(t *testing.T) {
		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		server := &Server{
			V1Alpha1Server: v1alpha1Server,
			SyncChecker:    syncChecker,
		}
		request := httptest.NewRequest(http.MethodGet, "http://foo.example/eth/v1/validator/blinded_blocks/1?randao_reveal=0x213123", nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlindedBlock(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
	})
	t.Run("syncing", func(t *testing.T) {
		chainService := &blockchainTesting.ChainService{}
		server := &Server{
			SyncChecker:           &mockSync.Sync{IsSyncing: true},
			HeadFetcher:           chainService,
			TimeFetcher:           chainService,
			OptimisticModeFetcher: chainService,
		}

		request := httptest.NewRequest(http.MethodPost, "http://foo.example", bytes.NewReader([]byte("foo")))
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlindedBlock(writer, request)
		assert.Equal(t, http.StatusServiceUnavailable, writer.Code)
		assert.Equal(t, true, strings.Contains(writer.Body.String(), "Beacon node is currently syncing and not serving request on that endpoint"))
	})
}

func TestProduceBlockV3(t *testing.T) {
	ctrl := gomock.NewController(t)
	randao := "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
	graffiti := "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2"
	bRandao, err := hexutil.Decode(randao)
	require.NoError(t, err)
	bGraffiti, err := hexutil.Decode(graffiti)
	require.NoError(t, err)
	chainService := &blockchainTesting.ChainService{}
	syncChecker := &mockSync.Sync{IsSyncing: false}
	rewardFetcher := &rewardtesting.MockBlockRewardFetcher{Rewards: &rewards.BlockRewards{Total: "10"}}

	t.Run("Phase 0", func(t *testing.T) {
		var block *shared.SignedBeaconBlock
		err := json.Unmarshal([]byte(rpctesting.Phase0Block), &block)
		require.NoError(t, err)
		jsonBytes, err := json.Marshal(block.Message)
		require.NoError(t, err)
		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: false,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				return block.Message.ToGeneric()
			}())
		server := &Server{
			V1Alpha1Server: v1alpha1Server,
			SyncChecker:    syncChecker,
		}
		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v3/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV3(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		want := fmt.Sprintf(`{"version":"phase0","execution_payload_blinded":false,"execution_payload_value":"","consensus_block_value":"","data":%s}`, string(jsonBytes))
		body := strings.ReplaceAll(writer.Body.String(), "\n", "")
		require.Equal(t, want, body)
		require.Equal(t, "false", writer.Header().Get(api.ExecutionPayloadBlindedHeader))
		require.Equal(t, "", writer.Header().Get(api.ExecutionPayloadValueHeader))
		require.Equal(t, "phase0", writer.Header().Get(api.VersionHeader))
		require.Equal(t, "", writer.Header().Get(api.ConsensusBlockValueHeader))
	})
	t.Run("Altair", func(t *testing.T) {
		var block *shared.SignedBeaconBlockAltair
		err := json.Unmarshal([]byte(rpctesting.AltairBlock), &block)

		require.NoError(t, err)
		jsonBytes, err := json.Marshal(block.Message)
		require.NoError(t, err)
		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: false,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {

				return block.Message.ToGeneric()
			}())
		server := &Server{
			V1Alpha1Server:     v1alpha1Server,
			SyncChecker:        syncChecker,
			BlockRewardFetcher: rewardFetcher,
		}
		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v3/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV3(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		want := fmt.Sprintf(`{"version":"altair","execution_payload_blinded":false,"execution_payload_value":"","consensus_block_value":"10","data":%s}`, string(jsonBytes))
		body := strings.ReplaceAll(writer.Body.String(), "\n", "")
		require.Equal(t, want, body)
		require.Equal(t, "false", writer.Header().Get(api.ExecutionPayloadBlindedHeader))
		require.Equal(t, "", writer.Header().Get(api.ExecutionPayloadValueHeader))
		require.Equal(t, "altair", writer.Header().Get(api.VersionHeader))
		require.Equal(t, "10", writer.Header().Get(api.ConsensusBlockValueHeader))
	})
	t.Run("Bellatrix", func(t *testing.T) {
		var block *shared.SignedBeaconBlockBellatrix
		err := json.Unmarshal([]byte(rpctesting.BellatrixBlock), &block)
		require.NoError(t, err)
		jsonBytes, err := json.Marshal(block.Message)
		require.NoError(t, err)
		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: false,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				b, err := block.Message.ToGeneric()
				require.NoError(t, err)
				b.PayloadValue = "2000"
				return b, nil
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}
		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v3/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV3(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		want := fmt.Sprintf(`{"version":"bellatrix","execution_payload_blinded":false,"execution_payload_value":"2000","consensus_block_value":"10","data":%s}`, string(jsonBytes))
		body := strings.ReplaceAll(writer.Body.String(), "\n", "")
		require.Equal(t, want, body)
		require.Equal(t, "false", writer.Header().Get(api.ExecutionPayloadBlindedHeader))
		require.Equal(t, "2000", writer.Header().Get(api.ExecutionPayloadValueHeader))
		require.Equal(t, "bellatrix", writer.Header().Get(api.VersionHeader))
		require.Equal(t, "10", writer.Header().Get(api.ConsensusBlockValueHeader))
	})
	t.Run("BlindedBellatrix", func(t *testing.T) {
		var block *shared.SignedBlindedBeaconBlockBellatrix
		err := json.Unmarshal([]byte(rpctesting.BlindedBellatrixBlock), &block)
		require.NoError(t, err)
		jsonBytes, err := json.Marshal(block.Message)
		require.NoError(t, err)
		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: false,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				b, err := block.Message.ToGeneric()
				require.NoError(t, err)
				b.PayloadValue = "2000"
				return b, nil
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}
		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v3/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV3(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		want := fmt.Sprintf(`{"version":"bellatrix","execution_payload_blinded":true,"execution_payload_value":"2000","consensus_block_value":"10","data":%s}`, string(jsonBytes))
		body := strings.ReplaceAll(writer.Body.String(), "\n", "")
		require.Equal(t, want, body)
		require.Equal(t, "true", writer.Header().Get(api.ExecutionPayloadBlindedHeader))
		require.Equal(t, "2000", writer.Header().Get(api.ExecutionPayloadValueHeader))
		require.Equal(t, "bellatrix", writer.Header().Get(api.VersionHeader))
		require.Equal(t, "10", writer.Header().Get(api.ConsensusBlockValueHeader))
	})
	t.Run("Capella", func(t *testing.T) {
		var block *shared.SignedBeaconBlockCapella
		err := json.Unmarshal([]byte(rpctesting.CapellaBlock), &block)
		require.NoError(t, err)
		jsonBytes, err := json.Marshal(block.Message)
		require.NoError(t, err)
		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: false,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				b, err := block.Message.ToGeneric()
				require.NoError(t, err)
				b.PayloadValue = "2000"
				return b, nil
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}
		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v3/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV3(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		want := fmt.Sprintf(`{"version":"capella","execution_payload_blinded":false,"execution_payload_value":"2000","consensus_block_value":"10","data":%s}`, string(jsonBytes))
		body := strings.ReplaceAll(writer.Body.String(), "\n", "")
		require.Equal(t, want, body)
		require.Equal(t, "false", writer.Header().Get(api.ExecutionPayloadBlindedHeader))
		require.Equal(t, "2000", writer.Header().Get(api.ExecutionPayloadValueHeader))
		require.Equal(t, "capella", writer.Header().Get(api.VersionHeader))
		require.Equal(t, "10", writer.Header().Get(api.ConsensusBlockValueHeader))
	})
	t.Run("Blinded Capella", func(t *testing.T) {
		var block *shared.SignedBlindedBeaconBlockCapella
		err := json.Unmarshal([]byte(rpctesting.BlindedCapellaBlock), &block)
		require.NoError(t, err)
		jsonBytes, err := json.Marshal(block.Message)
		require.NoError(t, err)
		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: false,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				g, err := block.Message.ToGeneric()
				require.NoError(t, err)
				g.PayloadValue = "2000"
				return g, err
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}
		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v3/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV3(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		want := fmt.Sprintf(`{"version":"capella","execution_payload_blinded":true,"execution_payload_value":"2000","consensus_block_value":"10","data":%s}`, string(jsonBytes))
		body := strings.ReplaceAll(writer.Body.String(), "\n", "")
		require.Equal(t, want, body)
		require.Equal(t, "true", writer.Header().Get(api.ExecutionPayloadBlindedHeader))
		require.Equal(t, "2000", writer.Header().Get(api.ExecutionPayloadValueHeader))
		require.Equal(t, "capella", writer.Header().Get(api.VersionHeader))
		require.Equal(t, "10", writer.Header().Get(api.ConsensusBlockValueHeader))
	})
	t.Run("Deneb", func(t *testing.T) {
		var block *shared.SignedBeaconBlockContentsDeneb
		err := json.Unmarshal([]byte(rpctesting.DenebBlockContents), &block)
		require.NoError(t, err)
		jsonBytes, err := json.Marshal(block.ToUnsigned())
		require.NoError(t, err)
		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: false,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				b, err := block.ToUnsigned().ToGeneric()
				require.NoError(t, err)
				b.PayloadValue = "2000"
				return b, nil
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}
		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v3/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV3(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		want := fmt.Sprintf(`{"version":"deneb","execution_payload_blinded":false,"execution_payload_value":"2000","consensus_block_value":"10","data":%s}`, string(jsonBytes))
		body := strings.ReplaceAll(writer.Body.String(), "\n", "")
		require.Equal(t, want, body)
		require.Equal(t, "false", writer.Header().Get(api.ExecutionPayloadBlindedHeader))
		require.Equal(t, "2000", writer.Header().Get(api.ExecutionPayloadValueHeader))
		require.Equal(t, "deneb", writer.Header().Get(api.VersionHeader))
		require.Equal(t, "10", writer.Header().Get(api.ConsensusBlockValueHeader))
	})
	t.Run("Blinded Deneb", func(t *testing.T) {
		var block *shared.SignedBlindedBeaconBlockDeneb
		err := json.Unmarshal([]byte(rpctesting.BlindedDenebBlock), &block)
		require.NoError(t, err)
		jsonBytes, err := json.Marshal(block.Message)
		require.NoError(t, err)
		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: false,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				b, err := block.Message.ToGeneric()
				require.NoError(t, err)
				b.PayloadValue = "2000"
				return b, nil
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}
		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v3/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV3(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		want := fmt.Sprintf(`{"version":"deneb","execution_payload_blinded":true,"execution_payload_value":"2000","consensus_block_value":"10","data":%s}`, string(jsonBytes))
		body := strings.ReplaceAll(writer.Body.String(), "\n", "")
		require.Equal(t, want, body)
		require.Equal(t, "true", writer.Header().Get(api.ExecutionPayloadBlindedHeader))
		require.Equal(t, "2000", writer.Header().Get(api.ExecutionPayloadValueHeader))
		require.Equal(t, "deneb", writer.Header().Get(api.VersionHeader))
		require.Equal(t, "10", writer.Header().Get(api.ConsensusBlockValueHeader))
	})
	t.Run("invalid query parameter slot empty", func(t *testing.T) {
		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		server := &Server{
			V1Alpha1Server: v1alpha1Server,
			SyncChecker:    &mockSync.Sync{IsSyncing: false},
		}
		request := httptest.NewRequest(http.MethodGet, "http://foo.example/eth/v3/validator/blocks/", nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV3(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		assert.Equal(t, true, strings.Contains(writer.Body.String(), "slot is required"))
	})
	t.Run("invalid query parameter slot invalid", func(t *testing.T) {
		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		server := &Server{
			V1Alpha1Server: v1alpha1Server,
			SyncChecker:    syncChecker,
		}
		request := httptest.NewRequest(http.MethodGet, "http://foo.example/eth/v3/validator/blocks/asdfsad", nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV3(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		assert.Equal(t, true, strings.Contains(writer.Body.String(), "slot is invalid"))
	})
	t.Run("invalid query parameter randao_reveal invalid", func(t *testing.T) {
		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		server := &Server{
			V1Alpha1Server: v1alpha1Server,
			SyncChecker:    syncChecker,
		}
		request := httptest.NewRequest(http.MethodGet, "http://foo.example/eth/v3/validator/blocks/1?randao_reveal=0x213123", nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV3(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
	})
	t.Run("syncing", func(t *testing.T) {
		server := &Server{
			SyncChecker:           &mockSync.Sync{IsSyncing: true},
			HeadFetcher:           chainService,
			TimeFetcher:           chainService,
			OptimisticModeFetcher: chainService,
		}

		request := httptest.NewRequest(http.MethodPost, "http://foo.example", bytes.NewReader([]byte("foo")))
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV3(writer, request)
		assert.Equal(t, http.StatusServiceUnavailable, writer.Code)
		assert.Equal(t, true, strings.Contains(writer.Body.String(), "Beacon node is currently syncing and not serving request on that endpoint"))
	})
}

func TestProduceBlockV3SSZ(t *testing.T) {
	ctrl := gomock.NewController(t)
	randao := "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
	graffiti := "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2"
	bRandao, err := hexutil.Decode(randao)
	require.NoError(t, err)
	bGraffiti, err := hexutil.Decode(graffiti)
	require.NoError(t, err)
	chainService := &blockchainTesting.ChainService{}
	syncChecker := &mockSync.Sync{IsSyncing: false}
	rewardFetcher := &rewardtesting.MockBlockRewardFetcher{Rewards: &rewards.BlockRewards{Total: "10"}}

	t.Run("Phase 0", func(t *testing.T) {
		var block *shared.SignedBeaconBlock
		err := json.Unmarshal([]byte(rpctesting.Phase0Block), &block)
		require.NoError(t, err)
		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: false,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				return block.Message.ToGeneric()
			}())
		server := &Server{
			V1Alpha1Server: v1alpha1Server,
			SyncChecker:    syncChecker,
		}
		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v3/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		request.Header.Set("Accept", api.OctetStreamMediaType)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV3(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		g, err := block.ToGeneric()
		require.NoError(t, err)
		bl, ok := g.Block.(*eth.GenericSignedBeaconBlock_Phase0)
		require.Equal(t, true, ok)
		ssz, err := bl.Phase0.Block.MarshalSSZ()
		require.NoError(t, err)
		require.Equal(t, string(ssz), writer.Body.String())
		require.Equal(t, "false", writer.Header().Get(api.ExecutionPayloadBlindedHeader))
		require.Equal(t, "", writer.Header().Get(api.ExecutionPayloadValueHeader))
		require.Equal(t, "phase0", writer.Header().Get(api.VersionHeader))
		require.Equal(t, "", writer.Header().Get(api.ConsensusBlockValueHeader))
	})
	t.Run("Altair", func(t *testing.T) {
		var block *shared.SignedBeaconBlockAltair
		err := json.Unmarshal([]byte(rpctesting.AltairBlock), &block)
		require.NoError(t, err)
		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: false,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				return block.Message.ToGeneric()
			}())

		server := &Server{
			V1Alpha1Server:     v1alpha1Server,
			SyncChecker:        syncChecker,
			BlockRewardFetcher: rewardFetcher,
		}
		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v3/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		request.Header.Set("Accept", api.OctetStreamMediaType)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV3(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		g, err := block.ToGeneric()
		require.NoError(t, err)
		bl, ok := g.Block.(*eth.GenericSignedBeaconBlock_Altair)
		require.Equal(t, true, ok)
		ssz, err := bl.Altair.Block.MarshalSSZ()
		require.NoError(t, err)
		require.Equal(t, string(ssz), writer.Body.String())
		require.Equal(t, "false", writer.Header().Get(api.ExecutionPayloadBlindedHeader))
		require.Equal(t, "", writer.Header().Get(api.ExecutionPayloadValueHeader))
		require.Equal(t, "altair", writer.Header().Get(api.VersionHeader))
		require.Equal(t, "10", writer.Header().Get(api.ConsensusBlockValueHeader))
	})
	t.Run("Bellatrix", func(t *testing.T) {
		var block *shared.SignedBeaconBlockBellatrix
		err := json.Unmarshal([]byte(rpctesting.BellatrixBlock), &block)
		require.NoError(t, err)
		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: false,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				b, err := block.Message.ToGeneric()
				require.NoError(t, err)
				b.PayloadValue = "2000"
				return b, nil
			}())
		mockChainService := &blockchainTesting.ChainService{}
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: mockChainService,
			BlockRewardFetcher:    rewardFetcher,
		}
		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v3/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		request.Header.Set("Accept", api.OctetStreamMediaType)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV3(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		g, err := block.ToGeneric()
		require.NoError(t, err)
		bl, ok := g.Block.(*eth.GenericSignedBeaconBlock_Bellatrix)
		require.Equal(t, true, ok)
		ssz, err := bl.Bellatrix.Block.MarshalSSZ()
		require.NoError(t, err)
		require.Equal(t, string(ssz), writer.Body.String())
		require.Equal(t, "false", writer.Header().Get(api.ExecutionPayloadBlindedHeader))
		require.Equal(t, "2000", writer.Header().Get(api.ExecutionPayloadValueHeader))
		require.Equal(t, "bellatrix", writer.Header().Get(api.VersionHeader))
		require.Equal(t, "10", writer.Header().Get(api.ConsensusBlockValueHeader))
	})
	t.Run("BlindedBellatrix", func(t *testing.T) {
		var block *shared.SignedBlindedBeaconBlockBellatrix
		err := json.Unmarshal([]byte(rpctesting.BlindedBellatrixBlock), &block)
		require.NoError(t, err)
		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: false,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				b, err := block.Message.ToGeneric()
				require.NoError(t, err)
				b.PayloadValue = "2000"
				return b, nil
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}
		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v3/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		request.Header.Set("Accept", api.OctetStreamMediaType)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV3(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		g, err := block.ToGeneric()
		require.NoError(t, err)
		bl, ok := g.Block.(*eth.GenericSignedBeaconBlock_BlindedBellatrix)
		require.Equal(t, true, ok)
		ssz, err := bl.BlindedBellatrix.Block.MarshalSSZ()
		require.NoError(t, err)
		require.Equal(t, string(ssz), writer.Body.String())
		require.Equal(t, "true", writer.Header().Get(api.ExecutionPayloadBlindedHeader))
		require.Equal(t, "2000", writer.Header().Get(api.ExecutionPayloadValueHeader))
		require.Equal(t, "bellatrix", writer.Header().Get(api.VersionHeader))
		require.Equal(t, "10", writer.Header().Get(api.ConsensusBlockValueHeader))
	})
	t.Run("Capella", func(t *testing.T) {
		var block *shared.SignedBeaconBlockCapella
		err := json.Unmarshal([]byte(rpctesting.CapellaBlock), &block)
		require.NoError(t, err)
		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: false,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				b, err := block.Message.ToGeneric()
				require.NoError(t, err)
				b.PayloadValue = "2000"
				return b, nil
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}
		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v3/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		request.Header.Set("Accept", api.OctetStreamMediaType)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV3(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		g, err := block.ToGeneric()
		require.NoError(t, err)
		bl, ok := g.Block.(*eth.GenericSignedBeaconBlock_Capella)
		require.Equal(t, true, ok)
		ssz, err := bl.Capella.Block.MarshalSSZ()
		require.NoError(t, err)
		require.Equal(t, string(ssz), writer.Body.String())
		require.Equal(t, "false", writer.Header().Get(api.ExecutionPayloadBlindedHeader))
		require.Equal(t, "2000", writer.Header().Get(api.ExecutionPayloadValueHeader))
		require.Equal(t, "capella", writer.Header().Get(api.VersionHeader))
		require.Equal(t, "10", writer.Header().Get(api.ConsensusBlockValueHeader))
	})
	t.Run("Blinded Capella", func(t *testing.T) {
		var block *shared.SignedBlindedBeaconBlockCapella
		err := json.Unmarshal([]byte(rpctesting.BlindedCapellaBlock), &block)
		require.NoError(t, err)
		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: false,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				g, err := block.Message.ToGeneric()
				require.NoError(t, err)
				g.PayloadValue = "2000"
				return g, err
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}
		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v3/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		request.Header.Set("Accept", api.OctetStreamMediaType)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV3(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		g, err := block.ToGeneric()
		require.NoError(t, err)
		bl, ok := g.Block.(*eth.GenericSignedBeaconBlock_BlindedCapella)
		require.Equal(t, true, ok)
		ssz, err := bl.BlindedCapella.Block.MarshalSSZ()
		require.NoError(t, err)
		require.Equal(t, string(ssz), writer.Body.String())
		require.Equal(t, "true", writer.Header().Get(api.ExecutionPayloadBlindedHeader))
		require.Equal(t, "2000", writer.Header().Get(api.ExecutionPayloadValueHeader))
		require.Equal(t, "capella", writer.Header().Get(api.VersionHeader))
		require.Equal(t, "10", writer.Header().Get(api.ConsensusBlockValueHeader))
	})
	t.Run("Deneb", func(t *testing.T) {
		var block *shared.SignedBeaconBlockContentsDeneb
		err := json.Unmarshal([]byte(rpctesting.DenebBlockContents), &block)
		require.NoError(t, err)
		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: false,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				b, err := block.ToUnsigned().ToGeneric()
				require.NoError(t, err)
				b.PayloadValue = "2000"
				return b, nil
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}
		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v3/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		request.Header.Set("Accept", api.OctetStreamMediaType)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV3(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		g, err := block.ToUnsigned().ToGeneric()
		require.NoError(t, err)
		bl, ok := g.Block.(*eth.GenericBeaconBlock_Deneb)
		require.Equal(t, true, ok)
		ssz, err := bl.Deneb.MarshalSSZ()
		require.NoError(t, err)
		require.Equal(t, string(ssz), writer.Body.String())
		require.Equal(t, "false", writer.Header().Get(api.ExecutionPayloadBlindedHeader))
		require.Equal(t, "2000", writer.Header().Get(api.ExecutionPayloadValueHeader))
		require.Equal(t, "deneb", writer.Header().Get(api.VersionHeader))
		require.Equal(t, "10", writer.Header().Get(api.ConsensusBlockValueHeader))
	})
	t.Run("Blinded Deneb", func(t *testing.T) {
		var block *shared.SignedBlindedBeaconBlockDeneb
		err := json.Unmarshal([]byte(rpctesting.BlindedDenebBlock), &block)
		require.NoError(t, err)
		v1alpha1Server := mock2.NewMockBeaconNodeValidatorServer(ctrl)
		v1alpha1Server.EXPECT().GetBeaconBlock(gomock.Any(), &eth.BlockRequest{
			Slot:         1,
			RandaoReveal: bRandao,
			Graffiti:     bGraffiti,
			SkipMevBoost: false,
		}).Return(
			func() (*eth.GenericBeaconBlock, error) {
				b, err := block.Message.ToGeneric()
				require.NoError(t, err)
				b.PayloadValue = "2000"
				return b, nil
			}())
		server := &Server{
			V1Alpha1Server:        v1alpha1Server,
			SyncChecker:           syncChecker,
			OptimisticModeFetcher: chainService,
			BlockRewardFetcher:    rewardFetcher,
		}
		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://foo.example/eth/v3/validator/blocks/1?randao_reveal=%s&graffiti=%s", randao, graffiti), nil)
		request.Header.Set("Accept", api.OctetStreamMediaType)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		server.ProduceBlockV3(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		g, err := block.Message.ToGeneric()
		require.NoError(t, err)
		bl, ok := g.Block.(*eth.GenericBeaconBlock_BlindedDeneb)
		require.Equal(t, true, ok)
		ssz, err := bl.BlindedDeneb.MarshalSSZ()
		require.NoError(t, err)
		require.Equal(t, string(ssz), writer.Body.String())
		require.Equal(t, "true", writer.Header().Get(api.ExecutionPayloadBlindedHeader))
		require.Equal(t, "2000", writer.Header().Get(api.ExecutionPayloadValueHeader))
		require.Equal(t, "deneb", writer.Header().Get(api.VersionHeader))
		require.Equal(t, "10", writer.Header().Get(api.ConsensusBlockValueHeader))
	})
}
