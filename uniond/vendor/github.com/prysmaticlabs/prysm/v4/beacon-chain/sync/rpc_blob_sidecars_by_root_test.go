package sync

import (
	"fmt"
	"sort"
	"testing"

	"github.com/libp2p/go-libp2p/core/network"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	p2pTypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/types"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	types "github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

func (c *blobsTestCase) defaultOldestSlotByRoot(t *testing.T) types.Slot {
	oldest, err := slots.EpochStart(blobMinReqEpoch(c.chain.FinalizedCheckPoint.Epoch, slots.ToEpoch(c.clock.CurrentSlot())))
	require.NoError(t, err)
	return oldest
}

func blobRootRequestFromSidecars(scs []blocks.ROBlob) interface{} {
	req := make(p2pTypes.BlobSidecarsByRootReq, 0)
	for i := range scs {
		sc := scs[i]
		req = append(req, &ethpb.BlobIdentifier{BlockRoot: sc.BlockRootSlice(), Index: sc.Index})
	}
	return &req
}

func (c *blobsTestCase) filterExpectedByRoot(t *testing.T, scs []blocks.ROBlob, r interface{}) []*expectedBlobChunk {
	rp, ok := r.(*p2pTypes.BlobSidecarsByRootReq)
	if !ok {
		panic("unexpected request type in filterExpectedByRoot")
	}
	req := *rp
	if uint64(len(req)) > params.BeaconConfig().MaxRequestBlobSidecars {
		return []*expectedBlobChunk{{
			code:    responseCodeInvalidRequest,
			message: p2pTypes.ErrBlobLTMinRequest.Error(),
		}}
	}
	sort.Sort(req)
	var expect []*expectedBlobChunk
	blockOffset := 0
	if len(scs) == 0 {
		return expect
	}
	lastRoot := scs[0].BlockRoot()
	rootToOffset := make(map[[32]byte]int)
	rootToOffset[lastRoot] = 0
	scMap := make(map[[32]byte]map[uint64]blocks.ROBlob)
	for i := range scs {
		sc := scs[i]
		root := sc.BlockRoot()
		if root != lastRoot {
			blockOffset += 1
			rootToOffset[root] = blockOffset
		}
		lastRoot = root
		_, ok := scMap[root]
		if !ok {
			scMap[root] = make(map[uint64]blocks.ROBlob)
		}
		scMap[root][sc.Index] = sc
	}
	for i := range req {
		scid := req[i]
		rootMap, ok := scMap[bytesutil.ToBytes32(scid.BlockRoot)]
		if !ok {
			panic(fmt.Sprintf("test setup failure, no fixture with root %#x", scid.BlockRoot))
		}
		sc, idxOk := rootMap[scid.Index]
		if !idxOk {
			panic(fmt.Sprintf("test setup failure, no fixture at index %d with root %#x", scid.Index, scid.BlockRoot))
		}
		// Skip sidecars that are supposed to be missing.
		root := sc.BlockRoot()
		if c.missing[rootToOffset[root]] {
			continue
		}
		// If a sidecar is expired, we'll expect an error for the *first* index, and after that
		// we'll expect no further chunks in the stream, so filter out any further expected responses.
		// We don't need to check what index this is because we work through them in order and the first one
		// will set streamTerminated = true and skip everything else in the test case.
		if c.expired[rootToOffset[root]] {
			return append(expect, &expectedBlobChunk{
				sidecar: &sc,
				code:    responseCodeResourceUnavailable,
				message: p2pTypes.ErrBlobLTMinRequest.Error(),
			})
		}

		expect = append(expect, &expectedBlobChunk{
			sidecar: &sc,
			code:    responseCodeSuccess,
			message: "",
		})
	}
	return expect
}

func (c *blobsTestCase) runTestBlobSidecarsByRoot(t *testing.T) {
	if c.serverHandle == nil {
		c.serverHandle = func(s *Service) rpcHandler { return s.blobSidecarByRootRPCHandler }
	}
	if c.defineExpected == nil {
		c.defineExpected = c.filterExpectedByRoot
	}
	if c.requestFromSidecars == nil {
		c.requestFromSidecars = blobRootRequestFromSidecars
	}
	if c.topic == "" {
		c.topic = p2p.RPCBlobSidecarsByRootTopicV1
	}
	if c.oldestSlot == nil {
		c.oldestSlot = c.defaultOldestSlotByRoot
	}
	if c.streamReader == nil {
		c.streamReader = defaultExpectedRequirer
	}
	c.run(t)
}

func TestReadChunkEncodedBlobs(t *testing.T) {
	cases := []*blobsTestCase{
		{
			name:         "test successful read via requester",
			nblocks:      1,
			streamReader: readChunkEncodedBlobsAsStreamReader,
		},
		{
			name:         "test peer sending excess blobs",
			nblocks:      1,
			streamReader: readChunkEncodedBlobsLowMax,
		},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			c.runTestBlobSidecarsByRoot(t)
		})
	}
}

