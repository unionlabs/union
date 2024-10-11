package server

import (
	"context"

	abci "github.com/cometbft/cometbft/abci/types"
	abciproto "github.com/cometbft/cometbft/api/cometbft/abci/v1"

	servertypes "github.com/cosmos/cosmos-sdk/server/types"
)

type cometABCIWrapper struct {
	app servertypes.ABCI
}

func NewCometABCIWrapper(app servertypes.ABCI) abci.Application {
	return cometABCIWrapper{app: app}
}

func (w cometABCIWrapper) Info(_ context.Context, req *abciproto.InfoRequest) (*abciproto.InfoResponse, error) {
	return w.app.Info(req)
}

func (w cometABCIWrapper) Query(ctx context.Context, req *abciproto.QueryRequest) (*abciproto.QueryResponse, error) {
	return w.app.Query(ctx, req)
}

func (w cometABCIWrapper) CheckTx(_ context.Context, req *abciproto.CheckTxRequest) (*abciproto.CheckTxResponse, error) {
	return w.app.CheckTx(req)
}

func (w cometABCIWrapper) InitChain(_ context.Context, req *abciproto.InitChainRequest) (*abciproto.InitChainResponse, error) {
	return w.app.InitChain(req)
}

func (w cometABCIWrapper) PrepareProposal(_ context.Context, req *abciproto.PrepareProposalRequest) (*abciproto.PrepareProposalResponse, error) {
	return w.app.PrepareProposal(req)
}

func (w cometABCIWrapper) ProcessProposal(_ context.Context, req *abciproto.ProcessProposalRequest) (*abciproto.ProcessProposalResponse, error) {
	return w.app.ProcessProposal(req)
}

func (w cometABCIWrapper) FinalizeBlock(_ context.Context, req *abciproto.FinalizeBlockRequest) (*abciproto.FinalizeBlockResponse, error) {
	return w.app.FinalizeBlock(req)
}

func (w cometABCIWrapper) ExtendVote(ctx context.Context, req *abciproto.ExtendVoteRequest) (*abciproto.ExtendVoteResponse, error) {
	return w.app.ExtendVote(ctx, req)
}

func (w cometABCIWrapper) VerifyVoteExtension(_ context.Context, req *abciproto.VerifyVoteExtensionRequest) (*abciproto.VerifyVoteExtensionResponse, error) {
	return w.app.VerifyVoteExtension(req)
}

func (w cometABCIWrapper) Commit(_ context.Context, _ *abciproto.CommitRequest) (*abciproto.CommitResponse, error) {
	return w.app.Commit()
}

func (w cometABCIWrapper) ListSnapshots(_ context.Context, req *abciproto.ListSnapshotsRequest) (*abciproto.ListSnapshotsResponse, error) {
	return w.app.ListSnapshots(req)
}

func (w cometABCIWrapper) OfferSnapshot(_ context.Context, req *abciproto.OfferSnapshotRequest) (*abciproto.OfferSnapshotResponse, error) {
	return w.app.OfferSnapshot(req)
}

func (w cometABCIWrapper) LoadSnapshotChunk(_ context.Context, req *abciproto.LoadSnapshotChunkRequest) (*abciproto.LoadSnapshotChunkResponse, error) {
	return w.app.LoadSnapshotChunk(req)
}

func (w cometABCIWrapper) ApplySnapshotChunk(_ context.Context, req *abciproto.ApplySnapshotChunkRequest) (*abciproto.ApplySnapshotChunkResponse, error) {
	return w.app.ApplySnapshotChunk(req)
}
