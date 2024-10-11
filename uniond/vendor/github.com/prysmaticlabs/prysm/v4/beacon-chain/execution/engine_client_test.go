package execution

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"math/big"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	gethtypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/rpc"
	gethRPC "github.com/ethereum/go-ethereum/rpc"
	"github.com/holiman/uint256"
	"github.com/pkg/errors"
	mocks "github.com/prysmaticlabs/prysm/v4/beacon-chain/execution/testing"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	payloadattribute "github.com/prysmaticlabs/prysm/v4/consensus-types/payload-attribute"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	pb "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

var (
	_ = PayloadReconstructor(&Service{})
	_ = EngineCaller(&Service{})
	_ = PayloadReconstructor(&Service{})
	_ = EngineCaller(&mocks.EngineClient{})
)

type RPCClientBad struct {
}

func (RPCClientBad) Close() {}
func (RPCClientBad) BatchCall([]gethRPC.BatchElem) error {
	return errors.New("rpc client is not initialized")
}

func (RPCClientBad) CallContext(context.Context, interface{}, string, ...interface{}) error {
	return ethereum.NotFound
}

func TestClient_IPC(t *testing.T) {
	t.Skip("Skipping IPC test to support Capella devnet-3")
	server := newTestIPCServer(t)
	defer server.Stop()
	rpcClient := rpc.DialInProc(server)
	defer rpcClient.Close()
	srv := &Service{}
	srv.rpcClient = rpcClient
	ctx := context.Background()
	fix := fixtures()

	params.SetupTestConfigCleanup(t)
	cfg := params.BeaconConfig().Copy()
	cfg.CapellaForkEpoch = 1
	params.OverrideBeaconConfig(cfg)

	t.Run(GetPayloadMethod, func(t *testing.T) {
		want, ok := fix["ExecutionPayload"].(*pb.ExecutionPayload)
		require.Equal(t, true, ok)
		payloadId := [8]byte{1}
		resp, _, override, err := srv.GetPayload(ctx, payloadId, 1)
		require.NoError(t, err)
		require.Equal(t, false, override)
		resPb, err := resp.PbBellatrix()
		require.NoError(t, err)
		require.DeepEqual(t, want, resPb)
	})
	t.Run(GetPayloadMethodV2, func(t *testing.T) {
		want, ok := fix["ExecutionPayloadCapellaWithValue"].(*pb.ExecutionPayloadCapellaWithValue)
		require.Equal(t, true, ok)
		payloadId := [8]byte{1}
		resp, _, override, err := srv.GetPayload(ctx, payloadId, params.BeaconConfig().SlotsPerEpoch)
		require.NoError(t, err)
		require.Equal(t, false, override)
		resPb, err := resp.PbCapella()
		require.NoError(t, err)
		require.DeepEqual(t, want, resPb)
	})
	t.Run(ForkchoiceUpdatedMethod, func(t *testing.T) {
		want, ok := fix["ForkchoiceUpdatedResponse"].(*ForkchoiceUpdatedResponse)
		require.Equal(t, true, ok)
		p, err := payloadattribute.New(&pb.PayloadAttributes{})
		require.NoError(t, err)
		payloadID, validHash, err := srv.ForkchoiceUpdated(ctx, &pb.ForkchoiceState{}, p)
		require.NoError(t, err)
		require.DeepEqual(t, want.Status.LatestValidHash, validHash)
		require.DeepEqual(t, want.PayloadId, payloadID)
	})
	t.Run(ForkchoiceUpdatedMethodV2, func(t *testing.T) {
		want, ok := fix["ForkchoiceUpdatedResponse"].(*ForkchoiceUpdatedResponse)
		require.Equal(t, true, ok)
		p, err := payloadattribute.New(&pb.PayloadAttributesV2{})
		require.NoError(t, err)
		payloadID, validHash, err := srv.ForkchoiceUpdated(ctx, &pb.ForkchoiceState{}, p)
		require.NoError(t, err)
		require.DeepEqual(t, want.Status.LatestValidHash, validHash)
		require.DeepEqual(t, want.PayloadId, payloadID)
	})
	t.Run(NewPayloadMethod, func(t *testing.T) {
		want, ok := fix["ValidPayloadStatus"].(*pb.PayloadStatus)
		require.Equal(t, true, ok)
		req, ok := fix["ExecutionPayload"].(*pb.ExecutionPayload)
		require.Equal(t, true, ok)
		wrappedPayload, err := blocks.WrappedExecutionPayload(req)
		require.NoError(t, err)
		latestValidHash, err := srv.NewPayload(ctx, wrappedPayload, []common.Hash{}, &common.Hash{})
		require.NoError(t, err)
		require.DeepEqual(t, bytesutil.ToBytes32(want.LatestValidHash), bytesutil.ToBytes32(latestValidHash))
	})
	t.Run(NewPayloadMethodV2, func(t *testing.T) {
		want, ok := fix["ValidPayloadStatus"].(*pb.PayloadStatus)
		require.Equal(t, true, ok)
		req, ok := fix["ExecutionPayloadCapella"].(*pb.ExecutionPayloadCapella)
		require.Equal(t, true, ok)
		wrappedPayload, err := blocks.WrappedExecutionPayloadCapella(req, big.NewInt(0))
		require.NoError(t, err)
		latestValidHash, err := srv.NewPayload(ctx, wrappedPayload, []common.Hash{}, &common.Hash{})
		require.NoError(t, err)
		require.DeepEqual(t, bytesutil.ToBytes32(want.LatestValidHash), bytesutil.ToBytes32(latestValidHash))
	})
	t.Run(BlockByNumberMethod, func(t *testing.T) {
		want, ok := fix["ExecutionBlock"].(*pb.ExecutionBlock)
		require.Equal(t, true, ok)
		resp, err := srv.LatestExecutionBlock(ctx)
		require.NoError(t, err)
		require.DeepEqual(t, want, resp)
	})
	t.Run(BlockByHashMethod, func(t *testing.T) {
		want, ok := fix["ExecutionBlock"].(*pb.ExecutionBlock)
		require.Equal(t, true, ok)
		arg := common.BytesToHash([]byte("foo"))
		resp, err := srv.ExecutionBlockByHash(ctx, arg, true /* with txs */)
		require.NoError(t, err)
		require.DeepEqual(t, want, resp)
	})
}

