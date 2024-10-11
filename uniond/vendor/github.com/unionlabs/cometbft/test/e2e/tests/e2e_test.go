package e2e_test

import (
	"context"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"testing"

	"github.com/stretchr/testify/require"

	abci "github.com/cometbft/cometbft/abci/types"
	rpchttp "github.com/cometbft/cometbft/rpc/client/http"
	rpctypes "github.com/cometbft/cometbft/rpc/core/types"
	"github.com/cometbft/cometbft/test/e2e/app"
	e2e "github.com/cometbft/cometbft/test/e2e/pkg"
	"github.com/cometbft/cometbft/test/e2e/pkg/infra/docker"
	"github.com/cometbft/cometbft/types"
)

func init() {
	// This can be used to manually specify a testnet manifest and/or node to
	// run tests against. The testnet must have been started by the runner first.
	// os.Setenv("E2E_MANIFEST", "networks/ci.toml")
	// os.Setenv("E2E_TESTNET_DIR", "networks/ci")
	// os.Setenv("E2E_NODE", "validator01")
}

var (
	ctx             = context.Background()
	testnetCache    = map[string]e2e.Testnet{}
	testnetCacheMtx = sync.Mutex{}
	blocksCache     = map[string][]*types.Block{}
	blocksCacheMtx  = sync.Mutex{}
)

// testNode runs tests for testnet nodes. The callback function is given a
// single node to test, running as a subtest in parallel with other subtests.
//
// The testnet manifest must be given as the envvar E2E_MANIFEST. If not set,
// these tests are skipped so that they're not picked up during normal unit
// test runs. If E2E_NODE is also set, only the specified node is tested,
// otherwise all nodes are tested.
func testNode(t *testing.T, testFunc func(*testing.T, e2e.Node)) {
	t.Helper()

	testnet := loadTestnet(t)
	nodes := testnet.Nodes

	if name := os.Getenv("E2E_NODE"); name != "" {
		node := testnet.LookupNode(name)
		require.NotNil(t, node, "node %q not found in testnet %q", name, testnet.Name)
		nodes = []*e2e.Node{node}
	}

	for _, node := range nodes {
		if node.Stateless() {
			continue
		}

		node := *node
		t.Run(node.Name, func(t *testing.T) {
			testFunc(t, node)
		})
	}
}

// Similar to testNode, except only runs the given test on full nodes or
// validators. Also only runs the test on the given maximum number of nodes.
//
// If maxNodes is set to 0 or below, all full nodes and validators will be
// tested.
func testFullNodesOrValidators(t *testing.T, maxNodes int, testFunc func(*testing.T, e2e.Node)) { //nolint:unparam // maxNodes always receives 0 but that could change so we should keep the parameter.
	t.Helper()

	testnet := loadTestnet(t)
	nodes := testnet.Nodes

	if name := os.Getenv("E2E_NODE"); name != "" {
		node := testnet.LookupNode(name)
		require.NotNil(t, node, "node %q not found in testnet %q", name, testnet.Name)
		nodes = []*e2e.Node{node}
	}

	nodeCount := 0
	for _, node := range nodes {
		if node.Stateless() {
			continue
		}

		if node.Mode == e2e.ModeFull || node.Mode == e2e.ModeValidator {
			node := *node
			t.Run(node.Name, func(t *testing.T) {
				testFunc(t, node)
			})
			nodeCount++
			if maxNodes > 0 && nodeCount >= maxNodes {
				break
			}
		}
	}
}

