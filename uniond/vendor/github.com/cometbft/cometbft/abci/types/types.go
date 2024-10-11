//nolint:stylecheck,revive
package types

import (
	"encoding/json"

	"github.com/cosmos/gogoproto/grpc"

	v1 "github.com/cometbft/cometbft/api/cometbft/abci/v1"
)

type (
	Request                    = v1.Request
	EchoRequest                = v1.EchoRequest
	FlushRequest               = v1.FlushRequest
	InfoRequest                = v1.InfoRequest
	InitChainRequest           = v1.InitChainRequest
	QueryRequest               = v1.QueryRequest
	CheckTxRequest             = v1.CheckTxRequest
	CommitRequest              = v1.CommitRequest
	ListSnapshotsRequest       = v1.ListSnapshotsRequest
	OfferSnapshotRequest       = v1.OfferSnapshotRequest
	LoadSnapshotChunkRequest   = v1.LoadSnapshotChunkRequest
	ApplySnapshotChunkRequest  = v1.ApplySnapshotChunkRequest
	PrepareProposalRequest     = v1.PrepareProposalRequest
	ProcessProposalRequest     = v1.ProcessProposalRequest
	ExtendVoteRequest          = v1.ExtendVoteRequest
	VerifyVoteExtensionRequest = v1.VerifyVoteExtensionRequest
	FinalizeBlockRequest       = v1.FinalizeBlockRequest
)

// Discriminated Request variants are defined in the latest proto package.
type (
	Request_Echo                = v1.Request_Echo
	Request_Flush               = v1.Request_Flush
	Request_Info                = v1.Request_Info
	Request_InitChain           = v1.Request_InitChain
	Request_Query               = v1.Request_Query
	Request_CheckTx             = v1.Request_CheckTx
	Request_Commit              = v1.Request_Commit
	Request_ListSnapshots       = v1.Request_ListSnapshots
	Request_OfferSnapshot       = v1.Request_OfferSnapshot
	Request_LoadSnapshotChunk   = v1.Request_LoadSnapshotChunk
	Request_ApplySnapshotChunk  = v1.Request_ApplySnapshotChunk
	Request_PrepareProposal     = v1.Request_PrepareProposal
	Request_ProcessProposal     = v1.Request_ProcessProposal
	Request_ExtendVote          = v1.Request_ExtendVote
	Request_VerifyVoteExtension = v1.Request_VerifyVoteExtension
	Request_FinalizeBlock       = v1.Request_FinalizeBlock
)

type (
	Response                    = v1.Response
	ExceptionResponse           = v1.ExceptionResponse
	EchoResponse                = v1.EchoResponse
	FlushResponse               = v1.FlushResponse
	InfoResponse                = v1.InfoResponse
	InitChainResponse           = v1.InitChainResponse
	QueryResponse               = v1.QueryResponse
	CheckTxResponse             = v1.CheckTxResponse
	CommitResponse              = v1.CommitResponse
	ListSnapshotsResponse       = v1.ListSnapshotsResponse
	OfferSnapshotResponse       = v1.OfferSnapshotResponse
	LoadSnapshotChunkResponse   = v1.LoadSnapshotChunkResponse
	ApplySnapshotChunkResponse  = v1.ApplySnapshotChunkResponse
	PrepareProposalResponse     = v1.PrepareProposalResponse
	ProcessProposalResponse     = v1.ProcessProposalResponse
	ExtendVoteResponse          = v1.ExtendVoteResponse
	VerifyVoteExtensionResponse = v1.VerifyVoteExtensionResponse
	FinalizeBlockResponse       = v1.FinalizeBlockResponse
)

