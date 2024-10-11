// Package iface defines an interface for the validator database.
package iface

import (
	"context"
	"io"

	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	validatorServiceConfig "github.com/prysmaticlabs/prysm/v4/config/validator/service"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/monitoring/backup"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/validator/db/kv"
)

// Ensure the kv store implements the interface.
var _ = ValidatorDB(&kv.Store{})

// ValidatorDB defines the necessary methods for a Prysm validator DB.
type ValidatorDB interface {
	io.Closer
	backup.Exporter
	DatabasePath() string
	ClearDB() error
	RunUpMigrations(ctx context.Context) error
	RunDownMigrations(ctx context.Context) error
	UpdatePublicKeysBuckets(publicKeys [][fieldparams.BLSPubkeyLength]byte) error

	// Genesis information related methods.
	GenesisValidatorsRoot(ctx context.Context) ([]byte, error)
	SaveGenesisValidatorsRoot(ctx context.Context, genValRoot []byte) error

	// Proposer protection related methods.
	HighestSignedProposal(ctx context.Context, publicKey [fieldparams.BLSPubkeyLength]byte) (primitives.Slot, bool, error)
	LowestSignedProposal(ctx context.Context, publicKey [fieldparams.BLSPubkeyLength]byte) (primitives.Slot, bool, error)
	ProposalHistoryForPubKey(ctx context.Context, publicKey [fieldparams.BLSPubkeyLength]byte) ([]*kv.Proposal, error)
	ProposalHistoryForSlot(ctx context.Context, publicKey [fieldparams.BLSPubkeyLength]byte, slot primitives.Slot) ([32]byte, bool, bool, error)
	SaveProposalHistoryForSlot(ctx context.Context, pubKey [fieldparams.BLSPubkeyLength]byte, slot primitives.Slot, signingRoot []byte) error
	ProposedPublicKeys(ctx context.Context) ([][fieldparams.BLSPubkeyLength]byte, error)

	// Attester protection related methods.
	// Methods to store and read blacklisted public keys from EIP-3076
	// slashing protection imports.
	EIPImportBlacklistedPublicKeys(ctx context.Context) ([][fieldparams.BLSPubkeyLength]byte, error)
	SaveEIPImportBlacklistedPublicKeys(ctx context.Context, publicKeys [][fieldparams.BLSPubkeyLength]byte) error
	SigningRootAtTargetEpoch(ctx context.Context, publicKey [fieldparams.BLSPubkeyLength]byte, target primitives.Epoch) ([]byte, error)
	LowestSignedTargetEpoch(ctx context.Context, publicKey [fieldparams.BLSPubkeyLength]byte) (primitives.Epoch, bool, error)
	LowestSignedSourceEpoch(ctx context.Context, publicKey [fieldparams.BLSPubkeyLength]byte) (primitives.Epoch, bool, error)
	AttestedPublicKeys(ctx context.Context) ([][fieldparams.BLSPubkeyLength]byte, error)
	CheckSlashableAttestation(
		ctx context.Context, pubKey [fieldparams.BLSPubkeyLength]byte, signingRoot []byte, att *ethpb.IndexedAttestation,
	) (kv.SlashingKind, error)
	SaveAttestationForPubKey(
		ctx context.Context, pubKey [fieldparams.BLSPubkeyLength]byte, signingRoot [32]byte, att *ethpb.IndexedAttestation,
	) error
	SaveAttestationsForPubKey(
		ctx context.Context, pubKey [fieldparams.BLSPubkeyLength]byte, signingRoots [][]byte, atts []*ethpb.IndexedAttestation,
	) error
	AttestationHistoryForPubKey(
		ctx context.Context, pubKey [fieldparams.BLSPubkeyLength]byte,
	) ([]*kv.AttestationRecord, error)

	// Graffiti ordered index related methods
	SaveGraffitiOrderedIndex(ctx context.Context, index uint64) error
	GraffitiOrderedIndex(ctx context.Context, fileHash [32]byte) (uint64, error)

	// ProposerSettings related methods
	ProposerSettings(context.Context) (*validatorServiceConfig.ProposerSettings, error)
	ProposerSettingsExists(ctx context.Context) (bool, error)
	UpdateProposerSettingsDefault(context.Context, *validatorServiceConfig.ProposerOption) error
	UpdateProposerSettingsForPubkey(context.Context, [fieldparams.BLSPubkeyLength]byte, *validatorServiceConfig.ProposerOption) error
	SaveProposerSettings(ctx context.Context, settings *validatorServiceConfig.ProposerSettings) error
}