func TestClient_HTTP(t *testing.T) {
	ctx := context.Background()
	fix := fixtures()

	params.SetupTestConfigCleanup(t)
	cfg := params.BeaconConfig().Copy()
	cfg.CapellaForkEpoch = 1
	cfg.DenebForkEpoch = 2
	params.OverrideBeaconConfig(cfg)

	t.Run(GetPayloadMethod, func(t *testing.T) {
		payloadId := [8]byte{1}
		want, ok := fix["ExecutionPayload"].(*pb.ExecutionPayload)
		require.Equal(t, true, ok)
		srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			defer func() {
				require.NoError(t, r.Body.Close())
			}()
			enc, err := io.ReadAll(r.Body)
			require.NoError(t, err)
			jsonRequestString := string(enc)

			reqArg, err := json.Marshal(pb.PayloadIDBytes(payloadId))
			require.NoError(t, err)

			// We expect the JSON string RPC request contains the right arguments.
			require.Equal(t, true, strings.Contains(
				jsonRequestString, string(reqArg),
			))
			resp := map[string]interface{}{
				"jsonrpc": "2.0",
				"id":      1,
				"result":  want,
			}
			err = json.NewEncoder(w).Encode(resp)
			require.NoError(t, err)
		}))
		defer srv.Close()

		rpcClient, err := rpc.DialHTTP(srv.URL)
		require.NoError(t, err)
		defer rpcClient.Close()

		client := &Service{}
		client.rpcClient = rpcClient

		// We call the RPC method via HTTP and expect a proper result.
		resp, _, override, err := client.GetPayload(ctx, payloadId, 1)
		require.NoError(t, err)
		require.Equal(t, false, override)
		pb, err := resp.PbBellatrix()
		require.NoError(t, err)
		require.DeepEqual(t, want, pb)
	})
	t.Run(GetPayloadMethodV2, func(t *testing.T) {
		payloadId := [8]byte{1}
		want, ok := fix["ExecutionPayloadCapellaWithValue"].(*pb.GetPayloadV2ResponseJson)
		require.Equal(t, true, ok)
		srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			defer func() {
				require.NoError(t, r.Body.Close())
			}()
			enc, err := io.ReadAll(r.Body)
			require.NoError(t, err)
			jsonRequestString := string(enc)

			reqArg, err := json.Marshal(pb.PayloadIDBytes(payloadId))
			require.NoError(t, err)

			// We expect the JSON string RPC request contains the right arguments.
			require.Equal(t, true, strings.Contains(
				jsonRequestString, string(reqArg),
			))
			resp := map[string]interface{}{
				"jsonrpc": "2.0",
				"id":      1,
				"result":  want,
			}
			err = json.NewEncoder(w).Encode(resp)
			require.NoError(t, err)
		}))
		defer srv.Close()

		rpcClient, err := rpc.DialHTTP(srv.URL)
		require.NoError(t, err)
		defer rpcClient.Close()

		client := &Service{}
		client.rpcClient = rpcClient

		// We call the RPC method via HTTP and expect a proper result.
		resp, _, override, err := client.GetPayload(ctx, payloadId, params.BeaconConfig().SlotsPerEpoch)
		require.NoError(t, err)
		require.Equal(t, false, override)
		pb, err := resp.PbCapella()
		require.NoError(t, err)
		require.DeepEqual(t, want.ExecutionPayload.BlockHash.Bytes(), pb.BlockHash)
		require.DeepEqual(t, want.ExecutionPayload.StateRoot.Bytes(), pb.StateRoot)
		require.DeepEqual(t, want.ExecutionPayload.ParentHash.Bytes(), pb.ParentHash)
		require.DeepEqual(t, want.ExecutionPayload.FeeRecipient.Bytes(), pb.FeeRecipient)
		require.DeepEqual(t, want.ExecutionPayload.PrevRandao.Bytes(), pb.PrevRandao)
		require.DeepEqual(t, want.ExecutionPayload.ParentHash.Bytes(), pb.ParentHash)

		v, err := resp.ValueInGwei()
		require.NoError(t, err)
		require.Equal(t, uint64(1236), v)
	})
	t.Run(GetPayloadMethodV3, func(t *testing.T) {
		payloadId := [8]byte{1}
		want, ok := fix["ExecutionPayloadDenebWithValue"].(*pb.GetPayloadV3ResponseJson)
		require.Equal(t, true, ok)
		srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			defer func() {
				require.NoError(t, r.Body.Close())
			}()
			enc, err := io.ReadAll(r.Body)
			require.NoError(t, err)
			jsonRequestString := string(enc)

			reqArg, err := json.Marshal(pb.PayloadIDBytes(payloadId))
			require.NoError(t, err)

			// We expect the JSON string RPC request contains the right arguments.
			require.Equal(t, true, strings.Contains(
				jsonRequestString, string(reqArg),
			))
			resp := map[string]interface{}{
				"jsonrpc": "2.0",
				"id":      1,
				"result":  want,
			}
			err = json.NewEncoder(w).Encode(resp)
			require.NoError(t, err)
		}))
		defer srv.Close()

		rpcClient, err := rpc.DialHTTP(srv.URL)
		require.NoError(t, err)
		defer rpcClient.Close()

		client := &Service{}
		client.rpcClient = rpcClient

		// We call the RPC method via HTTP and expect a proper result.
		resp, blobsBundle, override, err := client.GetPayload(ctx, payloadId, 2*params.BeaconConfig().SlotsPerEpoch)
		require.NoError(t, err)
		require.Equal(t, true, override)
		g, err := resp.ExcessBlobGas()
		require.NoError(t, err)
		require.DeepEqual(t, uint64(3), g)
		g, err = resp.BlobGasUsed()
		require.NoError(t, err)
		require.DeepEqual(t, uint64(2), g)

		commitments := [][]byte{bytesutil.PadTo([]byte("commitment1"), fieldparams.BLSPubkeyLength), bytesutil.PadTo([]byte("commitment2"), fieldparams.BLSPubkeyLength)}
		require.DeepEqual(t, commitments, blobsBundle.KzgCommitments)
		proofs := [][]byte{bytesutil.PadTo([]byte("proof1"), fieldparams.BLSPubkeyLength), bytesutil.PadTo([]byte("proof2"), fieldparams.BLSPubkeyLength)}
		require.DeepEqual(t, proofs, blobsBundle.Proofs)
		blobs := [][]byte{bytesutil.PadTo([]byte("a"), fieldparams.BlobLength), bytesutil.PadTo([]byte("b"), fieldparams.BlobLength)}
		require.DeepEqual(t, blobs, blobsBundle.Blobs)
	})
	t.Run(ForkchoiceUpdatedMethod+" VALID status", func(t *testing.T) {
		forkChoiceState := &pb.ForkchoiceState{
			HeadBlockHash:      []byte("head"),
			SafeBlockHash:      []byte("safe"),
			FinalizedBlockHash: []byte("finalized"),
		}
		payloadAttributes := &pb.PayloadAttributes{
			Timestamp:             1,
			PrevRandao:            []byte("random"),
			SuggestedFeeRecipient: []byte("suggestedFeeRecipient"),
		}
		p, err := payloadattribute.New(payloadAttributes)
		require.NoError(t, err)
		want, ok := fix["ForkchoiceUpdatedResponse"].(*ForkchoiceUpdatedResponse)
		require.Equal(t, true, ok)
		srv := forkchoiceUpdateSetup(t, forkChoiceState, payloadAttributes, want)

		// We call the RPC method via HTTP and expect a proper result.
		payloadID, validHash, err := srv.ForkchoiceUpdated(ctx, forkChoiceState, p)
		require.NoError(t, err)
		require.DeepEqual(t, want.Status.LatestValidHash, validHash)
		require.DeepEqual(t, want.PayloadId, payloadID)
	})
	t.Run(ForkchoiceUpdatedMethodV2+" VALID status", func(t *testing.T) {
		forkChoiceState := &pb.ForkchoiceState{
			HeadBlockHash:      []byte("head"),
			SafeBlockHash:      []byte("safe"),
			FinalizedBlockHash: []byte("finalized"),
		}
		payloadAttributes := &pb.PayloadAttributesV2{
			Timestamp:             1,
			PrevRandao:            []byte("random"),
			SuggestedFeeRecipient: []byte("suggestedFeeRecipient"),
			Withdrawals:           []*pb.Withdrawal{{ValidatorIndex: 1, Amount: 1}},
		}
		p, err := payloadattribute.New(payloadAttributes)
		require.NoError(t, err)
		want, ok := fix["ForkchoiceUpdatedResponse"].(*ForkchoiceUpdatedResponse)
		require.Equal(t, true, ok)
		srv := forkchoiceUpdateSetupV2(t, forkChoiceState, payloadAttributes, want)

		// We call the RPC method via HTTP and expect a proper result.
		payloadID, validHash, err := srv.ForkchoiceUpdated(ctx, forkChoiceState, p)
		require.NoError(t, err)
		require.DeepEqual(t, want.Status.LatestValidHash, validHash)
		require.DeepEqual(t, want.PayloadId, payloadID)
	})
	t.Run(ForkchoiceUpdatedMethod+" SYNCING status", func(t *testing.T) {
		forkChoiceState := &pb.ForkchoiceState{
			HeadBlockHash:      []byte("head"),
			SafeBlockHash:      []byte("safe"),
			FinalizedBlockHash: []byte("finalized"),
		}
		payloadAttributes := &pb.PayloadAttributes{
			Timestamp:             1,
			PrevRandao:            []byte("random"),
			SuggestedFeeRecipient: []byte("suggestedFeeRecipient"),
		}
		p, err := payloadattribute.New(payloadAttributes)
		require.NoError(t, err)
		want, ok := fix["ForkchoiceUpdatedSyncingResponse"].(*ForkchoiceUpdatedResponse)
		require.Equal(t, true, ok)
		client := forkchoiceUpdateSetup(t, forkChoiceState, payloadAttributes, want)

		// We call the RPC method via HTTP and expect a proper result.
		payloadID, validHash, err := client.ForkchoiceUpdated(ctx, forkChoiceState, p)
		require.ErrorIs(t, err, ErrAcceptedSyncingPayloadStatus)
		require.DeepEqual(t, (*pb.PayloadIDBytes)(nil), payloadID)
		require.DeepEqual(t, []byte(nil), validHash)
	})
	t.Run(ForkchoiceUpdatedMethodV2+" SYNCING status", func(t *testing.T) {
		forkChoiceState := &pb.ForkchoiceState{
			HeadBlockHash:      []byte("head"),
			SafeBlockHash:      []byte("safe"),
			FinalizedBlockHash: []byte("finalized"),
		}
		payloadAttributes := &pb.PayloadAttributesV2{
			Timestamp:             1,
			PrevRandao:            []byte("random"),
			SuggestedFeeRecipient: []byte("suggestedFeeRecipient"),
			Withdrawals:           []*pb.Withdrawal{{ValidatorIndex: 1, Amount: 1}},
		}
		p, err := payloadattribute.New(payloadAttributes)
		require.NoError(t, err)
		want, ok := fix["ForkchoiceUpdatedSyncingResponse"].(*ForkchoiceUpdatedResponse)
		require.Equal(t, true, ok)
		srv := forkchoiceUpdateSetupV2(t, forkChoiceState, payloadAttributes, want)

		// We call the RPC method via HTTP and expect a proper result.
		payloadID, validHash, err := srv.ForkchoiceUpdated(ctx, forkChoiceState, p)
		require.ErrorIs(t, err, ErrAcceptedSyncingPayloadStatus)
		require.DeepEqual(t, (*pb.PayloadIDBytes)(nil), payloadID)
		require.DeepEqual(t, []byte(nil), validHash)
	})
	t.Run(ForkchoiceUpdatedMethod+" INVALID status", func(t *testing.T) {
		forkChoiceState := &pb.ForkchoiceState{
			HeadBlockHash:      []byte("head"),
			SafeBlockHash:      []byte("safe"),
			FinalizedBlockHash: []byte("finalized"),
		}
		payloadAttributes := &pb.PayloadAttributes{
			Timestamp:             1,
			PrevRandao:            []byte("random"),
			SuggestedFeeRecipient: []byte("suggestedFeeRecipient"),
		}
		p, err := payloadattribute.New(payloadAttributes)
		require.NoError(t, err)
		want, ok := fix["ForkchoiceUpdatedInvalidResponse"].(*ForkchoiceUpdatedResponse)
		require.Equal(t, true, ok)
		client := forkchoiceUpdateSetup(t, forkChoiceState, payloadAttributes, want)

		// We call the RPC method via HTTP and expect a proper result.
		payloadID, validHash, err := client.ForkchoiceUpdated(ctx, forkChoiceState, p)
		require.ErrorIs(t, err, ErrInvalidPayloadStatus)
		require.DeepEqual(t, (*pb.PayloadIDBytes)(nil), payloadID)
		require.DeepEqual(t, want.Status.LatestValidHash, validHash)
	})
	t.Run(ForkchoiceUpdatedMethod+" UNKNOWN status", func(t *testing.T) {
		forkChoiceState := &pb.ForkchoiceState{
			HeadBlockHash:      []byte("head"),
			SafeBlockHash:      []byte("safe"),
			FinalizedBlockHash: []byte("finalized"),
		}
		payloadAttributes := &pb.PayloadAttributes{
			Timestamp:             1,
			PrevRandao:            []byte("random"),
			SuggestedFeeRecipient: []byte("suggestedFeeRecipient"),
		}
		p, err := payloadattribute.New(payloadAttributes)
		require.NoError(t, err)
		want, ok := fix["ForkchoiceUpdatedAcceptedResponse"].(*ForkchoiceUpdatedResponse)
		require.Equal(t, true, ok)
		client := forkchoiceUpdateSetup(t, forkChoiceState, payloadAttributes, want)

		// We call the RPC method via HTTP and expect a proper result.
		payloadID, validHash, err := client.ForkchoiceUpdated(ctx, forkChoiceState, p)
		require.ErrorIs(t, err, ErrUnknownPayloadStatus)
		require.DeepEqual(t, (*pb.PayloadIDBytes)(nil), payloadID)
		require.DeepEqual(t, []byte(nil), validHash)
	})
	t.Run(NewPayloadMethod+" VALID status", func(t *testing.T) {
		execPayload, ok := fix["ExecutionPayload"].(*pb.ExecutionPayload)
		require.Equal(t, true, ok)
		want, ok := fix["ValidPayloadStatus"].(*pb.PayloadStatus)
		require.Equal(t, true, ok)
		client := newPayloadSetup(t, want, execPayload)

		// We call the RPC method via HTTP and expect a proper result.
		wrappedPayload, err := blocks.WrappedExecutionPayload(execPayload)
		require.NoError(t, err)
		resp, err := client.NewPayload(ctx, wrappedPayload, []common.Hash{}, &common.Hash{})
		require.NoError(t, err)
		require.DeepEqual(t, want.LatestValidHash, resp)
	})
	t.Run(NewPayloadMethodV2+" VALID status", func(t *testing.T) {
		execPayload, ok := fix["ExecutionPayloadCapella"].(*pb.ExecutionPayloadCapella)
		require.Equal(t, true, ok)
		want, ok := fix["ValidPayloadStatus"].(*pb.PayloadStatus)
		require.Equal(t, true, ok)
		client := newPayloadV2Setup(t, want, execPayload)

		// We call the RPC method via HTTP and expect a proper result.
		wrappedPayload, err := blocks.WrappedExecutionPayloadCapella(execPayload, big.NewInt(0))
		require.NoError(t, err)
		resp, err := client.NewPayload(ctx, wrappedPayload, []common.Hash{}, &common.Hash{})
		require.NoError(t, err)
		require.DeepEqual(t, want.LatestValidHash, resp)
	})
	t.Run(NewPayloadMethodV3+" VALID status", func(t *testing.T) {
		execPayload, ok := fix["ExecutionPayloadDeneb"].(*pb.ExecutionPayloadDeneb)
		require.Equal(t, true, ok)
		want, ok := fix["ValidPayloadStatus"].(*pb.PayloadStatus)
		require.Equal(t, true, ok)
		client := newPayloadV3Setup(t, want, execPayload)

		// We call the RPC method via HTTP and expect a proper result.
		wrappedPayload, err := blocks.WrappedExecutionPayloadDeneb(execPayload, big.NewInt(0))
		require.NoError(t, err)
		resp, err := client.NewPayload(ctx, wrappedPayload, []common.Hash{}, &common.Hash{'a'})
		require.NoError(t, err)
		require.DeepEqual(t, want.LatestValidHash, resp)
	})
	t.Run(NewPayloadMethod+" SYNCING status", func(t *testing.T) {
		execPayload, ok := fix["ExecutionPayload"].(*pb.ExecutionPayload)
		require.Equal(t, true, ok)
		want, ok := fix["SyncingStatus"].(*pb.PayloadStatus)
		require.Equal(t, true, ok)
		client := newPayloadSetup(t, want, execPayload)

		// We call the RPC method via HTTP and expect a proper result.
		wrappedPayload, err := blocks.WrappedExecutionPayload(execPayload)
		require.NoError(t, err)
		resp, err := client.NewPayload(ctx, wrappedPayload, []common.Hash{}, &common.Hash{})
		require.ErrorIs(t, ErrAcceptedSyncingPayloadStatus, err)
		require.DeepEqual(t, []uint8(nil), resp)
	})
	t.Run(NewPayloadMethodV2+" SYNCING status", func(t *testing.T) {
		execPayload, ok := fix["ExecutionPayloadCapella"].(*pb.ExecutionPayloadCapella)
		require.Equal(t, true, ok)
		want, ok := fix["SyncingStatus"].(*pb.PayloadStatus)
		require.Equal(t, true, ok)
		client := newPayloadV2Setup(t, want, execPayload)

		// We call the RPC method via HTTP and expect a proper result.
		wrappedPayload, err := blocks.WrappedExecutionPayloadCapella(execPayload, big.NewInt(0))
		require.NoError(t, err)
		resp, err := client.NewPayload(ctx, wrappedPayload, []common.Hash{}, &common.Hash{})
		require.ErrorIs(t, ErrAcceptedSyncingPayloadStatus, err)
		require.DeepEqual(t, []uint8(nil), resp)
	})
	t.Run(NewPayloadMethodV3+" SYNCING status", func(t *testing.T) {
		execPayload, ok := fix["ExecutionPayloadDeneb"].(*pb.ExecutionPayloadDeneb)
		require.Equal(t, true, ok)
		want, ok := fix["SyncingStatus"].(*pb.PayloadStatus)
		require.Equal(t, true, ok)
		client := newPayloadV3Setup(t, want, execPayload)

		// We call the RPC method via HTTP and expect a proper result.
		wrappedPayload, err := blocks.WrappedExecutionPayloadDeneb(execPayload, big.NewInt(0))
		require.NoError(t, err)
		resp, err := client.NewPayload(ctx, wrappedPayload, []common.Hash{}, &common.Hash{'a'})
		require.ErrorIs(t, ErrAcceptedSyncingPayloadStatus, err)
		require.DeepEqual(t, []uint8(nil), resp)
	})
	t.Run(NewPayloadMethod+" INVALID_BLOCK_HASH status", func(t *testing.T) {
		execPayload, ok := fix["ExecutionPayload"].(*pb.ExecutionPayload)
		require.Equal(t, true, ok)
		want, ok := fix["InvalidBlockHashStatus"].(*pb.PayloadStatus)
		require.Equal(t, true, ok)
		client := newPayloadSetup(t, want, execPayload)

		// We call the RPC method via HTTP and expect a proper result.
		wrappedPayload, err := blocks.WrappedExecutionPayload(execPayload)
		require.NoError(t, err)
		resp, err := client.NewPayload(ctx, wrappedPayload, []common.Hash{}, &common.Hash{})
		require.ErrorIs(t, ErrInvalidBlockHashPayloadStatus, err)
		require.DeepEqual(t, []uint8(nil), resp)
	})
	t.Run(NewPayloadMethodV2+" INVALID_BLOCK_HASH status", func(t *testing.T) {
		execPayload, ok := fix["ExecutionPayloadCapella"].(*pb.ExecutionPayloadCapella)
		require.Equal(t, true, ok)
		want, ok := fix["InvalidBlockHashStatus"].(*pb.PayloadStatus)
		require.Equal(t, true, ok)
		client := newPayloadV2Setup(t, want, execPayload)

		// We call the RPC method via HTTP and expect a proper result.
		wrappedPayload, err := blocks.WrappedExecutionPayloadCapella(execPayload, big.NewInt(0))
		require.NoError(t, err)
		resp, err := client.NewPayload(ctx, wrappedPayload, []common.Hash{}, &common.Hash{})
		require.ErrorIs(t, ErrInvalidBlockHashPayloadStatus, err)
		require.DeepEqual(t, []uint8(nil), resp)
	})
	t.Run(NewPayloadMethodV3+" INVALID_BLOCK_HASH status", func(t *testing.T) {
		execPayload, ok := fix["ExecutionPayloadDeneb"].(*pb.ExecutionPayloadDeneb)
		require.Equal(t, true, ok)
		want, ok := fix["InvalidBlockHashStatus"].(*pb.PayloadStatus)
		require.Equal(t, true, ok)
		client := newPayloadV3Setup(t, want, execPayload)

		// We call the RPC method via HTTP and expect a proper result.
		wrappedPayload, err := blocks.WrappedExecutionPayloadDeneb(execPayload, big.NewInt(0))
		require.NoError(t, err)
		resp, err := client.NewPayload(ctx, wrappedPayload, []common.Hash{}, &common.Hash{'a'})
		require.ErrorIs(t, ErrInvalidBlockHashPayloadStatus, err)
		require.DeepEqual(t, []uint8(nil), resp)
	})
	t.Run(NewPayloadMethod+" INVALID status", func(t *testing.T) {
		execPayload, ok := fix["ExecutionPayload"].(*pb.ExecutionPayload)
		require.Equal(t, true, ok)
		want, ok := fix["InvalidStatus"].(*pb.PayloadStatus)
		require.Equal(t, true, ok)
		client := newPayloadSetup(t, want, execPayload)

		// We call the RPC method via HTTP and expect a proper result.
		wrappedPayload, err := blocks.WrappedExecutionPayload(execPayload)
		require.NoError(t, err)
		resp, err := client.NewPayload(ctx, wrappedPayload, []common.Hash{}, &common.Hash{})
		require.ErrorIs(t, ErrInvalidPayloadStatus, err)
		require.DeepEqual(t, want.LatestValidHash, resp)
	})
	t.Run(NewPayloadMethodV2+" INVALID status", func(t *testing.T) {
		execPayload, ok := fix["ExecutionPayloadCapella"].(*pb.ExecutionPayloadCapella)
		require.Equal(t, true, ok)
		want, ok := fix["InvalidStatus"].(*pb.PayloadStatus)
		require.Equal(t, true, ok)
		client := newPayloadV2Setup(t, want, execPayload)

		// We call the RPC method via HTTP and expect a proper result.
		wrappedPayload, err := blocks.WrappedExecutionPayloadCapella(execPayload, big.NewInt(0))
		require.NoError(t, err)
		resp, err := client.NewPayload(ctx, wrappedPayload, []common.Hash{}, &common.Hash{})
		require.ErrorIs(t, ErrInvalidPayloadStatus, err)
		require.DeepEqual(t, want.LatestValidHash, resp)
	})
	t.Run(NewPayloadMethodV3+" INVALID status", func(t *testing.T) {
		execPayload, ok := fix["ExecutionPayloadDeneb"].(*pb.ExecutionPayloadDeneb)
		require.Equal(t, true, ok)
		want, ok := fix["InvalidStatus"].(*pb.PayloadStatus)
		require.Equal(t, true, ok)
		client := newPayloadV3Setup(t, want, execPayload)

		// We call the RPC method via HTTP and expect a proper result.
		wrappedPayload, err := blocks.WrappedExecutionPayloadDeneb(execPayload, big.NewInt(0))
		require.NoError(t, err)
		resp, err := client.NewPayload(ctx, wrappedPayload, []common.Hash{}, &common.Hash{'a'})
		require.ErrorIs(t, ErrInvalidPayloadStatus, err)
		require.DeepEqual(t, want.LatestValidHash, resp)
	})
	t.Run(NewPayloadMethod+" UNKNOWN status", func(t *testing.T) {
		execPayload, ok := fix["ExecutionPayload"].(*pb.ExecutionPayload)
		require.Equal(t, true, ok)
		want, ok := fix["UnknownStatus"].(*pb.PayloadStatus)
		require.Equal(t, true, ok)
		client := newPayloadSetup(t, want, execPayload)

		// We call the RPC method via HTTP and expect a proper result.
		wrappedPayload, err := blocks.WrappedExecutionPayload(execPayload)
		require.NoError(t, err)
		resp, err := client.NewPayload(ctx, wrappedPayload, []common.Hash{}, &common.Hash{})
		require.ErrorIs(t, ErrUnknownPayloadStatus, err)
		require.DeepEqual(t, []uint8(nil), resp)
	})
	t.Run(BlockByNumberMethod, func(t *testing.T) {
		want, ok := fix["ExecutionBlock"].(*pb.ExecutionBlock)
		require.Equal(t, true, ok)
		srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			defer func() {
				require.NoError(t, r.Body.Close())
			}()
			resp := map[string]interface{}{
				"jsonrpc": "2.0",
				"id":      1,
				"result":  want,
			}
			err := json.NewEncoder(w).Encode(resp)
			require.NoError(t, err)
		}))
		defer srv.Close()

		rpcClient, err := rpc.DialHTTP(srv.URL)
		require.NoError(t, err)
		defer rpcClient.Close()

		service := &Service{}
		service.rpcClient = rpcClient

		// We call the RPC method via HTTP and expect a proper result.
		resp, err := service.LatestExecutionBlock(ctx)
		require.NoError(t, err)
		require.DeepEqual(t, want, resp)
	})
	t.Run(BlockByHashMethod, func(t *testing.T) {
		arg := common.BytesToHash([]byte("foo"))
		want, ok := fix["ExecutionBlock"].(*pb.ExecutionBlock)
		require.Equal(t, true, ok)
		srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			defer func() {
				require.NoError(t, r.Body.Close())
			}()
			enc, err := io.ReadAll(r.Body)
			require.NoError(t, err)
			jsonRequestString := string(enc)
			// We expect the JSON string RPC request contains the right arguments.
			require.Equal(t, true, strings.Contains(
				jsonRequestString, fmt.Sprintf("%#x", arg),
			))
			resp := map[string]interface{}{
				"jsonrpc": "2.0",
				"id":      1,
				"result":  want,
			}
			err = json.NewEncoder(w).Encode(resp)
			require.NoError(t, err)
		}))
		defer srv.Close()

		rpcClient, err := rpc.DialHTTP(srv.URL)
		require.NoError(t, err)
		defer rpcClient.Close()

		service := &Service{}
		service.rpcClient = rpcClient

		// We call the RPC method via HTTP and expect a proper result.
		resp, err := service.ExecutionBlockByHash(ctx, arg, true /* with txs */)
		require.NoError(t, err)
		require.DeepEqual(t, want, resp)
	})
}

