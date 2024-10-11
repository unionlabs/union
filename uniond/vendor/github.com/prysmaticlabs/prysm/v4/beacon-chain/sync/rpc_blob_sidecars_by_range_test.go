package sync

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	types "github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

func (c *blobsTestCase) defaultOldestSlotByRange(t *testing.T) types.Slot {
	currentEpoch := slots.ToEpoch(c.chain.CurrentSlot())
	oldestEpoch := currentEpoch - params.BeaconConfig().MinEpochsForBlobsSidecarsRequest
	if oldestEpoch < params.BeaconConfig().DenebForkEpoch {
		oldestEpoch = params.BeaconConfig().DenebForkEpoch
	}
	oldestSlot, err := slots.EpochStart(oldestEpoch)
	require.NoError(t, err)
	return oldestSlot
}

func blobRangeRequestFromSidecars(scs []blocks.ROBlob) interface{} {
	maxBlobs := fieldparams.MaxBlobsPerBlock
	count := uint64(len(scs) / maxBlobs)
	return &ethpb.BlobSidecarsByRangeRequest{
		StartSlot: scs[0].Slot(),
		Count:     count,
	}
}

func (c *blobsTestCase) filterExpectedByRange(t *testing.T, scs []blocks.ROBlob, req interface{}) []*expectedBlobChunk {
	var expect []*expectedBlobChunk
	blockOffset := 0
	lastRoot := scs[0].BlockRoot()
	rreq, ok := req.(*ethpb.BlobSidecarsByRangeRequest)
	require.Equal(t, true, ok)
	var writes uint64
	for i := range scs {
		sc := scs[i]
		root := sc.BlockRoot()
		if root != lastRoot {
			blockOffset += 1
		}
		lastRoot = root

		if sc.Slot() < c.oldestSlot(t) {
			continue
		}
		if sc.Slot() < rreq.StartSlot || sc.Slot() > rreq.StartSlot+types.Slot(rreq.Count)-1 {
			continue
		}
		if writes == params.BeaconConfig().MaxRequestBlobSidecars {
			continue
		}
		expect = append(expect, &expectedBlobChunk{
			sidecar: &sc,
			code:    responseCodeSuccess,
			message: "",
		})
		writes += 1
	}
	return expect
}

func (c *blobsTestCase) runTestBlobSidecarsByRange(t *testing.T) {
	if c.serverHandle == nil {
		c.serverHandle = func(s *Service) rpcHandler { return s.blobSidecarsByRangeRPCHandler }
	}
	if c.defineExpected == nil {
		c.defineExpected = c.filterExpectedByRange
	}
	if c.requestFromSidecars == nil {
		c.requestFromSidecars = blobRangeRequestFromSidecars
	}
	if c.topic == "" {
		c.topic = p2p.RPCBlobSidecarsByRangeTopicV1
	}
	if c.oldestSlot == nil {
		c.oldestSlot = c.defaultOldestSlotByRange
	}
	if c.streamReader == nil {
		c.streamReader = defaultExpectedRequirer
	}
	c.run(t)
}

func TestBlobByRangeOK(t *testing.T) {
	origNC := params.BeaconConfig()
	// restore network config after test completes
	defer func() {
		params.OverrideBeaconConfig(origNC)
	}()
	// set MaxRequestBlobSidecars to a low-ish value so the test doesn't timeout.
	nc := params.BeaconConfig().Copy()
	nc.MaxRequestBlobSidecars = 100
	params.OverrideBeaconConfig(nc)

	cases := []*blobsTestCase{
		{
			name:    "beginning of window + 10",
			nblocks: 10,
		},
		{
			name:    "10 slots before window, 10 slots after, count = 20",
			nblocks: 10,
			requestFromSidecars: func(scs []blocks.ROBlob) interface{} {
				return &ethpb.BlobSidecarsByRangeRequest{
					StartSlot: scs[0].Slot() - 10,
					Count:     20,
				}
			},
		},
		{
			name:    "request before window, empty response",
			nblocks: 10,
			requestFromSidecars: func(scs []blocks.ROBlob) interface{} {
				return &ethpb.BlobSidecarsByRangeRequest{
					StartSlot: scs[0].Slot() - 10,
					Count:     10,
				}
			},
			total: func() *int { x := 0; return &x }(),
		},
		{
			name:    "10 blocks * 4 blobs = 40",
			nblocks: 10,
			requestFromSidecars: func(scs []blocks.ROBlob) interface{} {
				return &ethpb.BlobSidecarsByRangeRequest{
					StartSlot: scs[0].Slot() - 10,
					Count:     20,
				}
			},
			total: func() *int { x := fieldparams.MaxBlobsPerBlock * 10; return &x }(), // 10 blocks * 4 blobs = 40
		},
		{
			name:    "when request count > MAX_REQUEST_BLOCKS_DENEB, MAX_REQUEST_BLOBS_SIDECARS sidecars in response",
			nblocks: int(params.BeaconConfig().MaxRequestBlocksDeneb) + 10,
			requestFromSidecars: func(scs []blocks.ROBlob) interface{} {
				return &ethpb.BlobSidecarsByRangeRequest{
					StartSlot: scs[0].Slot(),
					Count:     params.BeaconConfig().MaxRequestBlocksDeneb + 1,
				}
			},
			total: func() *int { x := int(params.BeaconConfig().MaxRequestBlobSidecars); return &x }(),
		},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			c.runTestBlobSidecarsByRange(t)
		})
	}
}