// loadTestnet loads the testnet based on the E2E_MANIFEST envvar.
func loadTestnet(t *testing.T) e2e.Testnet {
	t.Helper()

	manifestFile := os.Getenv("E2E_MANIFEST")
	if manifestFile == "" {
		t.Skip("E2E_MANIFEST not set, not an end-to-end test run")
	}
	if !filepath.IsAbs(manifestFile) {
		manifestFile = filepath.Join("..", manifestFile)
	}
	ifdType := os.Getenv("INFRASTRUCTURE_TYPE")
	ifdFile := os.Getenv("INFRASTRUCTURE_FILE")
	if ifdType != "docker" && ifdFile == "" {
		t.Fatalf("INFRASTRUCTURE_FILE not set and INFRASTRUCTURE_TYPE is not 'docker'")
	}
	testnetCacheMtx.Lock()
	defer testnetCacheMtx.Unlock()
	if testnet, ok := testnetCache[manifestFile]; ok {
		return testnet
	}
	m, err := e2e.LoadManifest(manifestFile)
	require.NoError(t, err)

	var ifd e2e.InfrastructureData
	switch ifdType {
	case "docker":
		ifd, err = e2e.NewDockerInfrastructureData(m)
		require.NoError(t, err)
	case "digital-ocean":
		ifd, err = e2e.InfrastructureDataFromFile(ifdFile)
		require.NoError(t, err)
	default:
	}
	require.NoError(t, err)

	testnetDir := os.Getenv("E2E_TESTNET_DIR")
	if !filepath.IsAbs(testnetDir) {
		testnetDir = filepath.Join("..", testnetDir)
	}

	testnet, err := e2e.LoadTestnet(manifestFile, ifd, testnetDir)
	require.NoError(t, err)
	testnetCache[manifestFile] = *testnet
	return *testnet
}

// fetchBlockChain fetches a complete, up-to-date block history from
// the freshest testnet archive node.
func fetchBlockChain(t *testing.T) []*types.Block {
	t.Helper()

	testnet := loadTestnet(t)

	// Find the freshest archive node
	var (
		client *rpchttp.HTTP
		status *rpctypes.ResultStatus
	)
	for _, node := range testnet.ArchiveNodes() {
		c, err := node.Client()
		require.NoError(t, err)
		s, err := c.Status(ctx)
		require.NoError(t, err)
		if status == nil || s.SyncInfo.LatestBlockHeight > status.SyncInfo.LatestBlockHeight {
			client = c
			status = s
		}
	}
	require.NotNil(t, client, "couldn't find an archive node")

	// Fetch blocks. Look for existing block history in the block cache, and
	// extend it with any new blocks that have been produced.
	blocksCacheMtx.Lock()
	defer blocksCacheMtx.Unlock()

	from := status.SyncInfo.EarliestBlockHeight
	to := status.SyncInfo.LatestBlockHeight
	blocks, ok := blocksCache[testnet.Name]
	if !ok {
		blocks = make([]*types.Block, 0, to-from+1)
	}
	if len(blocks) > 0 {
		from = blocks[len(blocks)-1].Height + 1
	}

	for h := from; h <= to; h++ {
		resp, err := client.Block(ctx, &(h))
		require.NoError(t, err)
		require.NotNil(t, resp.Block)
		require.Equal(t, h, resp.Block.Height, "unexpected block height %v", resp.Block.Height)
		blocks = append(blocks, resp.Block)
	}
	require.NotEmpty(t, blocks, "blockchain does not contain any blocks")
	blocksCache[testnet.Name] = blocks

	return blocks
}

// fetchABCIRequests go through the logs of a specific node and collect all ABCI requests (each slice represents requests from beginning until the first crash,
// and then between two crashes) for a specific node.
func fetchABCIRequests(t *testing.T, nodeName string) ([][]*abci.Request, error) {
	t.Helper()
	testnet := loadTestnet(t)
	logs, err := fetchNodeLogs(testnet)
	if err != nil {
		return nil, err
	}
	reqs := make([][]*abci.Request, 0)
	// Parse output line by line.
	lines := strings.Split(string(logs), "\n")
	for _, line := range lines {
		if !strings.Contains(line, nodeName) {
			continue
		}
		if strings.Contains(line, "Application started") {
			reqs = append(reqs, make([]*abci.Request, 0))
			continue
		}
		r, err := app.GetABCIRequestFromString(line)
		require.NoError(t, err)
		// Ship the lines that does not contain abci request.
		if r == nil {
			continue
		}
		reqs[len(reqs)-1] = append(reqs[len(reqs)-1], r)
	}
	return reqs, nil
}

func fetchNodeLogs(testnet e2e.Testnet) ([]byte, error) {
	return docker.ExecComposeOutput(context.Background(), testnet.Dir, "logs")
}