func TestReconstructFullBellatrixBlock(t *testing.T) {
	ctx := context.Background()
	t.Run("nil block", func(t *testing.T) {
		service := &Service{}

		_, err := service.ReconstructFullBlock(ctx, nil)
		require.ErrorContains(t, "nil data", err)
	})
	t.Run("only blinded block", func(t *testing.T) {
		want := "can only reconstruct block from blinded block format"
		service := &Service{}
		bellatrixBlock := util.NewBeaconBlockBellatrix()
		wrapped, err := blocks.NewSignedBeaconBlock(bellatrixBlock)
		require.NoError(t, err)
		_, err = service.ReconstructFullBlock(ctx, wrapped)
		require.ErrorContains(t, want, err)
	})
	t.Run("pre-merge execution payload", func(t *testing.T) {
		service := &Service{}
		bellatrixBlock := util.NewBlindedBeaconBlockBellatrix()
		wanted := util.NewBeaconBlockBellatrix()
		wanted.Block.Slot = 1
		// Make sure block hash is the zero hash.
		bellatrixBlock.Block.Body.ExecutionPayloadHeader.BlockHash = make([]byte, 32)
		bellatrixBlock.Block.Slot = 1
		wrapped, err := blocks.NewSignedBeaconBlock(bellatrixBlock)
		require.NoError(t, err)
		wantedWrapped, err := blocks.NewSignedBeaconBlock(wanted)
		require.NoError(t, err)
		reconstructed, err := service.ReconstructFullBlock(ctx, wrapped)
		require.NoError(t, err)
		require.DeepEqual(t, wantedWrapped, reconstructed)
	})
	t.Run("properly reconstructs block with correct payload", func(t *testing.T) {
		fix := fixtures()
		payload, ok := fix["ExecutionPayload"].(*pb.ExecutionPayload)
		require.Equal(t, true, ok)

		jsonPayload := make(map[string]interface{})
		tx := gethtypes.NewTransaction(
			0,
			common.HexToAddress("095e7baea6a6c7c4c2dfeb977efac326af552d87"),
			big.NewInt(0), 0, big.NewInt(0),
			nil,
		)
		txs := []*gethtypes.Transaction{tx}
		encodedBinaryTxs := make([][]byte, 1)
		var err error
		encodedBinaryTxs[0], err = txs[0].MarshalBinary()
		require.NoError(t, err)
		payload.Transactions = encodedBinaryTxs
		jsonPayload["transactions"] = []hexutil.Bytes{encodedBinaryTxs[0]}

		wrappedPayload, err := blocks.WrappedExecutionPayload(payload)
		require.NoError(t, err)
		header, err := blocks.PayloadToHeader(wrappedPayload)
		require.NoError(t, err)

		srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			defer func() {
				require.NoError(t, r.Body.Close())
			}()
			respJSON := map[string]interface{}{
				"jsonrpc": "2.0",
				"id":      1,
				"result":  []map[string]interface{}{jsonPayload},
			}
			require.NoError(t, json.NewEncoder(w).Encode(respJSON))
		}))
		defer srv.Close()

		rpcClient, err := rpc.DialHTTP(srv.URL)
		require.NoError(t, err)
		defer rpcClient.Close()

		service := &Service{}
		service.rpcClient = rpcClient
		blindedBlock := util.NewBlindedBeaconBlockBellatrix()

		blindedBlock.Block.Body.ExecutionPayloadHeader = header
		wrapped, err := blocks.NewSignedBeaconBlock(blindedBlock)
		require.NoError(t, err)
		reconstructed, err := service.ReconstructFullBlock(ctx, wrapped)
		require.NoError(t, err)

		got, err := reconstructed.Block().Body().Execution()
		require.NoError(t, err)
		require.DeepEqual(t, payload, got.Proto())
	})
}

