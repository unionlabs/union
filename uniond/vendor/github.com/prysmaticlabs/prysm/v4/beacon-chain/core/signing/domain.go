package signing

import (
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// Domain returns the domain version for BLS private key to sign and verify.
//
// Spec pseudocode definition:
//
//	def get_domain(state: BeaconState, domain_type: DomainType, epoch: Epoch=None) -> Domain:
//	  """
//	  Return the signature domain (fork version concatenated with domain type) of a message.
//	  """
//	  epoch = get_current_epoch(state) if epoch is None else epoch
//	  fork_version = state.fork.previous_version if epoch < state.fork.epoch else state.fork.current_version
//	  return compute_domain(domain_type, fork_version, state.genesis_validators_root)
func Domain(fork *eth.Fork, epoch primitives.Epoch, domainType [bls.DomainByteLength]byte, genesisRoot []byte) ([]byte, error) {
	if fork == nil {
		return []byte{}, errors.New("nil fork or domain type")
	}
	var forkVersion []byte
	if epoch < fork.Epoch {
		forkVersion = fork.PreviousVersion
	} else {
		forkVersion = fork.CurrentVersion
	}
	if len(forkVersion) != 4 {
		return []byte{}, errors.New("fork version length is not 4 byte")
	}
	var forkVersionArray [4]byte
	copy(forkVersionArray[:], forkVersion[:4])
	return ComputeDomain(domainType, forkVersionArray[:], genesisRoot)
}
