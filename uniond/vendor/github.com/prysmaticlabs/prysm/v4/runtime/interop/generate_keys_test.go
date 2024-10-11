package interop_test

import (
	"fmt"
	"os"
	"testing"

	"github.com/bazelbuild/rules_go/go/tools/bazel"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/go-yaml/yaml"
	"github.com/prysmaticlabs/prysm/v4/runtime/interop"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

type TestCase struct {
	Privkey string `yaml:"privkey"`
}

type KeyTest struct {
	TestCases []*TestCase `yaml:"test_cases"`
}

func TestKeyGenerator(t *testing.T) {
	path, err := bazel.Runfile("keygen_test_vector.yaml")
	require.NoError(t, err)
	file, err := os.ReadFile(path)
	require.NoError(t, err)
	testCases := &KeyTest{}
	require.NoError(t, yaml.Unmarshal(file, testCases))
	priv, pubkeys, err := interop.DeterministicallyGenerateKeys(0, 1000)
	require.NoError(t, err)
	// cross-check with the first 1000 keys generated from the python spec
	for i, key := range priv {
		hexKey := testCases.TestCases[i].Privkey
		nKey, err := hexutil.Decode("0x" + hexKey)
		if err != nil {
			t.Error(err)
			continue
		}
		assert.DeepEqual(t, key.Marshal(), nKey)
		fmt.Printf("pubkey: %s privkey: %s \n", hexutil.Encode(pubkeys[i].Marshal()), hexutil.Encode(key.Marshal()))
	}
}