func TestReconstructFullBellatrixBlockBatch(t *testing.T) {
	ctx := context.Background()
	t.Run("nil block", func(t *testing.T) {
		service := &Service{}

		_, err := service.ReconstructFullBellatrixBlockBatch(ctx, []interfaces.ReadOnlySignedBeaconBlock{nil})
		require.ErrorContains(t, "nil data", err)
	})
	t.Run("only blinded block", func(t *testing.T) {
		want := "can only reconstruct block from blinded block format"
		service := &Service{}
		bellatrixBlock := util.NewBeaconBlockBellatrix()
		wrapped, err := blocks.NewSignedBeaconBlock(bellatrixBlock)
		require.NoError(t, err)
		_, err = service.ReconstructFullBellatrixBlockBatch(ctx, []interfaces.ReadOnlySignedBeaconBlock{wrapped})
		require.ErrorContains(t, want, err)
	})
	t.Run("pre-merge execution payload", func(t *testing.T) {
		service := &Service{}
		bellatrixBlock := util.NewBlindedBeaconBlockBellatrix()
		wanted := util.NewBeaconBlockBellatrix()
		wanted.Block.Slot = 1
		// Make sure block hash is the zero hash.
		bellatrixBlock.Block.Body.ExecutionPayloadHeader.BlockHash = make([]byte, 32)
		bellatrixBlock.Block.Slot = 1
		wrapped, err := blocks.NewSignedBeaconBlock(bellatrixBlock)
		require.NoError(t, err)
		wantedWrapped, err := blocks.NewSignedBeaconBlock(wanted)
		require.NoError(t, err)
		reconstructed, err := service.ReconstructFullBellatrixBlockBatch(ctx, []interfaces.ReadOnlySignedBeaconBlock{wrapped})
		require.NoError(t, err)
		require.DeepEqual(t, []interfaces.SignedBeaconBlock{wantedWrapped}, reconstructed)
	})
	t.Run("properly reconstructs block batch with correct payload", func(t *testing.T) {
		fix := fixtures()
		payload, ok := fix["ExecutionPayload"].(*pb.ExecutionPayload)
		require.Equal(t, true, ok)

		jsonPayload := make(map[string]interface{})
		tx := gethtypes.NewTransaction(
			0,
			common.HexToAddress("095e7baea6a6c7c4c2dfeb977efac326af552d87"),
			big.NewInt(0), 0, big.NewInt(0),
			nil,
		)
		txs := []*gethtypes.Transaction{tx}
		encodedBinaryTxs := make([][]byte, 1)
		var err error
		encodedBinaryTxs[0], err = txs[0].MarshalBinary()
		require.NoError(t, err)
		payload.Transactions = encodedBinaryTxs
		jsonPayload["transactions"] = []hexutil.Bytes{encodedBinaryTxs[0]}

		wrappedPayload, err := blocks.WrappedExecutionPayload(payload)
		require.NoError(t, err)
		header, err := blocks.PayloadToHeader(wrappedPayload)
		require.NoError(t, err)

		bellatrixBlock := util.NewBlindedBeaconBlockBellatrix()
		wanted := util.NewBeaconBlockBellatrix()
		wanted.Block.Slot = 1
		// Make sure block hash is the zero hash.
		bellatrixBlock.Block.Body.ExecutionPayloadHeader.BlockHash = make([]byte, 32)
		bellatrixBlock.Block.Slot = 1
		wrappedEmpty, err := blocks.NewSignedBeaconBlock(bellatrixBlock)
		require.NoError(t, err)
		wantedWrappedEmpty, err := blocks.NewSignedBeaconBlock(wanted)
		require.NoError(t, err)

		srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			defer func() {
				require.NoError(t, r.Body.Close())
			}()

			respJSON := map[string]interface{}{
				"jsonrpc": "2.0",
				"id":      1,
				"result":  []map[string]interface{}{jsonPayload, jsonPayload},
			}
			require.NoError(t, json.NewEncoder(w).Encode(respJSON))

		}))
		defer srv.Close()

		rpcClient, err := rpc.DialHTTP(srv.URL)
		require.NoError(t, err)
		defer rpcClient.Close()

		service := &Service{}
		service.rpcClient = rpcClient
		blindedBlock := util.NewBlindedBeaconBlockBellatrix()

		blindedBlock.Block.Body.ExecutionPayloadHeader = header
		wrapped, err := blocks.NewSignedBeaconBlock(blindedBlock)
		require.NoError(t, err)
		copiedWrapped, err := wrapped.Copy()
		require.NoError(t, err)

		reconstructed, err := service.ReconstructFullBellatrixBlockBatch(ctx, []interfaces.ReadOnlySignedBeaconBlock{wrappedEmpty, wrapped, copiedWrapped})
		require.NoError(t, err)

		// Make sure empty blocks are handled correctly
		require.DeepEqual(t, wantedWrappedEmpty, reconstructed[0])

		// Handle normal execution blocks correctly
		got, err := reconstructed[1].Block().Body().Execution()
		require.NoError(t, err)
		require.DeepEqual(t, payload, got.Proto())

		got, err = reconstructed[2].Block().Body().Execution()
		require.NoError(t, err)
		require.DeepEqual(t, payload, got.Proto())
	})
}

func TestServer_getPowBlockHashAtTerminalTotalDifficulty(t *testing.T) {
	tests := []struct {
		name                  string
		paramsTd              string
		currentPowBlock       *pb.ExecutionBlock
		parentPowBlock        *pb.ExecutionBlock
		errLatestExecutionBlk error
		wantTerminalBlockHash []byte
		wantExists            bool
		errString             string
	}{
		{
			name:      "config td overflows",
			paramsTd:  "1115792089237316195423570985008687907853269984665640564039457584007913129638912",
			errString: "could not convert terminal total difficulty to uint256",
		},
		{
			name:                  "could not get latest execution block",
			paramsTd:              "1",
			errLatestExecutionBlk: errors.New("blah"),
			errString:             "could not get latest execution block",
		},
		{
			name:      "nil latest execution block",
			paramsTd:  "1",
			errString: "latest execution block is nil",
		},
		{
			name:     "current execution block invalid TD",
			paramsTd: "1",
			currentPowBlock: &pb.ExecutionBlock{
				Hash:            common.BytesToHash([]byte("a")),
				TotalDifficulty: "1115792089237316195423570985008687907853269984665640564039457584007913129638912",
			},
			errString: "could not convert total difficulty to uint256",
		},
		{
			name:     "current execution block has zero hash parent",
			paramsTd: "2",
			currentPowBlock: &pb.ExecutionBlock{
				Hash: common.BytesToHash([]byte("a")),
				Header: gethtypes.Header{
					ParentHash: common.BytesToHash(params.BeaconConfig().ZeroHash[:]),
				},
				TotalDifficulty: "0x3",
			},
		},
		{
			name:     "could not get parent block",
			paramsTd: "2",
			currentPowBlock: &pb.ExecutionBlock{
				Hash: common.BytesToHash([]byte("a")),
				Header: gethtypes.Header{
					ParentHash: common.BytesToHash([]byte("b")),
				},
				TotalDifficulty: "0x3",
			},
			errString: "could not get parent execution block",
		},
		{
			name:     "parent execution block invalid TD",
			paramsTd: "2",
			currentPowBlock: &pb.ExecutionBlock{
				Hash: common.BytesToHash([]byte("a")),
				Header: gethtypes.Header{
					ParentHash: common.BytesToHash([]byte("b")),
				},
				TotalDifficulty: "0x3",
			},
			parentPowBlock: &pb.ExecutionBlock{
				Hash: common.BytesToHash([]byte("b")),
				Header: gethtypes.Header{
					ParentHash: common.BytesToHash([]byte("c")),
				},
				TotalDifficulty: "1",
			},
			errString: "could not convert total difficulty to uint256",
		},
		{
			name:     "happy case",
			paramsTd: "2",
			currentPowBlock: &pb.ExecutionBlock{
				Hash: common.BytesToHash([]byte("a")),
				Header: gethtypes.Header{
					ParentHash: common.BytesToHash([]byte("b")),
				},
				TotalDifficulty: "0x3",
			},
			parentPowBlock: &pb.ExecutionBlock{
				Hash: common.BytesToHash([]byte("b")),
				Header: gethtypes.Header{
					ParentHash: common.BytesToHash([]byte("c")),
				},
				TotalDifficulty: "0x1",
			},
			wantExists:            true,
			wantTerminalBlockHash: common.BytesToHash([]byte("a")).Bytes(),
		},
		{
			name:     "happy case, but invalid timestamp",
			paramsTd: "2",
			currentPowBlock: &pb.ExecutionBlock{
				Hash: common.BytesToHash([]byte("a")),
				Header: gethtypes.Header{
					ParentHash: common.BytesToHash([]byte("b")),
					Time:       1,
				},
				TotalDifficulty: "0x3",
			},
			parentPowBlock: &pb.ExecutionBlock{
				Hash: common.BytesToHash([]byte("b")),
				Header: gethtypes.Header{
					ParentHash: common.BytesToHash([]byte("c")),
				},
				TotalDifficulty: "0x1",
			},
		},
		{
			name:     "ttd not reached",
			paramsTd: "3",
			currentPowBlock: &pb.ExecutionBlock{
				Hash: common.BytesToHash([]byte("a")),
				Header: gethtypes.Header{
					ParentHash: common.BytesToHash([]byte("b")),
				},
				TotalDifficulty: "0x2",
			},
			parentPowBlock: &pb.ExecutionBlock{
				Hash: common.BytesToHash([]byte("b")),
				Header: gethtypes.Header{
					ParentHash: common.BytesToHash([]byte("c")),
				},
				TotalDifficulty: "0x1",
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			cfg := params.BeaconConfig().Copy()
			cfg.TerminalTotalDifficulty = tt.paramsTd
			params.OverrideBeaconConfig(cfg)
			var m map[[32]byte]*pb.ExecutionBlock
			if tt.parentPowBlock != nil {
				m = map[[32]byte]*pb.ExecutionBlock{
					tt.parentPowBlock.Hash: tt.parentPowBlock,
				}
			}
			client := mocks.EngineClient{
				ErrLatestExecBlock: tt.errLatestExecutionBlk,
				ExecutionBlock:     tt.currentPowBlock,
				BlockByHashMap:     m,
			}
			b, e, err := client.GetTerminalBlockHash(context.Background(), 1)
			if tt.errString != "" {
				require.ErrorContains(t, tt.errString, err)
			} else {
				require.NoError(t, err)
				require.DeepEqual(t, tt.wantExists, e)
				require.DeepEqual(t, tt.wantTerminalBlockHash, b)
			}
		})
	}
}

func Test_tDStringToUint256(t *testing.T) {
	i, err := tDStringToUint256("0x0")
	require.NoError(t, err)
	require.DeepEqual(t, uint256.NewInt(0), i)

	i, err = tDStringToUint256("0x10000")
	require.NoError(t, err)
	require.DeepEqual(t, uint256.NewInt(65536), i)

	_, err = tDStringToUint256("100")
	require.ErrorContains(t, "hex string without 0x prefix", err)

	_, err = tDStringToUint256("0xzzzzzz")
	require.ErrorContains(t, "invalid hex string", err)

	_, err = tDStringToUint256("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF" +
		"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF")
	require.ErrorContains(t, "hex number > 256 bits", err)
}

