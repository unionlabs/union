package registration

import (
	"flag"
	"os"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/cmd"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/urfave/cli/v2"
)

func TestP2PPreregistration_DefaultDataDir(t *testing.T) {
	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.String(cmd.DataDirFlag.Name, "", "")
	ctx := cli.NewContext(&app, set, nil)

	_, dataDir, err := P2PPreregistration(ctx)
	require.NoError(t, err)
	assert.Equal(t, cmd.DefaultDataDir(), dataDir)
}

func TestP2PPreregistration(t *testing.T) {
	sampleNode := "- enr:-TESTNODE"
	testDataDir := "testDataDir"

	file, err := os.CreateTemp(t.TempDir(), "bootstrapFile*.yaml")
	require.NoError(t, err)
	err = os.WriteFile(file.Name(), []byte(sampleNode), 0644)
	require.NoError(t, err, "Error in WriteFile call")
	params.SetupTestConfigCleanup(t)
	config := params.BeaconNetworkConfig()
	config.BootstrapNodes = []string{file.Name()}
	params.OverrideBeaconNetworkConfig(config)

	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.String(cmd.DataDirFlag.Name, testDataDir, "")
	ctx := cli.NewContext(&app, set, nil)

	bootstrapNodeAddrs, dataDir, err := P2PPreregistration(ctx)
	require.NoError(t, err)
	require.Equal(t, 1, len(bootstrapNodeAddrs))
	assert.Equal(t, sampleNode[2:], bootstrapNodeAddrs[0])
	assert.Equal(t, testDataDir, dataDir)
}

func TestBootStrapNodeFile(t *testing.T) {
	file, err := os.CreateTemp(t.TempDir(), "bootstrapFile")
	require.NoError(t, err)

	sampleNode0 := "- enr:-Ku4QMKVC_MowDsmEa20d5uGjrChI0h8_KsKXDmgVQbIbngZV0i" +
		"dV6_RL7fEtZGo-kTNZ5o7_EJI_vCPJ6scrhwX0Z4Bh2F0dG5ldHOIAAAAAAAAAACEZXRoMpD" +
		"1pf1CAAAAAP__________gmlkgnY0gmlwhBLf22SJc2VjcDI1NmsxoQJxCnE6v_x2ekgY_uo" +
		"E1rtwzvGy40mq9eD66XfHPBWgIIN1ZHCCD6A"
	sampleNode1 := "- enr:-TESTNODE2"
	sampleNode2 := "- enr:-TESTNODE3"
	err = os.WriteFile(file.Name(), []byte(sampleNode0+"\n"+sampleNode1+"\n"+sampleNode2), 0644)
	require.NoError(t, err, "Error in WriteFile call")
	nodeList, err := readbootNodes(file.Name())
	require.NoError(t, err, "Error in readbootNodes call")
	assert.Equal(t, sampleNode0[2:], nodeList[0], "Unexpected nodes")
	assert.Equal(t, sampleNode1[2:], nodeList[1], "Unexpected nodes")
	assert.Equal(t, sampleNode2[2:], nodeList[2], "Unexpected nodes")
}