// Discriminated Response variants are defined in the latest proto package.
type (
	Response_Exception           = v1.Response_Exception
	Response_Echo                = v1.Response_Echo
	Response_Flush               = v1.Response_Flush
	Response_Info                = v1.Response_Info
	Response_InitChain           = v1.Response_InitChain
	Response_Query               = v1.Response_Query
	Response_CheckTx             = v1.Response_CheckTx
	Response_Commit              = v1.Response_Commit
	Response_ListSnapshots       = v1.Response_ListSnapshots
	Response_OfferSnapshot       = v1.Response_OfferSnapshot
	Response_LoadSnapshotChunk   = v1.Response_LoadSnapshotChunk
	Response_ApplySnapshotChunk  = v1.Response_ApplySnapshotChunk
	Response_PrepareProposal     = v1.Response_PrepareProposal
	Response_ProcessProposal     = v1.Response_ProcessProposal
	Response_ExtendVote          = v1.Response_ExtendVote
	Response_VerifyVoteExtension = v1.Response_VerifyVoteExtension
	Response_FinalizeBlock       = v1.Response_FinalizeBlock
)

type (
	CommitInfo         = v1.CommitInfo
	ExecTxResult       = v1.ExecTxResult
	ExtendedCommitInfo = v1.ExtendedCommitInfo
	ExtendedVoteInfo   = v1.ExtendedVoteInfo
	Event              = v1.Event
	EventAttribute     = v1.EventAttribute
	Misbehavior        = v1.Misbehavior
	Snapshot           = v1.Snapshot
	TxResult           = v1.TxResult
	Validator          = v1.Validator
	ValidatorUpdate    = v1.ValidatorUpdate
	VoteInfo           = v1.VoteInfo
)

type (
	ABCIServiceClient = v1.ABCIServiceClient
	ABCIServiceServer = v1.ABCIServiceServer
)

func NewABCIClient(cc grpc.ClientConn) ABCIServiceClient {
	return v1.NewABCIServiceClient(cc)
}

func RegisterABCIServer(s grpc.Server, srv ABCIServiceServer) {
	v1.RegisterABCIServiceServer(s, srv)
}

type CheckTxType = v1.CheckTxType

const (
	CHECK_TX_TYPE_UNKNOWN CheckTxType = v1.CHECK_TX_TYPE_UNKNOWN
	CHECK_TX_TYPE_CHECK   CheckTxType = v1.CHECK_TX_TYPE_CHECK
	CHECK_TX_TYPE_RECHECK CheckTxType = v1.CHECK_TX_TYPE_RECHECK
)

type MisbehaviorType = v1.MisbehaviorType

const (
	MISBEHAVIOR_TYPE_UNKNOWN             MisbehaviorType = v1.MISBEHAVIOR_TYPE_UNKNOWN
	MISBEHAVIOR_TYPE_DUPLICATE_VOTE      MisbehaviorType = v1.MISBEHAVIOR_TYPE_DUPLICATE_VOTE
	MISBEHAVIOR_TYPE_LIGHT_CLIENT_ATTACK MisbehaviorType = v1.MISBEHAVIOR_TYPE_LIGHT_CLIENT_ATTACK
)

type ApplySnapshotChunkResult = v1.ApplySnapshotChunkResult

const (
	APPLY_SNAPSHOT_CHUNK_RESULT_UNKNOWN         ApplySnapshotChunkResult = v1.APPLY_SNAPSHOT_CHUNK_RESULT_UNKNOWN
	APPLY_SNAPSHOT_CHUNK_RESULT_ACCEPT          ApplySnapshotChunkResult = v1.APPLY_SNAPSHOT_CHUNK_RESULT_ACCEPT
	APPLY_SNAPSHOT_CHUNK_RESULT_ABORT           ApplySnapshotChunkResult = v1.APPLY_SNAPSHOT_CHUNK_RESULT_ABORT
	APPLY_SNAPSHOT_CHUNK_RESULT_RETRY           ApplySnapshotChunkResult = v1.APPLY_SNAPSHOT_CHUNK_RESULT_RETRY
	APPLY_SNAPSHOT_CHUNK_RESULT_RETRY_SNAPSHOT  ApplySnapshotChunkResult = v1.APPLY_SNAPSHOT_CHUNK_RESULT_RETRY_SNAPSHOT
	APPLY_SNAPSHOT_CHUNK_RESULT_REJECT_SNAPSHOT ApplySnapshotChunkResult = v1.APPLY_SNAPSHOT_CHUNK_RESULT_REJECT_SNAPSHOT
)