type customError struct {
	code    int
	timeout bool
}

func (c *customError) ErrorCode() int {
	return c.code
}

func (*customError) Error() string {
	return "something went wrong"
}

func (c *customError) Timeout() bool {
	return c.timeout
}

type dataError struct {
	code int
	data interface{}
}

func (c *dataError) ErrorCode() int {
	return c.code
}

func (*dataError) Error() string {
	return "something went wrong"
}

func (c *dataError) ErrorData() interface{} {
	return c.data
}

func Test_handleRPCError(t *testing.T) {
	got := handleRPCError(nil)
	require.Equal(t, true, got == nil)

	var tests = []struct {
		name             string
		expected         error
		expectedContains string
		given            error
	}{
		{
			name:             "not an rpc error",
			expectedContains: "got an unexpected error",
			given:            errors.New("foo"),
		},
		{
			name:             "HTTP times out",
			expectedContains: ErrHTTPTimeout.Error(),
			given:            &customError{timeout: true},
		},
		{
			name:             "ErrParse",
			expectedContains: ErrParse.Error(),
			given:            &customError{code: -32700},
		},
		{
			name:             "ErrInvalidRequest",
			expectedContains: ErrInvalidRequest.Error(),
			given:            &customError{code: -32600},
		},
		{
			name:             "ErrMethodNotFound",
			expectedContains: ErrMethodNotFound.Error(),
			given:            &customError{code: -32601},
		},
		{
			name:             "ErrInvalidParams",
			expectedContains: ErrInvalidParams.Error(),
			given:            &customError{code: -32602},
		},
		{
			name:             "ErrInternal",
			expectedContains: ErrInternal.Error(),
			given:            &customError{code: -32603},
		},
		{
			name:             "ErrUnknownPayload",
			expectedContains: ErrUnknownPayload.Error(),
			given:            &customError{code: -38001},
		},
		{
			name:             "ErrInvalidForkchoiceState",
			expectedContains: ErrInvalidForkchoiceState.Error(),
			given:            &customError{code: -38002},
		},
		{
			name:             "ErrInvalidPayloadAttributes",
			expectedContains: ErrInvalidPayloadAttributes.Error(),
			given:            &customError{code: -38003},
		},
		{
			name:             "ErrServer unexpected no data",
			expectedContains: "got an unexpected error",
			given:            &customError{code: -32000},
		},
		{
			name:             "ErrServer with data",
			expectedContains: ErrServer.Error(),
			given:            &dataError{code: -32000, data: 5},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := handleRPCError(tt.given)
			require.ErrorContains(t, tt.expectedContains, got)
		})
	}
}

func newTestIPCServer(t *testing.T) *rpc.Server {
	server := rpc.NewServer()
	err := server.RegisterName("engine", new(testEngineService))
	require.NoError(t, err)
	err = server.RegisterName("eth", new(testEngineService))
	require.NoError(t, err)
	return server
}

func fixtures() map[string]interface{} {
	foo := bytesutil.ToBytes32([]byte("foo"))
	bar := bytesutil.PadTo([]byte("bar"), 20)
	baz := bytesutil.PadTo([]byte("baz"), 256)
	baseFeePerGas := big.NewInt(12345)
	executionPayloadFixture := &pb.ExecutionPayload{
		ParentHash:    foo[:],
		FeeRecipient:  bar,
		StateRoot:     foo[:],
		ReceiptsRoot:  foo[:],
		LogsBloom:     baz,
		PrevRandao:    foo[:],
		BlockNumber:   1,
		GasLimit:      1,
		GasUsed:       1,
		Timestamp:     1,
		ExtraData:     foo[:],
		BaseFeePerGas: bytesutil.PadTo(baseFeePerGas.Bytes(), fieldparams.RootLength),
		BlockHash:     foo[:],
		Transactions:  [][]byte{foo[:]},
	}
	executionPayloadBodyFixture := &pb.ExecutionPayloadBodyV1{
		Transactions: [][]byte{foo[:]},
		Withdrawals:  []*pb.Withdrawal{},
	}
	executionPayloadFixtureCapella := &pb.ExecutionPayloadCapella{
		ParentHash:    foo[:],
		FeeRecipient:  bar,
		StateRoot:     foo[:],
		ReceiptsRoot:  foo[:],
		LogsBloom:     baz,
		PrevRandao:    foo[:],
		BlockNumber:   1,
		GasLimit:      1,
		GasUsed:       1,
		Timestamp:     1,
		ExtraData:     foo[:],
		BaseFeePerGas: bytesutil.PadTo(baseFeePerGas.Bytes(), fieldparams.RootLength),
		BlockHash:     foo[:],
		Transactions:  [][]byte{foo[:]},
		Withdrawals:   []*pb.Withdrawal{},
	}
	executionPayloadFixtureDeneb := &pb.ExecutionPayloadDeneb{
		ParentHash:    foo[:],
		FeeRecipient:  bar,
		StateRoot:     foo[:],
		ReceiptsRoot:  foo[:],
		LogsBloom:     baz,
		PrevRandao:    foo[:],
		BlockNumber:   1,
		GasLimit:      1,
		GasUsed:       1,
		Timestamp:     1,
		ExtraData:     foo[:],
		BaseFeePerGas: bytesutil.PadTo(baseFeePerGas.Bytes(), fieldparams.RootLength),
		BlockHash:     foo[:],
		Transactions:  [][]byte{foo[:]},
		Withdrawals:   []*pb.Withdrawal{},
		BlobGasUsed:   2,
		ExcessBlobGas: 3,
	}
	hexUint := hexutil.Uint64(1)
	executionPayloadWithValueFixtureCapella := &pb.GetPayloadV2ResponseJson{
		ExecutionPayload: &pb.ExecutionPayloadCapellaJSON{
			ParentHash:    &common.Hash{'a'},
			FeeRecipient:  &common.Address{'b'},
			StateRoot:     &common.Hash{'c'},
			ReceiptsRoot:  &common.Hash{'d'},
			LogsBloom:     &hexutil.Bytes{'e'},
			PrevRandao:    &common.Hash{'f'},
			BaseFeePerGas: "0x123",
			BlockHash:     &common.Hash{'g'},
			Transactions:  []hexutil.Bytes{{'h'}},
			Withdrawals:   []*pb.Withdrawal{},
			BlockNumber:   &hexUint,
			GasLimit:      &hexUint,
			GasUsed:       &hexUint,
			Timestamp:     &hexUint,
		},
		BlockValue: "0x11fffffffff",
	}
	bgu := hexutil.Uint64(2)
	ebg := hexutil.Uint64(3)
	executionPayloadWithValueFixtureDeneb := &pb.GetPayloadV3ResponseJson{
		ShouldOverrideBuilder: true,
		ExecutionPayload: &pb.ExecutionPayloadDenebJSON{
			ParentHash:    &common.Hash{'a'},
			FeeRecipient:  &common.Address{'b'},
			StateRoot:     &common.Hash{'c'},
			ReceiptsRoot:  &common.Hash{'d'},
			LogsBloom:     &hexutil.Bytes{'e'},
			PrevRandao:    &common.Hash{'f'},
			BaseFeePerGas: "0x123",
			BlockHash:     &common.Hash{'g'},
			Transactions:  []hexutil.Bytes{{'h'}},
			Withdrawals:   []*pb.Withdrawal{},
			BlockNumber:   &hexUint,
			GasLimit:      &hexUint,
			GasUsed:       &hexUint,
			Timestamp:     &hexUint,
			BlobGasUsed:   &bgu,
			ExcessBlobGas: &ebg,
		},
		BlockValue: "0x11fffffffff",
		BlobsBundle: &pb.BlobBundleJSON{
			Commitments: []hexutil.Bytes{[]byte("commitment1"), []byte("commitment2")},
			Proofs:      []hexutil.Bytes{[]byte("proof1"), []byte("proof2")},
			Blobs:       []hexutil.Bytes{{'a'}, {'b'}},
		},
	}
	parent := bytesutil.PadTo([]byte("parentHash"), fieldparams.RootLength)
	sha3Uncles := bytesutil.PadTo([]byte("sha3Uncles"), fieldparams.RootLength)
	miner := bytesutil.PadTo([]byte("miner"), fieldparams.FeeRecipientLength)
	stateRoot := bytesutil.PadTo([]byte("stateRoot"), fieldparams.RootLength)
	transactionsRoot := bytesutil.PadTo([]byte("transactionsRoot"), fieldparams.RootLength)
	receiptsRoot := bytesutil.PadTo([]byte("receiptsRoot"), fieldparams.RootLength)
	logsBloom := bytesutil.PadTo([]byte("logs"), fieldparams.LogsBloomLength)
	executionBlock := &pb.ExecutionBlock{
		Version: version.Bellatrix,
		Header: gethtypes.Header{
			ParentHash:  common.BytesToHash(parent),
			UncleHash:   common.BytesToHash(sha3Uncles),
			Coinbase:    common.BytesToAddress(miner),
			Root:        common.BytesToHash(stateRoot),
			TxHash:      common.BytesToHash(transactionsRoot),
			ReceiptHash: common.BytesToHash(receiptsRoot),
			Bloom:       gethtypes.BytesToBloom(logsBloom),
			Difficulty:  big.NewInt(1),
			Number:      big.NewInt(2),
			GasLimit:    3,
			GasUsed:     4,
			Time:        5,
			Extra:       []byte("extra"),
			MixDigest:   common.BytesToHash([]byte("mix")),
			Nonce:       gethtypes.EncodeNonce(6),
			BaseFee:     big.NewInt(7),
		},
	}
	status := &pb.PayloadStatus{
		Status:          pb.PayloadStatus_VALID,
		LatestValidHash: foo[:],
		ValidationError: "",
	}
	id := pb.PayloadIDBytes([8]byte{1, 0, 0, 0, 0, 0, 0, 0})
	forkChoiceResp := &ForkchoiceUpdatedResponse{
		Status:    status,
		PayloadId: &id,
	}
	forkChoiceSyncingResp := &ForkchoiceUpdatedResponse{
		Status: &pb.PayloadStatus{
			Status:          pb.PayloadStatus_SYNCING,
			LatestValidHash: nil,
		},
		PayloadId: &id,
	}
	forkChoiceAcceptedResp := &ForkchoiceUpdatedResponse{
		Status: &pb.PayloadStatus{
			Status:          pb.PayloadStatus_ACCEPTED,
			LatestValidHash: nil,
		},
		PayloadId: &id,
	}
	forkChoiceInvalidResp := &ForkchoiceUpdatedResponse{
		Status: &pb.PayloadStatus{
			Status:          pb.PayloadStatus_INVALID,
			LatestValidHash: bytesutil.PadTo([]byte("latestValidHash"), 32),
		},
		PayloadId: &id,
	}
	validStatus := &pb.PayloadStatus{
		Status:          pb.PayloadStatus_VALID,
		LatestValidHash: foo[:],
		ValidationError: "",
	}
	inValidBlockHashStatus := &pb.PayloadStatus{
		Status:          pb.PayloadStatus_INVALID_BLOCK_HASH,
		LatestValidHash: nil,
	}
	acceptedStatus := &pb.PayloadStatus{
		Status:          pb.PayloadStatus_ACCEPTED,
		LatestValidHash: nil,
	}
	syncingStatus := &pb.PayloadStatus{
		Status:          pb.PayloadStatus_SYNCING,
		LatestValidHash: nil,
	}
	invalidStatus := &pb.PayloadStatus{
		Status:          pb.PayloadStatus_INVALID,
		LatestValidHash: foo[:],
	}
	unknownStatus := &pb.PayloadStatus{
		Status:          pb.PayloadStatus_UNKNOWN,
		LatestValidHash: foo[:],
	}
	return map[string]interface{}{
		"ExecutionBlock":                    executionBlock,
		"ExecutionPayloadBody":              executionPayloadBodyFixture,
		"ExecutionPayload":                  executionPayloadFixture,
		"ExecutionPayloadCapella":           executionPayloadFixtureCapella,
		"ExecutionPayloadDeneb":             executionPayloadFixtureDeneb,
		"ExecutionPayloadCapellaWithValue":  executionPayloadWithValueFixtureCapella,
		"ExecutionPayloadDenebWithValue":    executionPayloadWithValueFixtureDeneb,
		"ValidPayloadStatus":                validStatus,
		"InvalidBlockHashStatus":            inValidBlockHashStatus,
		"AcceptedStatus":                    acceptedStatus,
		"SyncingStatus":                     syncingStatus,
		"InvalidStatus":                     invalidStatus,
		"UnknownStatus":                     unknownStatus,
		"ForkchoiceUpdatedResponse":         forkChoiceResp,
		"ForkchoiceUpdatedSyncingResponse":  forkChoiceSyncingResp,
		"ForkchoiceUpdatedAcceptedResponse": forkChoiceAcceptedResp,
		"ForkchoiceUpdatedInvalidResponse":  forkChoiceInvalidResp,
	}
}

