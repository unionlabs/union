package utils_test

import (
	"context"
	"testing"

	"github.com/cometbft/cometbft/rpc/client/mock"
	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	tmtypes "github.com/cometbft/cometbft/types"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	"github.com/cosmos/cosmos-sdk/x/gov"
	"github.com/cosmos/cosmos-sdk/x/gov/client/utils"
	v1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1"
)

type TxSearchMock struct {
	txConfig client.TxConfig
	mock.Client
	txs []tmtypes.Tx
}

func (mock TxSearchMock) TxSearch(ctx context.Context, query string, prove bool, page, perPage *int, orderBy string) (*coretypes.ResultTxSearch, error) {
	if page == nil {
		*page = 0
	}

	if perPage == nil {
		*perPage = 0
	}

	start, end := client.Paginate(len(mock.txs), *page, *perPage, 100)
	if start < 0 || end < 0 {
		// nil result with nil error crashes utils.QueryTxsByEvents
		return &coretypes.ResultTxSearch{}, nil
	}

	txs := mock.txs[start:end]
	rst := &coretypes.ResultTxSearch{Txs: make([]*coretypes.ResultTx, len(txs)), TotalCount: len(txs)}
	for i := range txs {
		rst.Txs[i] = &coretypes.ResultTx{Tx: txs[i]}
	}
	return rst, nil
}

func (mock TxSearchMock) Block(ctx context.Context, height *int64) (*coretypes.ResultBlock, error) {
	// any non nil Block needs to be returned. used to get time value
	return &coretypes.ResultBlock{Block: &tmtypes.Block{}}, nil
}

func TestGetPaginatedVotes(t *testing.T) {
	encCfg := moduletestutil.MakeTestEncodingConfig(gov.AppModuleBasic{})

	type testCase struct {
		description string
		page, limit int
		msgs        [][]sdk.Msg
		votes       []v1.Vote
	}
	acc1 := make(sdk.AccAddress, 20)
	acc1[0] = 1
	acc2 := make(sdk.AccAddress, 20)
	acc2[0] = 2
	acc1Msgs := []sdk.Msg{
		v1.NewMsgVote(acc1, 0, v1.OptionYes, ""),
		v1.NewMsgVote(acc1, 0, v1.OptionYes, ""),
		v1.NewMsgDeposit(acc1, 0, sdk.NewCoins(sdk.NewCoin("stake", sdk.NewInt(10)))), // should be ignored
	}
	acc2Msgs := []sdk.Msg{
		v1.NewMsgVote(acc2, 0, v1.OptionYes, ""),
		v1.NewMsgVoteWeighted(acc2, 0, v1.NewNonSplitVoteOption(v1.OptionYes), ""),
	}
	for _, tc := range []testCase{
		{
			description: "1MsgPerTxAll",
			page:        1,
			limit:       2,
			msgs: [][]sdk.Msg{
				acc1Msgs[:1],
				acc2Msgs[:1],
			},
			votes: []v1.Vote{
				v1.NewVote(0, acc1, v1.NewNonSplitVoteOption(v1.OptionYes), ""),
				v1.NewVote(0, acc2, v1.NewNonSplitVoteOption(v1.OptionYes), ""),
			},
		},
		{
			description: "2MsgPerTx1Chunk",
			page:        1,
			limit:       2,
			msgs: [][]sdk.Msg{
				acc1Msgs,
				acc2Msgs,
			},
			votes: []v1.Vote{
				v1.NewVote(0, acc1, v1.NewNonSplitVoteOption(v1.OptionYes), ""),
				v1.NewVote(0, acc1, v1.NewNonSplitVoteOption(v1.OptionYes), ""),
			},
		},
		{
			description: "2MsgPerTx2Chunk",
			page:        2,
			limit:       2,
			msgs: [][]sdk.Msg{
				acc1Msgs,
				acc2Msgs,
			},
			votes: []v1.Vote{
				v1.NewVote(0, acc2, v1.NewNonSplitVoteOption(v1.OptionYes), ""),
				v1.NewVote(0, acc2, v1.NewNonSplitVoteOption(v1.OptionYes), ""),
			},
		},
		{
			description: "IncompleteSearchTx",
			page:        1,
			limit:       2,
			msgs: [][]sdk.Msg{
				acc1Msgs[:1],
			},
			votes: []v1.Vote{v1.NewVote(0, acc1, v1.NewNonSplitVoteOption(v1.OptionYes), "")},
		},
		{
			description: "InvalidPage",
			page:        -1,
			msgs: [][]sdk.Msg{
				acc1Msgs[:1],
			},
		},
		{
			description: "OutOfBounds",
			page:        2,
			limit:       10,
			msgs: [][]sdk.Msg{
				acc1Msgs[:1],
			},
		},
	} {
		tc := tc

		t.Run(tc.description, func(t *testing.T) {
			marshalled := make([]tmtypes.Tx, len(tc.msgs))
			cli := TxSearchMock{txs: marshalled, txConfig: encCfg.TxConfig}
			clientCtx := client.Context{}.
				WithLegacyAmino(encCfg.Amino).
				WithClient(cli).
				WithTxConfig(encCfg.TxConfig)

			for i := range tc.msgs {
				txBuilder := clientCtx.TxConfig.NewTxBuilder()
				err := txBuilder.SetMsgs(tc.msgs[i]...)
				require.NoError(t, err)

				tx, err := clientCtx.TxConfig.TxEncoder()(txBuilder.GetTx())
				require.NoError(t, err)
				marshalled[i] = tx
			}

			params := v1.NewQueryProposalVotesParams(0, tc.page, tc.limit)
			votesData, err := utils.QueryVotesByTxQuery(clientCtx, params)
			require.NoError(t, err)
			votes := []v1.Vote{}
			require.NoError(t, clientCtx.LegacyAmino.UnmarshalJSON(votesData, &votes))
			require.Equal(t, len(tc.votes), len(votes))
			for i := range votes {
				require.Equal(t, tc.votes[i], votes[i])
			}
		})
	}
}