type OfferSnapshotResult = v1.OfferSnapshotResult

const (
	OFFER_SNAPSHOT_RESULT_UNKNOWN       OfferSnapshotResult = v1.OFFER_SNAPSHOT_RESULT_UNKNOWN
	OFFER_SNAPSHOT_RESULT_ACCEPT        OfferSnapshotResult = v1.OFFER_SNAPSHOT_RESULT_ACCEPT
	OFFER_SNAPSHOT_RESULT_ABORT         OfferSnapshotResult = v1.OFFER_SNAPSHOT_RESULT_ABORT
	OFFER_SNAPSHOT_RESULT_REJECT        OfferSnapshotResult = v1.OFFER_SNAPSHOT_RESULT_REJECT
	OFFER_SNAPSHOT_RESULT_REJECT_FORMAT OfferSnapshotResult = v1.OFFER_SNAPSHOT_RESULT_REJECT_FORMAT
	OFFER_SNAPSHOT_RESULT_REJECT_SENDER OfferSnapshotResult = v1.OFFER_SNAPSHOT_RESULT_REJECT_SENDER
)

type ProcessProposalStatus = v1.ProcessProposalStatus

const (
	PROCESS_PROPOSAL_STATUS_UNKNOWN ProcessProposalStatus = v1.PROCESS_PROPOSAL_STATUS_UNKNOWN
	PROCESS_PROPOSAL_STATUS_ACCEPT  ProcessProposalStatus = v1.PROCESS_PROPOSAL_STATUS_ACCEPT
	PROCESS_PROPOSAL_STATUS_REJECT  ProcessProposalStatus = v1.PROCESS_PROPOSAL_STATUS_REJECT
)

type VerifyVoteExtensionStatus = v1.VerifyVoteExtensionStatus

const (
	VERIFY_VOTE_EXTENSION_STATUS_UNKNOWN VerifyVoteExtensionStatus = v1.VERIFY_VOTE_EXTENSION_STATUS_UNKNOWN
	VERIFY_VOTE_EXTENSION_STATUS_ACCEPT  VerifyVoteExtensionStatus = v1.VERIFY_VOTE_EXTENSION_STATUS_ACCEPT
	VERIFY_VOTE_EXTENSION_STATUS_REJECT  VerifyVoteExtensionStatus = v1.VERIFY_VOTE_EXTENSION_STATUS_REJECT
)

const (
	CodeTypeOK uint32 = 0
)

// Some compile time assertions to ensure we don't
// have accidental runtime surprises later on.

// jsonEncodingRoundTripper ensures that asserted
// interfaces implement both MarshalJSON and UnmarshalJSON.
type jsonRoundTripper interface {
	json.Marshaler
	json.Unmarshaler
}

var (
	_ jsonRoundTripper = (*CommitResponse)(nil)
	_ jsonRoundTripper = (*QueryResponse)(nil)
	_ jsonRoundTripper = (*ExecTxResult)(nil)
	_ jsonRoundTripper = (*CheckTxResponse)(nil)
)

var _ jsonRoundTripper = (*EventAttribute)(nil)

// constructs a copy of response that omits
// non-deterministic fields. The input response is not modified.
func DeterministicExecTxResult(response *ExecTxResult) *ExecTxResult {
	return &ExecTxResult{
		Code:      response.Code,
		Data:      response.Data,
		GasWanted: response.GasWanted,
		GasUsed:   response.GasUsed,
	}
}

// MarshalTxResults encodes the TxResults as a list of byte
// slices. It strips off the non-deterministic pieces of the TxResults
// so that the resulting data can be used for hash comparisons and used
// in Merkle proofs.
func MarshalTxResults(r []*ExecTxResult) ([][]byte, error) {
	s := make([][]byte, len(r))
	for i, e := range r {
		d := DeterministicExecTxResult(e)
		b, err := d.Marshal()
		if err != nil {
			return nil, err
		}
		s[i] = b
	}
	return s, nil
}

// -----------------------------------------------
// construct Result data