func Test_fullPayloadFromExecutionBlock(t *testing.T) {
	type args struct {
		header  *pb.ExecutionPayloadHeader
		block   *pb.ExecutionBlock
		version int
	}
	wantedHash := common.BytesToHash([]byte("foo"))
	tests := []struct {
		name string
		args args
		want func() interfaces.ExecutionData
		err  string
	}{
		{
			name: "block hash field in header and block hash mismatch",
			args: args{
				header: &pb.ExecutionPayloadHeader{
					BlockHash: []byte("foo"),
				},
				block: &pb.ExecutionBlock{
					Hash: common.BytesToHash([]byte("bar")),
				},
				version: version.Bellatrix,
			},
			err: "does not match execution block hash",
		},
		{
			name: "ok",
			args: args{
				header: &pb.ExecutionPayloadHeader{
					BlockHash: wantedHash[:],
				},
				block: &pb.ExecutionBlock{
					Hash: wantedHash,
				},
				version: version.Bellatrix,
			},
			want: func() interfaces.ExecutionData {
				p, err := blocks.WrappedExecutionPayload(&pb.ExecutionPayload{
					BlockHash:    wantedHash[:],
					Transactions: [][]byte{},
				})
				require.NoError(t, err)
				return p
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			wrapped, err := blocks.WrappedExecutionPayloadHeader(tt.args.header)
			require.NoError(t, err)
			got, err := fullPayloadFromExecutionBlock(tt.args.version, wrapped, tt.args.block)
			if err != nil {
				assert.ErrorContains(t, tt.err, err)
			} else {
				assert.DeepEqual(t, tt.want(), got)
			}
		})
	}
}

func Test_fullPayloadFromExecutionBlockCapella(t *testing.T) {
	type args struct {
		header  *pb.ExecutionPayloadHeaderCapella
		block   *pb.ExecutionBlock
		version int
	}
	wantedHash := common.BytesToHash([]byte("foo"))
	tests := []struct {
		name string
		args args
		want func() interfaces.ExecutionData
		err  string
	}{
		{
			name: "block hash field in header and block hash mismatch",
			args: args{
				header: &pb.ExecutionPayloadHeaderCapella{
					BlockHash: []byte("foo"),
				},
				block: &pb.ExecutionBlock{
					Hash: common.BytesToHash([]byte("bar")),
				},
				version: version.Capella,
			},
			err: "does not match execution block hash",
		},
		{
			name: "ok",
			args: args{
				header: &pb.ExecutionPayloadHeaderCapella{
					BlockHash: wantedHash[:],
				},
				block: &pb.ExecutionBlock{
					Hash: wantedHash,
				},
				version: version.Capella,
			},
			want: func() interfaces.ExecutionData {
				p, err := blocks.WrappedExecutionPayloadCapella(&pb.ExecutionPayloadCapella{
					BlockHash:    wantedHash[:],
					Transactions: [][]byte{},
				}, big.NewInt(0))
				require.NoError(t, err)
				return p
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			wrapped, err := blocks.WrappedExecutionPayloadHeaderCapella(tt.args.header, big.NewInt(0))
			require.NoError(t, err)
			got, err := fullPayloadFromExecutionBlock(tt.args.version, wrapped, tt.args.block)
			if err != nil {
				assert.ErrorContains(t, tt.err, err)
			} else {
				assert.DeepEqual(t, tt.want(), got)
			}
		})
	}
}

func Test_fullPayloadFromExecutionBlockDeneb(t *testing.T) {
	type args struct {
		header  *pb.ExecutionPayloadHeaderDeneb
		block   *pb.ExecutionBlock
		version int
	}
	wantedHash := common.BytesToHash([]byte("foo"))
	tests := []struct {
		name string
		args args
		want func() interfaces.ExecutionData
		err  string
	}{
		{
			name: "block hash field in header and block hash mismatch",
			args: args{
				header: &pb.ExecutionPayloadHeaderDeneb{
					BlockHash: []byte("foo"),
				},
				block: &pb.ExecutionBlock{
					Hash: common.BytesToHash([]byte("bar")),
				},
				version: version.Deneb,
			},
			err: "does not match execution block hash",
		},
		{
			name: "ok",
			args: args{
				header: &pb.ExecutionPayloadHeaderDeneb{
					BlockHash: wantedHash[:],
				},
				block: &pb.ExecutionBlock{
					Hash: wantedHash,
				},
				version: version.Deneb,
			},
			want: func() interfaces.ExecutionData {
				p, err := blocks.WrappedExecutionPayloadDeneb(&pb.ExecutionPayloadDeneb{
					BlockHash:    wantedHash[:],
					Transactions: [][]byte{},
				}, big.NewInt(0))
				require.NoError(t, err)
				return p
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			wrapped, err := blocks.WrappedExecutionPayloadHeaderDeneb(tt.args.header, big.NewInt(0))
			require.NoError(t, err)
			got, err := fullPayloadFromExecutionBlock(tt.args.version, wrapped, tt.args.block)
			if err != nil {
				assert.ErrorContains(t, tt.err, err)
			} else {
				assert.DeepEqual(t, tt.want(), got)
			}
		})
	}
}

func TestHeaderByHash_NotFound(t *testing.T) {
	srv := &Service{}
	srv.rpcClient = RPCClientBad{}

	_, err := srv.HeaderByHash(context.Background(), [32]byte{})
	assert.Equal(t, ethereum.NotFound, err)
}

func TestHeaderByNumber_NotFound(t *testing.T) {
	srv := &Service{}
	srv.rpcClient = RPCClientBad{}

	_, err := srv.HeaderByNumber(context.Background(), big.NewInt(100))
	assert.Equal(t, ethereum.NotFound, err)
}

func TestToBlockNumArg(t *testing.T) {
	tests := []struct {
		name   string
		number *big.Int
		want   string
	}{
		{
			name:   "genesis",
			number: big.NewInt(0),
			want:   "0x0",
		},
		{
			name:   "near genesis block",
			number: big.NewInt(300),
			want:   "0x12c",
		},
		{
			name:   "current block",
			number: big.NewInt(15838075),
			want:   "0xf1ab7b",
		},
		{
			name:   "far off block",
			number: big.NewInt(12032894823020),
			want:   "0xaf1a06bea6c",
		},
		{
			name:   "latest block",
			number: nil,
			want:   "latest",
		},
		{
			name:   "pending block",
			number: big.NewInt(-1),
			want:   "pending",
		},
		{
			name:   "finalized block",
			number: big.NewInt(-3),
			want:   "finalized",
		},
		{
			name:   "safe block",
			number: big.NewInt(-4),
			want:   "safe",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := toBlockNumArg(tt.number); got != tt.want {
				t.Errorf("toBlockNumArg() = %v, want %v", got, tt.want)
			}
		})
	}
}

type testEngineService struct{}

func (*testEngineService) NoArgsRets() {}

func (*testEngineService) GetBlockByHash(
	_ context.Context, _ common.Hash, _ bool,
) *pb.ExecutionBlock {
	fix := fixtures()
	item, ok := fix["ExecutionBlock"].(*pb.ExecutionBlock)
	if !ok {
		panic("not found")
	}
	return item
}

func (*testEngineService) GetBlockByNumber(
	_ context.Context, _ string, _ bool,
) *pb.ExecutionBlock {
	fix := fixtures()
	item, ok := fix["ExecutionBlock"].(*pb.ExecutionBlock)
	if !ok {
		panic("not found")
	}
	return item
}

func (*testEngineService) GetPayloadV1(
	_ context.Context, _ pb.PayloadIDBytes,
) *pb.ExecutionPayload {
	fix := fixtures()
	item, ok := fix["ExecutionPayload"].(*pb.ExecutionPayload)
	if !ok {
		panic("not found")
	}
	return item
}

func (*testEngineService) GetPayloadV2(
	_ context.Context, _ pb.PayloadIDBytes,
) *pb.ExecutionPayloadCapellaWithValue {
	fix := fixtures()
	item, ok := fix["ExecutionPayloadCapellaWithValue"].(*pb.ExecutionPayloadCapellaWithValue)
	if !ok {
		panic("not found")
	}
	return item
}

func (*testEngineService) ForkchoiceUpdatedV1(
	_ context.Context, _ *pb.ForkchoiceState, _ *pb.PayloadAttributes,
) *ForkchoiceUpdatedResponse {
	fix := fixtures()
	item, ok := fix["ForkchoiceUpdatedResponse"].(*ForkchoiceUpdatedResponse)
	if !ok {
		panic("not found")
	}
	item.Status.Status = pb.PayloadStatus_VALID
	return item
}

func (*testEngineService) ForkchoiceUpdatedV2(
	_ context.Context, _ *pb.ForkchoiceState, _ *pb.PayloadAttributes,
) *ForkchoiceUpdatedResponse {
	fix := fixtures()
	item, ok := fix["ForkchoiceUpdatedResponse"].(*ForkchoiceUpdatedResponse)
	if !ok {
		panic("not found")
	}
	item.Status.Status = pb.PayloadStatus_VALID
	return item
}

func (*testEngineService) NewPayloadV1(
	_ context.Context, _ *pb.ExecutionPayload,
) *pb.PayloadStatus {
	fix := fixtures()
	item, ok := fix["ValidPayloadStatus"].(*pb.PayloadStatus)
	if !ok {
		panic("not found")
	}
	return item
}

func (*testEngineService) NewPayloadV2(
	_ context.Context, _ *pb.ExecutionPayloadCapella,
) *pb.PayloadStatus {
	fix := fixtures()
	item, ok := fix["ValidPayloadStatus"].(*pb.PayloadStatus)
	if !ok {
		panic("not found")
	}
	return item
}

func forkchoiceUpdateSetup(t *testing.T, fcs *pb.ForkchoiceState, att *pb.PayloadAttributes, res *ForkchoiceUpdatedResponse) *Service {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		defer func() {
			require.NoError(t, r.Body.Close())
		}()
		enc, err := io.ReadAll(r.Body)
		require.NoError(t, err)
		jsonRequestString := string(enc)

		forkChoiceStateReq, err := json.Marshal(fcs)
		require.NoError(t, err)
		payloadAttrsReq, err := json.Marshal(att)
		require.NoError(t, err)

		// We expect the JSON string RPC request contains the right arguments.
		require.Equal(t, true, strings.Contains(
			jsonRequestString, string(forkChoiceStateReq),
		))
		require.Equal(t, true, strings.Contains(
			jsonRequestString, string(payloadAttrsReq),
		))
		resp := map[string]interface{}{
			"jsonrpc": "2.0",
			"id":      1,
			"result":  res,
		}
		err = json.NewEncoder(w).Encode(resp)
		require.NoError(t, err)
	}))

	rpcClient, err := rpc.DialHTTP(srv.URL)
	require.NoError(t, err)

	service := &Service{}
	service.rpcClient = rpcClient
	return service
}

