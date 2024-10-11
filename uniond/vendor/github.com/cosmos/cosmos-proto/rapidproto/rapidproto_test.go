package rapidproto_test

import (
	"fmt"
	"testing"

	"google.golang.org/protobuf/encoding/protojson"
	"google.golang.org/protobuf/proto"
	"gotest.tools/v3/assert"
	"gotest.tools/v3/golden"
	"pgregory.net/rapid"

	"github.com/cosmos/cosmos-proto/rapidproto"
	"github.com/cosmos/cosmos-proto/testpb"
)

// TestRegression checks that the generator still produces the same output
// for the same random seeds, assuming that this data has been hand expected
// to generally look good.
func TestRegression(t *testing.T) {
	gen := rapidproto.MessageGenerator(&testpb.A{}, rapidproto.GeneratorOptions{})
	for i := 1000; i < 1005; i++ {
		testRegressionSeed(t, i, gen)
	}
}

func testRegressionSeed[X proto.Message](t *testing.T, seed int, generator *rapid.Generator[X]) {
	x := generator.Example(seed)
	bz, err := protojson.Marshal(x)
	assert.NilError(t, err)
	golden.Assert(t, string(bz), fmt.Sprintf("seed%d.json", seed))
}
