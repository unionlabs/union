package p2p

import (
	"context"
	"strings"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/enr"
	"github.com/holiman/uint256"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/go-bitfield"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/flags"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/wrapper"
	"github.com/prysmaticlabs/prysm/v4/crypto/hash"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	mathutil "github.com/prysmaticlabs/prysm/v4/math"
	"go.opencensus.io/trace"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	pb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

var attestationSubnetCount = params.BeaconConfig().AttestationSubnetCount
var syncCommsSubnetCount = params.BeaconConfig().SyncCommitteeSubnetCount

var attSubnetEnrKey = params.BeaconNetworkConfig().AttSubnetKey
var syncCommsSubnetEnrKey = params.BeaconNetworkConfig().SyncCommsSubnetKey

// The value used with the subnet, inorder
// to create an appropriate key to retrieve
// the relevant lock. This is used to differentiate
// sync subnets from attestation subnets. This is deliberately
// chosen as more than 64(attestation subnet count).
const syncLockerVal = 100

// The value used with the blob sidecar subnet, in order
// to create an appropriate key to retrieve
// the relevant lock. This is used to differentiate
// blob subnets from others. This is deliberately
// chosen more than sync and attestation subnet combined.
const blobSubnetLockerVal = 110

// FindPeersWithSubnet performs a network search for peers
// subscribed to a particular subnet. Then we try to connect
// with those peers. This method will block until the required amount of
// peers are found, the method only exits in the event of context timeouts.
func (s *Service) FindPeersWithSubnet(ctx context.Context, topic string,
	index uint64, threshold int) (bool, error) {
	ctx, span := trace.StartSpan(ctx, "p2p.FindPeersWithSubnet")
	defer span.End()

	span.AddAttributes(trace.Int64Attribute("index", int64(index))) // lint:ignore uintcast -- It's safe to do this for tracing.

	if s.dv5Listener == nil {
		// return if discovery isn't set
		return false, nil
	}

	topic += s.Encoding().ProtocolSuffix()
	iterator := s.dv5Listener.RandomNodes()
	defer iterator.Close()
	switch {
	case strings.Contains(topic, GossipAttestationMessage):
		iterator = filterNodes(ctx, iterator, s.filterPeerForAttSubnet(index))
	case strings.Contains(topic, GossipSyncCommitteeMessage):
		iterator = filterNodes(ctx, iterator, s.filterPeerForSyncSubnet(index))
	default:
		return false, errors.New("no subnet exists for provided topic")
	}

	currNum := len(s.pubsub.ListPeers(topic))
	wg := new(sync.WaitGroup)
	for {
		if currNum >= threshold {
			break
		}
		if err := ctx.Err(); err != nil {
			return false, errors.Errorf("unable to find requisite number of peers for topic %s - "+
				"only %d out of %d peers were able to be found", topic, currNum, threshold)
		}
		nodes := enode.ReadNodes(iterator, int(params.BeaconNetworkConfig().MinimumPeersInSubnetSearch))
		for _, node := range nodes {
			info, _, err := convertToAddrInfo(node)
			if err != nil {
				continue
			}
			wg.Add(1)
			go func() {
				if err := s.connectWithPeer(ctx, *info); err != nil {
					log.WithError(err).Tracef("Could not connect with peer %s", info.String())
				}
				wg.Done()
			}()
		}
		// Wait for all dials to be completed.
		wg.Wait()
		currNum = len(s.pubsub.ListPeers(topic))
	}
	return true, nil
}

// returns a method with filters peers specifically for a particular attestation subnet.
func (s *Service) filterPeerForAttSubnet(index uint64) func(node *enode.Node) bool {
	return func(node *enode.Node) bool {
		if !s.filterPeer(node) {
			return false
		}
		subnets, err := attSubnets(node.Record())
		if err != nil {
			return false
		}
		indExists := false
		for _, comIdx := range subnets {
			if comIdx == index {
				indExists = true
				break
			}
		}
		return indExists
	}
}

// returns a method with filters peers specifically for a particular sync subnet.
func (s *Service) filterPeerForSyncSubnet(index uint64) func(node *enode.Node) bool {
	return func(node *enode.Node) bool {
		if !s.filterPeer(node) {
			return false
		}
		subnets, err := syncSubnets(node.Record())
		if err != nil {
			return false
		}
		indExists := false
		for _, comIdx := range subnets {
			if comIdx == index {
				indExists = true
				break
			}
		}
		return indExists
	}
}

// lower threshold to broadcast object compared to searching
// for a subnet. So that even in the event of poor peer
// connectivity, we can still broadcast an attestation.
func (s *Service) hasPeerWithSubnet(topic string) bool {
	// In the event peer threshold is lower, we will choose the lower
	// threshold.
	minPeers := mathutil.Min(1, uint64(flags.Get().MinimumPeersPerSubnet))
	return len(s.pubsub.ListPeers(topic+s.Encoding().ProtocolSuffix())) >= int(minPeers) // lint:ignore uintcast -- Min peers can be safely cast to int.
}

// Updates the service's discv5 listener record's attestation subnet
// with a new value for a bitfield of subnets tracked. It also updates
// the node's metadata by increasing the sequence number and the
// subnets tracked by the node.
func (s *Service) updateSubnetRecordWithMetadata(bitV bitfield.Bitvector64) {
	entry := enr.WithEntry(attSubnetEnrKey, &bitV)
	s.dv5Listener.LocalNode().Set(entry)
	s.metaData = wrapper.WrappedMetadataV0(&pb.MetaDataV0{
		SeqNumber: s.metaData.SequenceNumber() + 1,
		Attnets:   bitV,
	})
}

// Updates the service's discv5 listener record's attestation subnet
// with a new value for a bitfield of subnets tracked. It also record's
// the sync committee subnet in the enr. It also updates the node's
// metadata by increasing the sequence number and the subnets tracked by the node.
func (s *Service) updateSubnetRecordWithMetadataV2(bitVAtt bitfield.Bitvector64, bitVSync bitfield.Bitvector4) {
	entry := enr.WithEntry(attSubnetEnrKey, &bitVAtt)
	subEntry := enr.WithEntry(syncCommsSubnetEnrKey, &bitVSync)
	s.dv5Listener.LocalNode().Set(entry)
	s.dv5Listener.LocalNode().Set(subEntry)
	s.metaData = wrapper.WrappedMetadataV1(&pb.MetaDataV1{
		SeqNumber: s.metaData.SequenceNumber() + 1,
		Attnets:   bitVAtt,
		Syncnets:  bitVSync,
	})
}

func initializePersistentSubnets(id enode.ID, epoch primitives.Epoch) error {
	_, ok, expTime := cache.SubnetIDs.GetPersistentSubnets()
	if ok && expTime.After(time.Now()) {
		return nil
	}
	subs, err := computeSubscribedSubnets(id, epoch)
	if err != nil {
		return err
	}
	newExpTime := computeSubscriptionExpirationTime(id, epoch)
	cache.SubnetIDs.AddPersistentCommittee(subs, newExpTime)
	return nil
}

// Spec pseudocode definition:
//
// def compute_subscribed_subnets(node_id: NodeID, epoch: Epoch) -> Sequence[SubnetID]:
//
//	return [compute_subscribed_subnet(node_id, epoch, index) for index in range(SUBNETS_PER_NODE)]
func computeSubscribedSubnets(nodeID enode.ID, epoch primitives.Epoch) ([]uint64, error) {
	subs := []uint64{}
	for i := uint64(0); i < params.BeaconConfig().SubnetsPerNode; i++ {
		sub, err := computeSubscribedSubnet(nodeID, epoch, i)
		if err != nil {
			return nil, err
		}
		subs = append(subs, sub)
	}
	return subs, nil
}

//	Spec pseudocode definition:
//
// def compute_subscribed_subnet(node_id: NodeID, epoch: Epoch, index: int) -> SubnetID:
//
//	node_id_prefix = node_id >> (NODE_ID_BITS - ATTESTATION_SUBNET_PREFIX_BITS)
//	node_offset = node_id % EPOCHS_PER_SUBNET_SUBSCRIPTION
//	permutation_seed = hash(uint_to_bytes(uint64((epoch + node_offset) // EPOCHS_PER_SUBNET_SUBSCRIPTION)))
//	permutated_prefix = compute_shuffled_index(
//	    node_id_prefix,
//	    1 << ATTESTATION_SUBNET_PREFIX_BITS,
//	    permutation_seed,
//	)
//	return SubnetID((permutated_prefix + index) % ATTESTATION_SUBNET_COUNT)
func computeSubscribedSubnet(nodeID enode.ID, epoch primitives.Epoch, index uint64) (uint64, error) {
	nodeOffset, nodeIdPrefix := computeOffsetAndPrefix(nodeID)
	seedInput := (nodeOffset + uint64(epoch)) / params.BeaconConfig().EpochsPerSubnetSubscription
	permSeed := hash.Hash(bytesutil.Bytes8(seedInput))
	permutatedPrefix, err := helpers.ComputeShuffledIndex(primitives.ValidatorIndex(nodeIdPrefix), 1<<params.BeaconConfig().AttestationSubnetPrefixBits, permSeed, true)
	if err != nil {
		return 0, err
	}
	subnet := (uint64(permutatedPrefix) + index) % params.BeaconConfig().AttestationSubnetCount
	return subnet, nil
}

func computeSubscriptionExpirationTime(nodeID enode.ID, epoch primitives.Epoch) time.Duration {
	nodeOffset, _ := computeOffsetAndPrefix(nodeID)
	pastEpochs := (nodeOffset + uint64(epoch)) % params.BeaconConfig().EpochsPerSubnetSubscription
	remEpochs := params.BeaconConfig().EpochsPerSubnetSubscription - pastEpochs
	epochDuration := time.Duration(params.BeaconConfig().SlotsPerEpoch.Mul(params.BeaconConfig().SecondsPerSlot))
	epochTime := time.Duration(remEpochs) * epochDuration
	return epochTime * time.Second
}

func computeOffsetAndPrefix(nodeID enode.ID) (uint64, uint64) {
	num := uint256.NewInt(0).SetBytes(nodeID.Bytes())
	remBits := params.BeaconConfig().NodeIdBits - params.BeaconConfig().AttestationSubnetPrefixBits
	// Number of bits left will be representable by a uint64 value.
	nodeIdPrefix := num.Rsh(num, uint(remBits)).Uint64()
	// Reinitialize big int.
	num = uint256.NewInt(0).SetBytes(nodeID.Bytes())
	nodeOffset := num.Mod(num, uint256.NewInt(params.BeaconConfig().EpochsPerSubnetSubscription)).Uint64()
	return nodeOffset, nodeIdPrefix
}

// Initializes a bitvector of attestation subnets beacon nodes is subscribed to
// and creates a new ENR entry with its default value.
func initializeAttSubnets(node *enode.LocalNode) *enode.LocalNode {
	bitV := bitfield.NewBitvector64()
	entry := enr.WithEntry(attSubnetEnrKey, bitV.Bytes())
	node.Set(entry)
	return node
}

// Initializes a bitvector of sync committees subnets beacon nodes is subscribed to
// and creates a new ENR entry with its default value.
func initializeSyncCommSubnets(node *enode.LocalNode) *enode.LocalNode {
	bitV := bitfield.Bitvector4{byte(0x00)}
	entry := enr.WithEntry(syncCommsSubnetEnrKey, bitV.Bytes())
	node.Set(entry)
	return node
}

// Reads the attestation subnets entry from a node's ENR and determines
// the committee indices of the attestation subnets the node is subscribed to.
func attSubnets(record *enr.Record) ([]uint64, error) {
	bitV, err := attBitvector(record)
	if err != nil {
		return nil, err
	}
	// lint:ignore uintcast -- subnet count can be safely cast to int.
	if len(bitV) != byteCount(int(attestationSubnetCount)) {
		return []uint64{}, errors.Errorf("invalid bitvector provided, it has a size of %d", len(bitV))
	}
	var committeeIdxs []uint64
	for i := uint64(0); i < attestationSubnetCount; i++ {
		if bitV.BitAt(i) {
			committeeIdxs = append(committeeIdxs, i)
		}
	}
	return committeeIdxs, nil
}

// Reads the sync subnets entry from a node's ENR and determines
// the committee indices of the sync subnets the node is subscribed to.
func syncSubnets(record *enr.Record) ([]uint64, error) {
	bitV, err := syncBitvector(record)
	if err != nil {
		return nil, err
	}
	// lint:ignore uintcast -- subnet count can be safely cast to int.
	if len(bitV) != byteCount(int(syncCommsSubnetCount)) {
		return []uint64{}, errors.Errorf("invalid bitvector provided, it has a size of %d", len(bitV))
	}
	var committeeIdxs []uint64
	for i := uint64(0); i < syncCommsSubnetCount; i++ {
		if bitV.BitAt(i) {
			committeeIdxs = append(committeeIdxs, i)
		}
	}
	return committeeIdxs, nil
}

// Parses the attestation subnets ENR entry in a node and extracts its value
// as a bitvector for further manipulation.
func attBitvector(record *enr.Record) (bitfield.Bitvector64, error) {
	bitV := bitfield.NewBitvector64()
	entry := enr.WithEntry(attSubnetEnrKey, &bitV)
	err := record.Load(entry)
	if err != nil {
		return nil, err
	}
	return bitV, nil
}

// Parses the attestation subnets ENR entry in a node and extracts its value
// as a bitvector for further manipulation.
func syncBitvector(record *enr.Record) (bitfield.Bitvector4, error) {
	bitV := bitfield.Bitvector4{byte(0x00)}
	entry := enr.WithEntry(syncCommsSubnetEnrKey, &bitV)
	err := record.Load(entry)
	if err != nil {
		return nil, err
	}
	return bitV, nil
}

// The subnet locker is a map which keeps track of all
// mutexes stored per subnet. This locker is re-used
// between both the attestation and sync subnets. In
// order to differentiate between attestation and sync
// subnets. Sync subnets are stored by (subnet+syncLockerVal). This
// is to prevent conflicts while allowing both subnets
// to use a single locker.
func (s *Service) subnetLocker(i uint64) *sync.RWMutex {
	s.subnetsLockLock.Lock()
	defer s.subnetsLockLock.Unlock()
	l, ok := s.subnetsLock[i]
	if !ok {
		l = &sync.RWMutex{}
		s.subnetsLock[i] = l
	}
	return l
}

// Determines the number of bytes that are used
// to represent the provided number of bits.
func byteCount(bitCount int) int {
	numOfBytes := bitCount / 8
	if bitCount%8 != 0 {
		numOfBytes++
	}
	return numOfBytes
}
