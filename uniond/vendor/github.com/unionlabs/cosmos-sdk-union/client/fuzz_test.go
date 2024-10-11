package client_test

import (
	"context"
	"testing"

	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"

	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	"github.com/cosmos/cosmos-sdk/testutil/x/counter/types"
)

type fuzzSuite struct {
	IntegrationTestSuite
}

func (fz *fuzzSuite) FuzzQuery(f *testing.F) {
	if testing.Short() {
		f.Skip("In -short mode")
	}

	// gRPC query to test service should work
	testRes, err := fz.testClient.Echo(context.Background(), &testdata.EchoRequest{Message: "hello"})
	fz.Require().NoError(err)
	fz.Require().Equal("hello", testRes.Message)

	// 1. Generate some seeds.
	bz, err := fz.cdc.Marshal(&types.QueryGetCountRequest{})
	fz.Require().NoError(err)
	f.Add(bz)

	// 2. Now fuzz it and ensure that we don't get any panics.
	ctx := context.Background()
	f.Fuzz(func(t *testing.T, in []byte) {
		qbReq := new(types.QueryGetCountRequest)
		if err := fz.cdc.Unmarshal(in, qbReq); err != nil {
			return
		}

		// gRPC query to bank service should work
		var header metadata.MD
		_, _ = fz.counterClient.GetCount(
			ctx,
			qbReq,
			grpc.Header(&header),
		)
	})
}

func FuzzQuery(f *testing.F) {
	fzs := new(fuzzSuite)
	fzs.SetT(new(testing.T))
	fzs.SetupSuite()
	fzs.FuzzQuery(f)
}
