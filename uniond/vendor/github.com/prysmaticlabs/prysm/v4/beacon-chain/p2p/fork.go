package p2p

import (
	"bytes"
	"fmt"
	"time"

	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/enr"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/network/forks"
	pb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	prysmTime "github.com/prysmaticlabs/prysm/v4/time"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/sirupsen/logrus"
)

// ENR key used for Ethereum consensus-related fork data.
var eth2ENRKey = params.BeaconNetworkConfig().ETH2Key

// ForkDigest returns the current fork digest of
// the node according to the local clock.
func (s *Service) currentForkDigest() ([4]byte, error) {
	if !s.isInitialized() {
		return [4]byte{}, errors.New("state is not initialized")
	}
	return forks.CreateForkDigest(s.genesisTime, s.genesisValidatorsRoot)
}

// Compares fork ENRs between an incoming peer's record and our node's
// local record values for current and next fork version/epoch.
func (s *Service) compareForkENR(record *enr.Record) error {
	currentRecord := s.dv5Listener.LocalNode().Node().Record()
	peerForkENR, err := forkEntry(record)
	if err != nil {
		return err
	}
	currentForkENR, err := forkEntry(currentRecord)
	if err != nil {
		return err
	}
	enrString, err := SerializeENR(record)
	if err != nil {
		return err
	}
	// Clients SHOULD connect to peers with current_fork_digest, next_fork_version,
	// and next_fork_epoch that match local values.
	if !bytes.Equal(peerForkENR.CurrentForkDigest, currentForkENR.CurrentForkDigest) {
		return fmt.Errorf(
			"fork digest of peer with ENR %s: %v, does not match local value: %v",
			enrString,
			peerForkENR.CurrentForkDigest,
			currentForkENR.CurrentForkDigest,
		)
	}
	// Clients MAY connect to peers with the same current_fork_version but a
	// different next_fork_version/next_fork_epoch. Unless ENRForkID is manually
	// updated to matching prior to the earlier next_fork_epoch of the two clients,
	// these type of connecting clients will be unable to successfully interact
	// starting at the earlier next_fork_epoch.
	if peerForkENR.NextForkEpoch != currentForkENR.NextForkEpoch {
		log.WithFields(logrus.Fields{
			"peerNextForkEpoch": peerForkENR.NextForkEpoch,
			"peerENR":           enrString,
		}).Trace("Peer matches fork digest but has different next fork epoch")
	}
	if !bytes.Equal(peerForkENR.NextForkVersion, currentForkENR.NextForkVersion) {
		log.WithFields(logrus.Fields{
			"peerNextForkVersion": peerForkENR.NextForkVersion,
			"peerENR":             enrString,
		}).Trace("Peer matches fork digest but has different next fork version")
	}
	return nil
}

// Adds a fork entry as an ENR record under the Ethereum consensus EnrKey for
// the local node. The fork entry is an ssz-encoded enrForkID type
// which takes into account the current fork version from the current
// epoch to create a fork digest, the next fork version,
// and the next fork epoch.
func addForkEntry(
	node *enode.LocalNode,
	genesisTime time.Time,
	genesisValidatorsRoot []byte,
) (*enode.LocalNode, error) {
	digest, err := forks.CreateForkDigest(genesisTime, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	currentSlot := slots.Since(genesisTime)
	currentEpoch := slots.ToEpoch(currentSlot)
	if prysmTime.Now().Before(genesisTime) {
		currentEpoch = 0
	}
	nextForkVersion, nextForkEpoch, err := forks.NextForkData(currentEpoch)
	if err != nil {
		return nil, err
	}
	enrForkID := &pb.ENRForkID{
		CurrentForkDigest: digest[:],
		NextForkVersion:   nextForkVersion[:],
		NextForkEpoch:     nextForkEpoch,
	}
	enc, err := enrForkID.MarshalSSZ()
	if err != nil {
		return nil, err
	}
	forkEntry := enr.WithEntry(eth2ENRKey, enc)
	node.Set(forkEntry)
	return node, nil
}

// Retrieves an enrForkID from an ENR record by key lookup
// under the Ethereum consensus EnrKey
func forkEntry(record *enr.Record) (*pb.ENRForkID, error) {
	sszEncodedForkEntry := make([]byte, 16)
	entry := enr.WithEntry(eth2ENRKey, &sszEncodedForkEntry)
	err := record.Load(entry)
	if err != nil {
		return nil, err
	}
	forkEntry := &pb.ENRForkID{}
	if err := forkEntry.UnmarshalSSZ(sszEncodedForkEntry); err != nil {
		return nil, err
	}
	return forkEntry, nil
}
