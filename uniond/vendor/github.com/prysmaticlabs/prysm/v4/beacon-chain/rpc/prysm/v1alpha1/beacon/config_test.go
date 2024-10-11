package beacon

import (
	"context"
	"fmt"
	"reflect"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"google.golang.org/protobuf/types/known/emptypb"
)

func TestServer_GetBeaconConfig(t *testing.T) {
	ctx := context.Background()
	bs := &Server{}
	res, err := bs.GetBeaconConfig(ctx, &emptypb.Empty{})
	require.NoError(t, err)
	conf := params.BeaconConfig()
	numFields := reflect.TypeOf(conf).Elem().NumField()

	// Check if the result has the same number of items as our config struct.
	assert.Equal(t, numFields, len(res.Config), "Unexpected number of items in config")
	want := fmt.Sprintf("%d", conf.Eth1FollowDistance)

	// Check that an element is properly populated from the config.
	assert.Equal(t, want, res.Config["Eth1FollowDistance"], "Unexpected follow distance")
}
