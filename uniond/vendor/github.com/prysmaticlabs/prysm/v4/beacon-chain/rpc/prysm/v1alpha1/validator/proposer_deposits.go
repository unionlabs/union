package validator

import (
	"bytes"
	"context"
	"math/big"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/container/trie"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/sirupsen/logrus"
	"go.opencensus.io/trace"
	"golang.org/x/sync/errgroup"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

func (vs *Server) packDepositsAndAttestations(ctx context.Context, head state.BeaconState, eth1Data *ethpb.Eth1Data) ([]*ethpb.Deposit, []*ethpb.Attestation, error) {
	eg, egctx := errgroup.WithContext(ctx)
	var deposits []*ethpb.Deposit
	var atts []*ethpb.Attestation

	eg.Go(func() error {
		// Pack ETH1 deposits which have not been included in the beacon chain.
		localDeposits, err := vs.deposits(egctx, head, eth1Data)
		if err != nil {
			return status.Errorf(codes.Internal, "Could not get ETH1 deposits: %v", err)
		}
		// if the original context is cancelled, then cancel this routine too
		select {
		case <-egctx.Done():
			return egctx.Err()
		default:
		}
		deposits = localDeposits
		return nil
	})

	eg.Go(func() error {
		// Pack aggregated attestations which have not been included in the beacon chain.
		localAtts, err := vs.packAttestations(egctx, head)
		if err != nil {
			return status.Errorf(codes.Internal, "Could not get attestations to pack into block: %v", err)
		}
		// if the original context is cancelled, then cancel this routine too
		select {
		case <-egctx.Done():
			return egctx.Err()
		default:
		}
		atts = localAtts
		return nil
	})

	return deposits, atts, eg.Wait()
}

// deposits returns a list of pending deposits that are ready for inclusion in the next beacon
// block. Determining deposits depends on the current eth1data vote for the block and whether or not
// this eth1data has enough support to be considered for deposits inclusion. If current vote has
// enough support, then use that vote for basis of determining deposits, otherwise use current state
// eth1data.
func (vs *Server) deposits(
	ctx context.Context,
	beaconState state.BeaconState,
	currentVote *ethpb.Eth1Data,
) ([]*ethpb.Deposit, error) {
	ctx, span := trace.StartSpan(ctx, "ProposerServer.deposits")
	defer span.End()

	if vs.MockEth1Votes {
		return []*ethpb.Deposit{}, nil
	}

	if !vs.Eth1InfoFetcher.ExecutionClientConnected() {
		log.Warn("not connected to eth1 node, skip pending deposit insertion")
		return []*ethpb.Deposit{}, nil
	}
	// Need to fetch if the deposits up to the state's latest eth1 data matches
	// the number of all deposits in this RPC call. If not, then we return nil.
	canonicalEth1Data, canonicalEth1DataHeight, err := vs.canonicalEth1Data(ctx, beaconState, currentVote)
	if err != nil {
		return nil, err
	}

	_, genesisEth1Block := vs.Eth1InfoFetcher.GenesisExecutionChainInfo()
	if genesisEth1Block.Cmp(canonicalEth1DataHeight) == 0 {
		return []*ethpb.Deposit{}, nil
	}

	// If there are no pending deposits, exit early.
	allPendingContainers := vs.PendingDepositsFetcher.PendingContainers(ctx, canonicalEth1DataHeight)
	if len(allPendingContainers) == 0 {
		log.Debug("no pending deposits for inclusion in block")
		return []*ethpb.Deposit{}, nil
	}

	depositTrie, err := vs.depositTrie(ctx, canonicalEth1Data, canonicalEth1DataHeight)
	if err != nil {
		return nil, errors.Wrap(err, "could not retrieve deposit trie")
	}

	// Deposits need to be received in order of merkle index root, so this has to make sure
	// deposits are sorted from lowest to highest.
	var pendingDeps []*ethpb.DepositContainer
	for _, dep := range allPendingContainers {
		if uint64(dep.Index) >= beaconState.Eth1DepositIndex() && uint64(dep.Index) < canonicalEth1Data.DepositCount {
			pendingDeps = append(pendingDeps, dep)
		}
		// Don't try to pack more than the max allowed in a block
		if uint64(len(pendingDeps)) == params.BeaconConfig().MaxDeposits {
			break
		}
	}

	for i := range pendingDeps {
		pendingDeps[i].Deposit, err = constructMerkleProof(depositTrie, int(pendingDeps[i].Index), pendingDeps[i].Deposit)
		if err != nil {
			return nil, err
		}
	}

	var pendingDeposits []*ethpb.Deposit
	for i := uint64(0); i < uint64(len(pendingDeps)); i++ {
		pendingDeposits = append(pendingDeposits, pendingDeps[i].Deposit)
	}
	return pendingDeposits, nil
}

func (vs *Server) depositTrie(ctx context.Context, canonicalEth1Data *ethpb.Eth1Data, canonicalEth1DataHeight *big.Int) (cache.MerkleTree, error) {
	ctx, span := trace.StartSpan(ctx, "ProposerServer.depositTrie")
	defer span.End()

	var depositTrie cache.MerkleTree

	finalizedDeposits, err := vs.DepositFetcher.FinalizedDeposits(ctx)
	if err != nil {
		return nil, err
	}
	depositTrie = finalizedDeposits.Deposits()
	upToEth1DataDeposits := vs.DepositFetcher.NonFinalizedDeposits(ctx, finalizedDeposits.MerkleTrieIndex(), canonicalEth1DataHeight)
	insertIndex := finalizedDeposits.MerkleTrieIndex() + 1

	if shouldRebuildTrie(canonicalEth1Data.DepositCount, uint64(len(upToEth1DataDeposits))) {
		log.WithFields(logrus.Fields{
			"unfinalized deposits": len(upToEth1DataDeposits),
			"total deposit count":  canonicalEth1Data.DepositCount,
		}).Warn("Too many unfinalized deposits, building a deposit trie from scratch.")
		return vs.rebuildDepositTrie(ctx, canonicalEth1Data, canonicalEth1DataHeight)
	}
	for _, dep := range upToEth1DataDeposits {
		depHash, err := dep.Data.HashTreeRoot()
		if err != nil {
			return nil, errors.Wrap(err, "could not hash deposit data")
		}
		if err = depositTrie.Insert(depHash[:], int(insertIndex)); err != nil {
			return nil, err
		}
		insertIndex++
	}
	valid, err := validateDepositTrie(depositTrie, canonicalEth1Data)
	// Log a warning here, as the cached trie is invalid.
	if !valid {
		log.WithError(err).Warn("Cached deposit trie is invalid, rebuilding it now")
		return vs.rebuildDepositTrie(ctx, canonicalEth1Data, canonicalEth1DataHeight)
	}

	return depositTrie, nil
}

// rebuilds our deposit trie by recreating it from all processed deposits till
// specified eth1 block height.
func (vs *Server) rebuildDepositTrie(ctx context.Context, canonicalEth1Data *ethpb.Eth1Data, canonicalEth1DataHeight *big.Int) (cache.MerkleTree, error) {
	ctx, span := trace.StartSpan(ctx, "ProposerServer.rebuildDepositTrie")
	defer span.End()

	deposits := vs.DepositFetcher.AllDeposits(ctx, canonicalEth1DataHeight)
	trieItems := make([][]byte, 0, len(deposits))
	for _, dep := range deposits {
		depHash, err := dep.Data.HashTreeRoot()
		if err != nil {
			return nil, errors.Wrap(err, "could not hash deposit data")
		}
		trieItems = append(trieItems, depHash[:])
	}
	depositTrie, err := trie.GenerateTrieFromItems(trieItems, params.BeaconConfig().DepositContractTreeDepth)
	if err != nil {
		return nil, err
	}

	valid, err := validateDepositTrie(depositTrie, canonicalEth1Data)
	// Log an error here, as even with rebuilding the trie, it is still invalid.
	if !valid {
		log.WithError(err).Error("Rebuilt deposit trie is invalid")
	}
	return depositTrie, nil
}

// validate that the provided deposit trie matches up with the canonical eth1 data provided.
func validateDepositTrie(trie cache.MerkleTree, canonicalEth1Data *ethpb.Eth1Data) (bool, error) {
	if trie == nil || canonicalEth1Data == nil {
		return false, errors.New("nil trie or eth1data provided")
	}
	if trie.NumOfItems() != int(canonicalEth1Data.DepositCount) {
		return false, errors.Errorf("wanted the canonical count of %d but received %d", canonicalEth1Data.DepositCount, trie.NumOfItems())
	}
	rt, err := trie.HashTreeRoot()
	if err != nil {
		return false, err
	}
	if !bytes.Equal(rt[:], canonicalEth1Data.DepositRoot) {
		return false, errors.Errorf("wanted the canonical deposit root of %#x but received %#x", canonicalEth1Data.DepositRoot, rt)
	}
	return true, nil
}

func constructMerkleProof(trie cache.MerkleTree, index int, deposit *ethpb.Deposit) (*ethpb.Deposit, error) {
	proof, err := trie.MerkleProof(index)
	if err != nil {
		return nil, errors.Wrapf(err, "could not generate merkle proof for deposit at index %d", index)
	}
	// For every deposit, we construct a Merkle proof using the powchain service's
	// in-memory deposits trie, which is updated only once the state's LatestETH1Data
	// property changes during a state transition after a voting period.
	deposit.Proof = proof
	return deposit, nil
}

// This checks whether we should fallback to rebuild the whole deposit trie.
func shouldRebuildTrie(totalDepCount, unFinalizedDeps uint64) bool {
	if totalDepCount == 0 || unFinalizedDeps == 0 {
		return false
	}
	// The total number interior nodes hashed in a binary trie would be
	// x - 1, where x is the total number of leaves of the trie. For simplicity's
	// sake we assume it as x here as this function is meant as a heuristic rather than
	// and exact calculation.
	//
	// Since the effective_depth = log(x) , the total depth can be represented as
	// depth = log(x) + k. We can then find the total number of nodes to be hashed by
	// calculating  y (log(x) + k) , where y is the number of unfinalized deposits. For
	// the deposit trie, the value of log(x) + k is fixed at 32.
	unFinalizedCompute := unFinalizedDeps * params.BeaconConfig().DepositContractTreeDepth
	return unFinalizedCompute > totalDepCount
}