func forkchoiceUpdateSetupV2(t *testing.T, fcs *pb.ForkchoiceState, att *pb.PayloadAttributesV2, res *ForkchoiceUpdatedResponse) *Service {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		defer func() {
			require.NoError(t, r.Body.Close())
		}()
		enc, err := io.ReadAll(r.Body)
		require.NoError(t, err)
		jsonRequestString := string(enc)

		forkChoiceStateReq, err := json.Marshal(fcs)
		require.NoError(t, err)
		payloadAttrsReq, err := json.Marshal(att)
		require.NoError(t, err)

		// We expect the JSON string RPC request contains the right arguments.
		require.Equal(t, true, strings.Contains(
			jsonRequestString, string(forkChoiceStateReq),
		))
		require.Equal(t, true, strings.Contains(
			jsonRequestString, string(payloadAttrsReq),
		))
		resp := map[string]interface{}{
			"jsonrpc": "2.0",
			"id":      1,
			"result":  res,
		}
		err = json.NewEncoder(w).Encode(resp)
		require.NoError(t, err)
	}))

	rpcClient, err := rpc.DialHTTP(srv.URL)
	require.NoError(t, err)

	service := &Service{}
	service.rpcClient = rpcClient
	return service
}

func newPayloadSetup(t *testing.T, status *pb.PayloadStatus, payload *pb.ExecutionPayload) *Service {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		defer func() {
			require.NoError(t, r.Body.Close())
		}()
		enc, err := io.ReadAll(r.Body)
		require.NoError(t, err)
		jsonRequestString := string(enc)

		reqArg, err := json.Marshal(payload)
		require.NoError(t, err)

		// We expect the JSON string RPC request contains the right arguments.
		require.Equal(t, true, strings.Contains(
			jsonRequestString, string(reqArg),
		))
		resp := map[string]interface{}{
			"jsonrpc": "2.0",
			"id":      1,
			"result":  status,
		}
		err = json.NewEncoder(w).Encode(resp)
		require.NoError(t, err)
	}))

	rpcClient, err := rpc.DialHTTP(srv.URL)
	require.NoError(t, err)

	service := &Service{}
	service.rpcClient = rpcClient
	return service
}

func newPayloadV2Setup(t *testing.T, status *pb.PayloadStatus, payload *pb.ExecutionPayloadCapella) *Service {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		defer func() {
			require.NoError(t, r.Body.Close())
		}()
		enc, err := io.ReadAll(r.Body)
		require.NoError(t, err)
		jsonRequestString := string(enc)

		reqArg, err := json.Marshal(payload)
		require.NoError(t, err)

		// We expect the JSON string RPC request contains the right arguments.
		require.Equal(t, true, strings.Contains(
			jsonRequestString, string(reqArg),
		))
		resp := map[string]interface{}{
			"jsonrpc": "2.0",
			"id":      1,
			"result":  status,
		}
		err = json.NewEncoder(w).Encode(resp)
		require.NoError(t, err)
	}))

	rpcClient, err := rpc.DialHTTP(srv.URL)
	require.NoError(t, err)

	service := &Service{}
	service.rpcClient = rpcClient
	return service
}

func newPayloadV3Setup(t *testing.T, status *pb.PayloadStatus, payload *pb.ExecutionPayloadDeneb) *Service {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		defer func() {
			require.NoError(t, r.Body.Close())
		}()
		enc, err := io.ReadAll(r.Body)
		require.NoError(t, err)
		jsonRequestString := string(enc)

		reqArg, err := json.Marshal(payload)
		require.NoError(t, err)

		// We expect the JSON string RPC request contains the right arguments.
		require.Equal(t, true, strings.Contains(
			jsonRequestString, string(reqArg),
		))
		resp := map[string]interface{}{
			"jsonrpc": "2.0",
			"id":      1,
			"result":  status,
		}
		err = json.NewEncoder(w).Encode(resp)
		require.NoError(t, err)
	}))

	rpcClient, err := rpc.DialHTTP(srv.URL)
	require.NoError(t, err)

	service := &Service{}
	service.rpcClient = rpcClient
	return service
}

func TestCapella_PayloadBodiesByHash(t *testing.T) {
	t.Run("empty response works", func(t *testing.T) {
		srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			defer func() {
				require.NoError(t, r.Body.Close())
			}()
			executionPayloadBodies := make([]*pb.ExecutionPayloadBodyV1, 0)
			resp := map[string]interface{}{
				"jsonrpc": "2.0",
				"id":      1,
				"result":  executionPayloadBodies,
			}
			err := json.NewEncoder(w).Encode(resp)
			require.NoError(t, err)
		}))
		ctx := context.Background()

		rpcClient, err := rpc.DialHTTP(srv.URL)
		require.NoError(t, err)

		service := &Service{}
		service.rpcClient = rpcClient

		results, err := service.GetPayloadBodiesByHash(ctx, []common.Hash{})
		require.NoError(t, err)
		require.Equal(t, 0, len(results))

		for _, item := range results {
			require.NotNil(t, item)
		}
	})
	t.Run("single element response null works", func(t *testing.T) {
		srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			defer func() {
				require.NoError(t, r.Body.Close())
			}()
			executionPayloadBodies := make([]*pb.ExecutionPayloadBodyV1, 1)
			executionPayloadBodies[0] = nil

			resp := map[string]interface{}{
				"jsonrpc": "2.0",
				"id":      1,
				"result":  executionPayloadBodies,
			}
			err := json.NewEncoder(w).Encode(resp)
			require.NoError(t, err)
		}))
		ctx := context.Background()

		rpcClient, err := rpc.DialHTTP(srv.URL)
		require.NoError(t, err)

		service := &Service{}
		service.rpcClient = rpcClient

		bRoot := [32]byte{}
		copy(bRoot[:], "hash")
		results, err := service.GetPayloadBodiesByHash(ctx, []common.Hash{bRoot})
		require.NoError(t, err)
		require.Equal(t, 1, len(results))

		for _, item := range results {
			require.NotNil(t, item)
		}
	})
	t.Run("empty, null, full works", func(t *testing.T) {
		srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			defer func() {
				require.NoError(t, r.Body.Close())
			}()
			executionPayloadBodies := make([]*pb.ExecutionPayloadBodyV1, 3)
			executionPayloadBodies[0] = &pb.ExecutionPayloadBodyV1{
				Transactions: [][]byte{},
				Withdrawals:  []*pb.Withdrawal{},
			}
			executionPayloadBodies[1] = nil
			executionPayloadBodies[2] = &pb.ExecutionPayloadBodyV1{
				Transactions: [][]byte{hexutil.MustDecode("0x02f878831469668303f51d843b9ac9f9843b9aca0082520894c93269b73096998db66be0441e836d873535cb9c8894a19041886f000080c001a031cc29234036afbf9a1fb9476b463367cb1f957ac0b919b69bbc798436e604aaa018c4e9c3914eb27aadd0b91e10b18655739fcf8c1fc398763a9f1beecb8ddc86")},
				Withdrawals: []*pb.Withdrawal{{
					Index:          1,
					ValidatorIndex: 1,
					Address:        hexutil.MustDecode("0xcf8e0d4e9587369b2301d0790347320302cc0943"),
					Amount:         1,
				}},
			}

			resp := map[string]interface{}{
				"jsonrpc": "2.0",
				"id":      1,
				"result":  executionPayloadBodies,
			}
			err := json.NewEncoder(w).Encode(resp)
			require.NoError(t, err)
		}))
		ctx := context.Background()

		rpcClient, err := rpc.DialHTTP(srv.URL)
		require.NoError(t, err)

		service := &Service{}
		service.rpcClient = rpcClient

		bRoot := [32]byte{}
		copy(bRoot[:], "hash")
		results, err := service.GetPayloadBodiesByHash(ctx, []common.Hash{bRoot, bRoot, bRoot})
		require.NoError(t, err)
		require.Equal(t, 3, len(results))

		for _, item := range results {
			require.NotNil(t, item)
		}
	})
	t.Run("full works, single item", func(t *testing.T) {
		srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			defer func() {
				require.NoError(t, r.Body.Close())
			}()
			executionPayloadBodies := make([]*pb.ExecutionPayloadBodyV1, 1)
			executionPayloadBodies[0] = &pb.ExecutionPayloadBodyV1{
				Transactions: [][]byte{hexutil.MustDecode("0x02f878831469668303f51d843b9ac9f9843b9aca0082520894c93269b73096998db66be0441e836d873535cb9c8894a19041886f000080c001a031cc29234036afbf9a1fb9476b463367cb1f957ac0b919b69bbc798436e604aaa018c4e9c3914eb27aadd0b91e10b18655739fcf8c1fc398763a9f1beecb8ddc86")},
				Withdrawals: []*pb.Withdrawal{{
					Index:          1,
					ValidatorIndex: 1,
					Address:        hexutil.MustDecode("0xcf8e0d4e9587369b2301d0790347320302cc0943"),
					Amount:         1,
				}},
			}

			resp := map[string]interface{}{
				"jsonrpc": "2.0",
				"id":      1,
				"result":  executionPayloadBodies,
			}
			err := json.NewEncoder(w).Encode(resp)
			require.NoError(t, err)
		}))
		ctx := context.Background()

		rpcClient, err := rpc.DialHTTP(srv.URL)
		require.NoError(t, err)

		service := &Service{}
		service.rpcClient = rpcClient

		bRoot := [32]byte{}
		copy(bRoot[:], "hash")
		results, err := service.GetPayloadBodiesByHash(ctx, []common.Hash{bRoot})
		require.NoError(t, err)
		require.Equal(t, 1, len(results))

		for _, item := range results {
			require.NotNil(t, item)
		}
	})
	t.Run("full works, multiple items", func(t *testing.T) {
		srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			defer func() {
				require.NoError(t, r.Body.Close())
			}()
			executionPayloadBodies := make([]*pb.ExecutionPayloadBodyV1, 2)
			executionPayloadBodies[0] = &pb.ExecutionPayloadBodyV1{
				Transactions: [][]byte{hexutil.MustDecode("0x02f878831469668303f51d843b9ac9f9843b9aca0082520894c93269b73096998db66be0441e836d873535cb9c8894a19041886f000080c001a031cc29234036afbf9a1fb9476b463367cb1f957ac0b919b69bbc798436e604aaa018c4e9c3914eb27aadd0b91e10b18655739fcf8c1fc398763a9f1beecb8ddc86")},
				Withdrawals: []*pb.Withdrawal{{
					Index:          1,
					ValidatorIndex: 1,
					Address:        hexutil.MustDecode("0xcf8e0d4e9587369b2301d0790347320302cc0943"),
					Amount:         1,
				}},
			}
			executionPayloadBodies[1] = &pb.ExecutionPayloadBodyV1{
				Transactions: [][]byte{hexutil.MustDecode("0x02f878831469668303f51d843b9ac9f9843b9aca0082520894c93269b73096998db66be0441e836d873535cb9c8894a19041886f000080c001a031cc29234036afbf9a1fb9476b463367cb1f957ac0b919b69bbc798436e604aaa018c4e9c3914eb27aadd0b91e10b18655739fcf8c1fc398763a9f1beecb8ddc86")},
				Withdrawals: []*pb.Withdrawal{{
					Index:          2,
					ValidatorIndex: 1,
					Address:        hexutil.MustDecode("0xcf8e0d4e9587369b2301d0790347320302cc0943"),
					Amount:         1,
				}},
			}

			resp := map[string]interface{}{
				"jsonrpc": "2.0",
				"id":      1,
				"result":  executionPayloadBodies,
			}
			err := json.NewEncoder(w).Encode(resp)
			require.NoError(t, err)
		}))
		ctx := context.Background()

		rpcClient, err := rpc.DialHTTP(srv.URL)
		require.NoError(t, err)

		service := &Service{}
		service.rpcClient = rpcClient

		bRoot := [32]byte{}
		copy(bRoot[:], "hash")
		results, err := service.GetPayloadBodiesByHash(ctx, []common.Hash{bRoot})
		require.NoError(t, err)
		require.Equal(t, 2, len(results))

		for _, item := range results {
			require.NotNil(t, item)
		}
	})
	t.Run("returning empty, null, empty should work properly", func(t *testing.T) {
		srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			defer func() {
				require.NoError(t, r.Body.Close())
			}()
			// [A, B, C] but no B in the server means
			// we get [Abody, null, Cbody].
			executionPayloadBodies := make([]*pb.ExecutionPayloadBodyV1, 3)
			executionPayloadBodies[0] = &pb.ExecutionPayloadBodyV1{
				Transactions: [][]byte{},
				Withdrawals:  []*pb.Withdrawal{},
			}
			executionPayloadBodies[1] = nil
			executionPayloadBodies[2] = &pb.ExecutionPayloadBodyV1{
				Transactions: [][]byte{},
				Withdrawals:  []*pb.Withdrawal{},
			}

			resp := map[string]interface{}{
				"jsonrpc": "2.0",
				"id":      1,
				"result":  executionPayloadBodies,
			}
			err := json.NewEncoder(w).Encode(resp)
			require.NoError(t, err)
		}))
		ctx := context.Background()

		rpcClient, err := rpc.DialHTTP(srv.URL)
		require.NoError(t, err)

		service := &Service{}
		service.rpcClient = rpcClient

		bRoot := [32]byte{}
		copy(bRoot[:], "hash")
		results, err := service.GetPayloadBodiesByHash(ctx, []common.Hash{bRoot, bRoot, bRoot})
		require.NoError(t, err)
		require.Equal(t, 3, len(results))

		for _, item := range results {
			require.NotNil(t, item)
		}
	})
}

