package tx_test

import (
	"testing"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/tx"
	"github.com/stretchr/testify/require"
)

func TestFactoryPrepate(t *testing.T) {
	t.Parallel()

	factory := tx.Factory{}
	clientCtx := client.Context{}

	output, err := factory.Prepare(clientCtx.WithOffline(true))
	require.NoError(t, err)
	require.Equal(t, output, factory)

	factory = tx.Factory{}.WithAccountRetriever(client.MockAccountRetriever{ReturnAccNum: 10, ReturnAccSeq: 1}).WithAccountNumber(5)
	output, err = factory.Prepare(clientCtx.WithFrom("foo"))
	require.NoError(t, err)
	require.NotEqual(t, output, factory)
	require.Equal(t, output.AccountNumber(), uint64(5))
	require.Equal(t, output.Sequence(), uint64(1))

	factory = tx.Factory{}.WithAccountRetriever(client.MockAccountRetriever{ReturnAccNum: 10, ReturnAccSeq: 1})
	output, err = factory.Prepare(clientCtx.WithFrom("foo"))
	require.NoError(t, err)
	require.NotEqual(t, output, factory)
	require.Equal(t, output.AccountNumber(), uint64(10))
	require.Equal(t, output.Sequence(), uint64(1))
}
