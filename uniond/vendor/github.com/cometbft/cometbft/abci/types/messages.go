package types

import (
	"io"
	"math"

	"github.com/cosmos/gogoproto/proto"

	pb "github.com/cometbft/cometbft/api/cometbft/abci/v1"
	"github.com/cometbft/cometbft/libs/protoio"
)

const (
	maxMsgSize = math.MaxInt32 // 2GB
)

// WriteMessage writes a varint length-delimited protobuf message.
func WriteMessage(msg proto.Message, w io.Writer) error {
	protoWriter := protoio.NewDelimitedWriter(w)
	_, err := protoWriter.WriteMsg(msg)
	return err
}

// ReadMessage reads a varint length-delimited protobuf message.
func ReadMessage(r io.Reader, msg proto.Message) error {
	_, err := protoio.NewDelimitedReader(r, maxMsgSize).ReadMsg(msg)
	return err
}

// ----------------------------------------

func ToEchoRequest(message string) *Request {
	return &Request{
		Value: &pb.Request_Echo{Echo: &EchoRequest{Message: message}},
	}
}

func ToFlushRequest() *Request {
	return &Request{
		Value: &pb.Request_Flush{Flush: &FlushRequest{}},
	}
}

func ToInfoRequest(req *InfoRequest) *Request {
	return &Request{
		Value: &pb.Request_Info{Info: req},
	}
}

func ToCheckTxRequest(req *CheckTxRequest) *Request {
	return &Request{
		Value: &pb.Request_CheckTx{CheckTx: req},
	}
}

func ToCommitRequest() *Request {
	return &Request{
		Value: &pb.Request_Commit{Commit: &CommitRequest{}},
	}
}

func ToQueryRequest(req *QueryRequest) *Request {
	return &Request{
		Value: &pb.Request_Query{Query: req},
	}
}

func ToInitChainRequest(req *InitChainRequest) *Request {
	return &Request{
		Value: &pb.Request_InitChain{InitChain: req},
	}
}

func ToListSnapshotsRequest(req *ListSnapshotsRequest) *Request {
	return &Request{
		Value: &pb.Request_ListSnapshots{ListSnapshots: req},
	}
}

func ToOfferSnapshotRequest(req *OfferSnapshotRequest) *Request {
	return &Request{
		Value: &pb.Request_OfferSnapshot{OfferSnapshot: req},
	}
}

func ToLoadSnapshotChunkRequest(req *LoadSnapshotChunkRequest) *Request {
	return &Request{
		Value: &pb.Request_LoadSnapshotChunk{LoadSnapshotChunk: req},
	}
}

func ToApplySnapshotChunkRequest(req *ApplySnapshotChunkRequest) *Request {
	return &Request{
		Value: &pb.Request_ApplySnapshotChunk{ApplySnapshotChunk: req},
	}
}

func ToPrepareProposalRequest(req *PrepareProposalRequest) *Request {
	return &Request{
		Value: &pb.Request_PrepareProposal{PrepareProposal: req},
	}
}

func ToProcessProposalRequest(req *ProcessProposalRequest) *Request {
	return &Request{
		Value: &pb.Request_ProcessProposal{ProcessProposal: req},
	}
}

func ToExtendVoteRequest(req *ExtendVoteRequest) *Request {
	return &Request{
		Value: &pb.Request_ExtendVote{ExtendVote: req},
	}
}

func ToVerifyVoteExtensionRequest(req *VerifyVoteExtensionRequest) *Request {
	return &Request{
		Value: &pb.Request_VerifyVoteExtension{VerifyVoteExtension: req},
	}
}

func ToFinalizeBlockRequest(req *FinalizeBlockRequest) *Request {
	return &Request{
		Value: &pb.Request_FinalizeBlock{FinalizeBlock: req},
	}
}

// ----------------------------------------

func ToExceptionResponse(errStr string) *Response {
	return &Response{
		Value: &pb.Response_Exception{Exception: &ExceptionResponse{Error: errStr}},
	}
}

func ToEchoResponse(message string) *Response {
	return &Response{
		Value: &pb.Response_Echo{Echo: &EchoResponse{Message: message}},
	}
}

func ToFlushResponse() *Response {
	return &Response{
		Value: &pb.Response_Flush{Flush: &FlushResponse{}},
	}
}

func ToInfoResponse(res *InfoResponse) *Response {
	return &Response{
		Value: &pb.Response_Info{Info: res},
	}
}

func ToCheckTxResponse(res *CheckTxResponse) *Response {
	return &Response{
		Value: &pb.Response_CheckTx{CheckTx: res},
	}
}

func ToCommitResponse(res *CommitResponse) *Response {
	return &Response{
		Value: &pb.Response_Commit{Commit: res},
	}
}

func ToQueryResponse(res *QueryResponse) *Response {
	return &Response{
		Value: &pb.Response_Query{Query: res},
	}
}

func ToInitChainResponse(res *InitChainResponse) *Response {
	return &Response{
		Value: &pb.Response_InitChain{InitChain: res},
	}
}

func ToListSnapshotsResponse(res *ListSnapshotsResponse) *Response {
	return &Response{
		Value: &pb.Response_ListSnapshots{ListSnapshots: res},
	}
}

func ToOfferSnapshotResponse(res *OfferSnapshotResponse) *Response {
	return &Response{
		Value: &pb.Response_OfferSnapshot{OfferSnapshot: res},
	}
}

func ToLoadSnapshotChunkResponse(res *LoadSnapshotChunkResponse) *Response {
	return &Response{
		Value: &pb.Response_LoadSnapshotChunk{LoadSnapshotChunk: res},
	}
}

func ToApplySnapshotChunkResponse(res *ApplySnapshotChunkResponse) *Response {
	return &Response{
		Value: &pb.Response_ApplySnapshotChunk{ApplySnapshotChunk: res},
	}
}

func ToPrepareProposalResponse(res *PrepareProposalResponse) *Response {
	return &Response{
		Value: &pb.Response_PrepareProposal{PrepareProposal: res},
	}
}

func ToProcessProposalResponse(res *ProcessProposalResponse) *Response {
	return &Response{
		Value: &pb.Response_ProcessProposal{ProcessProposal: res},
	}
}

func ToExtendVoteResponse(res *ExtendVoteResponse) *Response {
	return &Response{
		Value: &pb.Response_ExtendVote{ExtendVote: res},
	}
}

func ToVerifyVoteExtensionResponse(res *VerifyVoteExtensionResponse) *Response {
	return &Response{
		Value: &pb.Response_VerifyVoteExtension{VerifyVoteExtension: res},
	}
}

func ToFinalizeBlockResponse(res *FinalizeBlockResponse) *Response {
	return &Response{
		Value: &pb.Response_FinalizeBlock{FinalizeBlock: res},
	}
}
