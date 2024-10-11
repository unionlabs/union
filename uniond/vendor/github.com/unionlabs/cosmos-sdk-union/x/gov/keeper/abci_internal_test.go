package keeper

import (
	"context"
	"testing"

	gogoproto "github.com/cosmos/gogoproto/proto"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/core/router"
)

type mockRouterService struct {
	router.Service

	panic bool
}

func (m *mockRouterService) Invoke(ctx context.Context, req gogoproto.Message) (res gogoproto.Message, err error) {
	if m.panic {
		panic("test-fail")
	}

	return nil, nil
}

func TestSafeExecuteHandler(t *testing.T) {
	t.Parallel()

	require := require.New(t)
	ctx := context.Background()

	r, err := safeExecuteHandler(ctx, nil, &mockRouterService{panic: true})
	require.ErrorContains(err, "test-fail")
	require.Nil(r)

	_, err = safeExecuteHandler(ctx, nil, &mockRouterService{panic: false})
	require.Nil(err)
}