func TestCapella_PayloadBodiesByRange(t *testing.T) {
	t.Run("empty response works", func(t *testing.T) {
		srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			defer func() {
				require.NoError(t, r.Body.Close())
			}()
			executionPayloadBodies := make([]*pb.ExecutionPayloadBodyV1, 0)
			resp := map[string]interface{}{
				"jsonrpc": "2.0",
				"id":      1,
				"result":  executionPayloadBodies,
			}
			err := json.NewEncoder(w).Encode(resp)
			require.NoError(t, err)
		}))
		ctx := context.Background()

		rpcClient, err := rpc.DialHTTP(srv.URL)
		require.NoError(t, err)

		service := &Service{}
		service.rpcClient = rpcClient

		results, err := service.GetPayloadBodiesByRange(ctx, uint64(1), uint64(2))
		require.NoError(t, err)
		require.Equal(t, 0, len(results))

		for _, item := range results {
			require.NotNil(t, item)
		}
	})
	t.Run("single element response null works", func(t *testing.T) {
		srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			defer func() {
				require.NoError(t, r.Body.Close())
			}()
			executionPayloadBodies := make([]*pb.ExecutionPayloadBodyV1, 1)
			executionPayloadBodies[0] = nil

			resp := map[string]interface{}{
				"jsonrpc": "2.0",
				"id":      1,
				"result":  executionPayloadBodies,
			}
			err := json.NewEncoder(w).Encode(resp)
			require.NoError(t, err)
		}))
		ctx := context.Background()

		rpcClient, err := rpc.DialHTTP(srv.URL)
		require.NoError(t, err)

		service := &Service{}
		service.rpcClient = rpcClient

		results, err := service.GetPayloadBodiesByRange(ctx, uint64(1), uint64(2))
		require.NoError(t, err)
		require.Equal(t, 1, len(results))

		for _, item := range results {
			require.NotNil(t, item)
		}
	})
	t.Run("empty, null, full works", func(t *testing.T) {
		srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			defer func() {
				require.NoError(t, r.Body.Close())
			}()
			executionPayloadBodies := make([]*pb.ExecutionPayloadBodyV1, 3)
			executionPayloadBodies[0] = &pb.ExecutionPayloadBodyV1{
				Transactions: [][]byte{},
				Withdrawals:  []*pb.Withdrawal{},
			}
			executionPayloadBodies[1] = nil
			executionPayloadBodies[2] = &pb.ExecutionPayloadBodyV1{
				Transactions: [][]byte{hexutil.MustDecode("0x02f878831469668303f51d843b9ac9f9843b9aca0082520894c93269b73096998db66be0441e836d873535cb9c8894a19041886f000080c001a031cc29234036afbf9a1fb9476b463367cb1f957ac0b919b69bbc798436e604aaa018c4e9c3914eb27aadd0b91e10b18655739fcf8c1fc398763a9f1beecb8ddc86")},
				Withdrawals: []*pb.Withdrawal{{
					Index:          1,
					ValidatorIndex: 1,
					Address:        hexutil.MustDecode("0xcf8e0d4e9587369b2301d0790347320302cc0943"),
					Amount:         1,
				}},
			}

			resp := map[string]interface{}{
				"jsonrpc": "2.0",
				"id":      1,
				"result":  executionPayloadBodies,
			}
			err := json.NewEncoder(w).Encode(resp)
			require.NoError(t, err)
		}))
		ctx := context.Background()

		rpcClient, err := rpc.DialHTTP(srv.URL)
		require.NoError(t, err)

		service := &Service{}
		service.rpcClient = rpcClient

		results, err := service.GetPayloadBodiesByRange(ctx, uint64(1), uint64(2))
		require.NoError(t, err)
		require.Equal(t, 3, len(results))

		for _, item := range results {
			require.NotNil(t, item)
		}
	})
	t.Run("full works, single item", func(t *testing.T) {
		srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			defer func() {
				require.NoError(t, r.Body.Close())
			}()
			executionPayloadBodies := make([]*pb.ExecutionPayloadBodyV1, 1)
			executionPayloadBodies[0] = &pb.ExecutionPayloadBodyV1{
				Transactions: [][]byte{hexutil.MustDecode("0x02f878831469668303f51d843b9ac9f9843b9aca0082520894c93269b73096998db66be0441e836d873535cb9c8894a19041886f000080c001a031cc29234036afbf9a1fb9476b463367cb1f957ac0b919b69bbc798436e604aaa018c4e9c3914eb27aadd0b91e10b18655739fcf8c1fc398763a9f1beecb8ddc86")},
				Withdrawals: []*pb.Withdrawal{{
					Index:          1,
					ValidatorIndex: 1,
					Address:        hexutil.MustDecode("0xcf8e0d4e9587369b2301d0790347320302cc0943"),
					Amount:         1,
				}},
			}

			resp := map[string]interface{}{
				"jsonrpc": "2.0",
				"id":      1,
				"result":  executionPayloadBodies,
			}
			err := json.NewEncoder(w).Encode(resp)
			require.NoError(t, err)
		}))
		ctx := context.Background()

		rpcClient, err := rpc.DialHTTP(srv.URL)
		require.NoError(t, err)

		service := &Service{}
		service.rpcClient = rpcClient

		results, err := service.GetPayloadBodiesByRange(ctx, uint64(1), uint64(2))
		require.NoError(t, err)
		require.Equal(t, 1, len(results))

		for _, item := range results {
			require.NotNil(t, item)
		}
	})
	t.Run("full works, multiple items", func(t *testing.T) {
		srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			defer func() {
				require.NoError(t, r.Body.Close())
			}()
			executionPayloadBodies := make([]*pb.ExecutionPayloadBodyV1, 2)
			executionPayloadBodies[0] = &pb.ExecutionPayloadBodyV1{
				Transactions: [][]byte{hexutil.MustDecode("0x02f878831469668303f51d843b9ac9f9843b9aca0082520894c93269b73096998db66be0441e836d873535cb9c8894a19041886f000080c001a031cc29234036afbf9a1fb9476b463367cb1f957ac0b919b69bbc798436e604aaa018c4e9c3914eb27aadd0b91e10b18655739fcf8c1fc398763a9f1beecb8ddc86")},
				Withdrawals: []*pb.Withdrawal{{
					Index:          1,
					ValidatorIndex: 1,
					Address:        hexutil.MustDecode("0xcf8e0d4e9587369b2301d0790347320302cc0943"),
					Amount:         1,
				}},
			}
			executionPayloadBodies[1] = &pb.ExecutionPayloadBodyV1{
				Transactions: [][]byte{hexutil.MustDecode("0x02f878831469668303f51d843b9ac9f9843b9aca0082520894c93269b73096998db66be0441e836d873535cb9c8894a19041886f000080c001a031cc29234036afbf9a1fb9476b463367cb1f957ac0b919b69bbc798436e604aaa018c4e9c3914eb27aadd0b91e10b18655739fcf8c1fc398763a9f1beecb8ddc86")},
				Withdrawals: []*pb.Withdrawal{{
					Index:          2,
					ValidatorIndex: 1,
					Address:        hexutil.MustDecode("0xcf8e0d4e9587369b2301d0790347320302cc0943"),
					Amount:         1,
				}},
			}

			resp := map[string]interface{}{
				"jsonrpc": "2.0",
				"id":      1,
				"result":  executionPayloadBodies,
			}
			err := json.NewEncoder(w).Encode(resp)
			require.NoError(t, err)
		}))
		ctx := context.Background()

		rpcClient, err := rpc.DialHTTP(srv.URL)
		require.NoError(t, err)

		service := &Service{}
		service.rpcClient = rpcClient

		results, err := service.GetPayloadBodiesByRange(ctx, uint64(1), uint64(2))
		require.NoError(t, err)
		require.Equal(t, 2, len(results))

		for _, item := range results {
			require.NotNil(t, item)
		}
	})
	t.Run("returning empty, null, empty should work properly", func(t *testing.T) {
		srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			defer func() {
				require.NoError(t, r.Body.Close())
			}()
			// [A, B, C] but no B in the server means
			// we get [Abody, null, Cbody].
			executionPayloadBodies := make([]*pb.ExecutionPayloadBodyV1, 3)
			executionPayloadBodies[0] = &pb.ExecutionPayloadBodyV1{
				Transactions: [][]byte{},
				Withdrawals:  []*pb.Withdrawal{},
			}
			executionPayloadBodies[1] = nil
			executionPayloadBodies[2] = &pb.ExecutionPayloadBodyV1{
				Transactions: [][]byte{},
				Withdrawals:  []*pb.Withdrawal{},
			}

			resp := map[string]interface{}{
				"jsonrpc": "2.0",
				"id":      1,
				"result":  executionPayloadBodies,
			}
			err := json.NewEncoder(w).Encode(resp)
			require.NoError(t, err)
		}))
		ctx := context.Background()

		rpcClient, err := rpc.DialHTTP(srv.URL)
		require.NoError(t, err)

		service := &Service{}
		service.rpcClient = rpcClient

		results, err := service.GetPayloadBodiesByRange(ctx, uint64(1), uint64(2))
		require.NoError(t, err)
		require.Equal(t, 3, len(results))

		for _, item := range results {
			require.NotNil(t, item)
		}
	})
}

func Test_ExchangeCapabilities(t *testing.T) {
	t.Run("empty response works", func(t *testing.T) {
		srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			defer func() {
				require.NoError(t, r.Body.Close())
			}()
			exchangeCapabilities := &pb.ExchangeCapabilities{}
			resp := map[string]interface{}{
				"jsonrpc": "2.0",
				"id":      1,
				"result":  exchangeCapabilities,
			}
			err := json.NewEncoder(w).Encode(resp)
			require.NoError(t, err)
		}))
		ctx := context.Background()
		logHook := logTest.NewGlobal()

		rpcClient, err := rpc.DialHTTP(srv.URL)
		require.NoError(t, err)

		service := &Service{}
		service.rpcClient = rpcClient

		results, err := service.ExchangeCapabilities(ctx)
		require.NoError(t, err)
		require.Equal(t, 0, len(results))

		for _, item := range results {
			require.NotNil(t, item)
		}
		assert.LogsContain(t, logHook, "Please update client, detected the following unsupported engine methods:")
	})
	t.Run("list of items", func(t *testing.T) {
		srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			defer func() {
				require.NoError(t, r.Body.Close())
			}()
			exchangeCapabilities := &pb.ExchangeCapabilities{
				SupportedMethods: []string{"A", "B", "C"},
			}

			resp := map[string]interface{}{
				"jsonrpc": "2.0",
				"id":      1,
				"result":  exchangeCapabilities,
			}
			err := json.NewEncoder(w).Encode(resp)
			require.NoError(t, err)
		}))
		ctx := context.Background()

		rpcClient, err := rpc.DialHTTP(srv.URL)
		require.NoError(t, err)

		service := &Service{}
		service.rpcClient = rpcClient

		results, err := service.ExchangeCapabilities(ctx)
		require.NoError(t, err)
		require.Equal(t, 3, len(results))

		for _, item := range results {
			require.NotNil(t, item)
		}
	})
}