// Specifies a max expected chunk parameter of 1, so that a response with one or more blobs will give ErrInvalidFetchedData.
func readChunkEncodedBlobsLowMax(t *testing.T, s *Service, expect []*expectedBlobChunk) func(network.Stream) {
	encoding := s.cfg.p2p.Encoding()
	ctxMap, err := ContextByteVersionsForValRoot(s.cfg.clock.GenesisValidatorsRoot())
	require.NoError(t, err)
	vf := func(sidecar blocks.ROBlob) error {
		return nil
	}
	return func(stream network.Stream) {
		_, err := readChunkEncodedBlobs(stream, encoding, ctxMap, vf, 1)
		require.ErrorIs(t, err, ErrInvalidFetchedData)
	}
}

func readChunkEncodedBlobsAsStreamReader(t *testing.T, s *Service, expect []*expectedBlobChunk) func(network.Stream) {
	encoding := s.cfg.p2p.Encoding()
	ctxMap, err := ContextByteVersionsForValRoot(s.cfg.clock.GenesisValidatorsRoot())
	require.NoError(t, err)
	vf := func(sidecar blocks.ROBlob) error {
		return nil
	}
	return func(stream network.Stream) {
		scs, err := readChunkEncodedBlobs(stream, encoding, ctxMap, vf, params.BeaconConfig().MaxRequestBlobSidecars)
		require.NoError(t, err)
		require.Equal(t, len(expect), len(scs))
		for i, sc := range scs {
			esc := expect[i].sidecar
			require.Equal(t, esc.Slot(), sc.Slot())
			require.Equal(t, esc.Index, sc.Index)
			require.Equal(t, esc.BlockRoot(), sc.BlockRoot())
		}
	}
}

func TestBlobsByRootValidation(t *testing.T) {
	cfg := params.BeaconConfig()
	repositionFutureEpochs(cfg)
	undo, err := params.SetActiveWithUndo(cfg)
	require.NoError(t, err)
	defer func() {
		require.NoError(t, undo())
	}()
	capellaSlot, err := slots.EpochStart(params.BeaconConfig().CapellaForkEpoch)
	require.NoError(t, err)
	dmc, clock := defaultMockChain(t)
	dmc.Slot = &capellaSlot
	dmc.FinalizedCheckPoint = &ethpb.Checkpoint{Epoch: params.BeaconConfig().CapellaForkEpoch}
	cases := []*blobsTestCase{
		{
			name:    "block before minimum_request_epoch",
			nblocks: 1,
			expired: map[int]bool{0: true},
			chain:   dmc,
			clock:   clock,
			err:     p2pTypes.ErrBlobLTMinRequest,
		},
		{
			name:    "blocks before and after minimum_request_epoch",
			nblocks: 2,
			expired: map[int]bool{0: true},
			chain:   dmc,
			clock:   clock,
			err:     p2pTypes.ErrBlobLTMinRequest,
		},
		{
			name:    "one after minimum_request_epoch then one before",
			nblocks: 2,
			expired: map[int]bool{1: true},
			chain:   dmc,
			clock:   clock,
			err:     p2pTypes.ErrBlobLTMinRequest,
		},
		{
			name:    "block with all indices missing between 2 full blocks",
			nblocks: 3,
			missing: map[int]bool{1: true},
			total:   func(i int) *int { return &i }(2 * fieldparams.MaxBlobsPerBlock),
		},
		{
			name:    "exceeds req max",
			nblocks: int(params.BeaconConfig().MaxRequestBlobSidecars) + 1,
			err:     p2pTypes.ErrMaxBlobReqExceeded,
		},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			c.runTestBlobSidecarsByRoot(t)
		})
	}
}

func TestBlobsByRootOK(t *testing.T) {
	cases := []*blobsTestCase{
		{
			name:    "0 blob",
			nblocks: 0,
		},
		{
			name:    "1 blob",
			nblocks: 1,
		},
		{
			name:    "2 blob",
			nblocks: 2,
		},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			c.runTestBlobSidecarsByRoot(t)
		})
	}
}

func TestBlobsByRootMinReqEpoch(t *testing.T) {
	winMin := params.BeaconConfig().MinEpochsForBlobsSidecarsRequest
	cases := []struct {
		name      string
		finalized types.Epoch
		current   types.Epoch
		deneb     types.Epoch
		expected  types.Epoch
	}{
		{
			name:      "testnet genesis",
			deneb:     100,
			current:   0,
			finalized: 0,
			expected:  100,
		},
		{
			name:      "underflow averted",
			deneb:     100,
			current:   winMin - 1,
			finalized: 0,
			expected:  100,
		},
		{
			name:      "underflow averted - finalized is higher",
			deneb:     100,
			current:   winMin - 1,
			finalized: winMin - 2,
			expected:  winMin - 2,
		},
		{
			name:      "underflow averted - genesis at deneb",
			deneb:     0,
			current:   winMin - 1,
			finalized: 0,
			expected:  0,
		},
		{
			name:      "max is finalized",
			deneb:     100,
			current:   99 + winMin,
			finalized: 101,
			expected:  101,
		},
		{
			name:      "reqWindow > finalized, reqWindow < deneb",
			deneb:     100,
			current:   99 + winMin,
			finalized: 98,
			expected:  100,
		},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			cfg := params.BeaconConfig()
			repositionFutureEpochs(cfg)
			cfg.DenebForkEpoch = c.deneb
			undo, err := params.SetActiveWithUndo(cfg)
			require.NoError(t, err)
			defer func() {
				require.NoError(t, undo())
			}()
			ep := blobMinReqEpoch(c.finalized, c.current)
			require.Equal(t, c.expected, ep)
		})
	}
}