func TestBlobsByRangeValidation(t *testing.T) {
	cfg := params.BeaconConfig()
	repositionFutureEpochs(cfg)
	undo, err := params.SetActiveWithUndo(cfg)
	require.NoError(t, err)
	defer func() {
		require.NoError(t, undo())
	}()
	denebSlot, err := slots.EpochStart(params.BeaconConfig().DenebForkEpoch)
	require.NoError(t, err)

	minReqEpochs := params.BeaconConfig().MinEpochsForBlobsSidecarsRequest
	minReqSlots, err := slots.EpochStart(minReqEpochs)
	require.NoError(t, err)
	// spec criteria for mix,max bound checking
	/*
		Clients MUST keep a record of signed blobs sidecars seen on the epoch range
		[max(current_epoch - MIN_EPOCHS_FOR_BLOB_SIDECARS_REQUESTS, DENEB_FORK_EPOCH), current_epoch]
		where current_epoch is defined by the current wall-clock time,
		and clients MUST support serving requests of blobs on this range.
	*/
	defaultCurrent := denebSlot + 100 + minReqSlots
	defaultMinStart, err := BlobsByRangeMinStartSlot(defaultCurrent)
	require.NoError(t, err)
	cases := []struct {
		name    string
		current types.Slot
		req     *ethpb.BlobSidecarsByRangeRequest
		// chain := defaultMockChain(t)

		start types.Slot
		end   types.Slot
		batch uint64
		err   error
	}{
		{
			name:    "start at current",
			current: denebSlot + 100,
			req: &ethpb.BlobSidecarsByRangeRequest{
				StartSlot: denebSlot + 100,
				Count:     10,
			},
			start: denebSlot + 100,
			end:   denebSlot + 100,
			batch: 10,
		},
		{
			name:    "start after current",
			current: denebSlot,
			req: &ethpb.BlobSidecarsByRangeRequest{
				StartSlot: denebSlot + 100,
				Count:     10,
			},
			start: denebSlot,
			end:   denebSlot,
			batch: 0,
		},
		{
			name:    "start before current_epoch - MIN_EPOCHS_FOR_BLOB_SIDECARS_REQUESTS",
			current: defaultCurrent,
			req: &ethpb.BlobSidecarsByRangeRequest{
				StartSlot: defaultMinStart - 100,
				Count:     10,
			},
			start: defaultMinStart,
			end:   defaultMinStart,
			batch: 10,
		},
		{
			name:    "start before current_epoch - MIN_EPOCHS_FOR_BLOB_SIDECARS_REQUESTS - end still valid",
			current: defaultCurrent,
			req: &ethpb.BlobSidecarsByRangeRequest{
				StartSlot: defaultMinStart - 10,
				Count:     20,
			},
			start: defaultMinStart,
			end:   defaultMinStart + 9,
			batch: blobBatchLimit(),
		},
		{
			name:    "count > MAX_REQUEST_BLOB_SIDECARS",
			current: defaultCurrent,
			req: &ethpb.BlobSidecarsByRangeRequest{
				StartSlot: defaultMinStart - 10,
				Count:     1000,
			},
			start: defaultMinStart,
			end:   defaultMinStart - 10 + 999,
			// a large count is ok, we just limit the amount of actual responses
			batch: blobBatchLimit(),
		},
		{
			name:    "start + count > current",
			current: defaultCurrent,
			req: &ethpb.BlobSidecarsByRangeRequest{
				StartSlot: defaultCurrent + 100,
				Count:     100,
			},
			start: defaultCurrent,
			end:   defaultCurrent,
			batch: 0,
		},
		{
			name:    "start before deneb",
			current: defaultCurrent - minReqSlots + 100,
			req: &ethpb.BlobSidecarsByRangeRequest{
				StartSlot: denebSlot - 10,
				Count:     100,
			},
			start: denebSlot,
			end:   denebSlot + 89,
			batch: blobBatchLimit(),
		},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			rp, err := validateBlobsByRange(c.req, c.current)
			if c.err != nil {
				require.ErrorIs(t, err, c.err)
				return
			} else {
				require.NoError(t, err)
			}
			require.Equal(t, c.start, rp.start)
			require.Equal(t, c.end, rp.end)
			require.Equal(t, c.batch, rp.size)
		})
	}
}
