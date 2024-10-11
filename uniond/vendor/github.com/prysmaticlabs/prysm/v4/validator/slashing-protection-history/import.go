package history

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"

	"github.com/pkg/errors"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/slashings"
	"github.com/prysmaticlabs/prysm/v4/validator/db"
	"github.com/prysmaticlabs/prysm/v4/validator/db/kv"
	"github.com/prysmaticlabs/prysm/v4/validator/slashing-protection-history/format"
)

// ImportStandardProtectionJSON takes in EIP-3076 compliant JSON file used for slashing protection
// by Ethereum validators and imports its data into Prysm's internal representation of slashing
// protection in the validator client's database. For more information, see the EIP document here:
// https://eips.ethereum.org/EIPS/eip-3076.
func ImportStandardProtectionJSON(ctx context.Context, validatorDB db.Database, r io.Reader) error {
	encodedJSON, err := io.ReadAll(r)
	if err != nil {
		return errors.Wrap(err, "could not read slashing protection JSON file")
	}

	interchangeJSON := &format.EIPSlashingProtectionFormat{}
	if err := json.Unmarshal(encodedJSON, interchangeJSON); err != nil {
		return errors.Wrap(err, "could not unmarshal slashing protection JSON file")
	}

	if interchangeJSON.Data == nil {
		log.Warn("No slashing protection data to import")
		return nil
	}

	// We validate the `MetadataV0` field of the slashing protection JSON file.
	if err := validateMetadata(ctx, validatorDB, interchangeJSON); err != nil {
		return errors.Wrap(err, "slashing protection JSON metadata was incorrect")
	}

	// We need to handle duplicate public keys in the JSON file, with potentially
	// different signing histories for both attestations and blocks.
	signedBlocksByPubKey, err := parseBlocksForUniquePublicKeys(interchangeJSON.Data)
	if err != nil {
		return errors.Wrap(err, "could not parse unique entries for blocks by public key")
	}

	signedAttsByPubKey, err := parseAttestationsForUniquePublicKeys(interchangeJSON.Data)
	if err != nil {
		return errors.Wrap(err, "could not parse unique entries for attestations by public key")
	}

	attestingHistoryByPubKey := make(map[[fieldparams.BLSPubkeyLength]byte][]*kv.AttestationRecord)
	proposalHistoryByPubKey := make(map[[fieldparams.BLSPubkeyLength]byte]kv.ProposalHistoryForPubkey)

	for pubKey, signedBlocks := range signedBlocksByPubKey {
		// Transform the processed signed blocks data from the JSON
		// file into the internal Prysm representation of proposal history.
		proposalHistory, err := transformSignedBlocks(ctx, signedBlocks)
		if err != nil {
			return errors.Wrapf(err, "could not parse signed blocks in JSON file for key %#x", pubKey)
		}

		proposalHistoryByPubKey[pubKey] = *proposalHistory
	}

	for pubKey, signedAtts := range signedAttsByPubKey {
		// Transform the processed signed attestation data from the JSON
		// file into the internal Prysm representation of attesting history.
		historicalAtt, err := transformSignedAttestations(pubKey, signedAtts)
		if err != nil {
			return errors.Wrapf(err, "could not parse signed attestations in JSON file for key %#x", pubKey)
		}

		attestingHistoryByPubKey[pubKey] = historicalAtt
	}

	// We validate and filter out public keys parsed from JSON to ensure we are
	// not importing those which are slashable with respect to other data within the same JSON.
	slashableProposerKeys := filterSlashablePubKeysFromBlocks(ctx, proposalHistoryByPubKey)

	slashableAttesterKeys, err := filterSlashablePubKeysFromAttestations(ctx, validatorDB, attestingHistoryByPubKey)
	if err != nil {
		return errors.Wrap(err, "could not filter slashable attester public keys from JSON data")
	}

	slashablePublicKeysCount := len(slashableProposerKeys) + len(slashableAttesterKeys)
	slashablePublicKeys := make([][fieldparams.BLSPubkeyLength]byte, 0, slashablePublicKeysCount)
	slashablePublicKeys = append(slashablePublicKeys, slashableProposerKeys...)
	slashablePublicKeys = append(slashablePublicKeys, slashableAttesterKeys...)

	if err := validatorDB.SaveEIPImportBlacklistedPublicKeys(ctx, slashablePublicKeys); err != nil {
		return errors.Wrap(err, "could not save slashable public keys to database")
	}

	// We save the histories to disk as atomic operations, ensuring that this only occurs
	// until after we successfully parse all data from the JSON file. If there is any error
	// in parsing the JSON proposal and attesting histories, we will not reach this point.
	if err := saveProposals(ctx, proposalHistoryByPubKey, validatorDB); err != nil {
		return errors.Wrap(err, "could not save proposals")
	}

	if err := saveAttestations(ctx, attestingHistoryByPubKey, validatorDB); err != nil {
		return errors.Wrap(err, "could not save attestations")
	}

	return nil
}

func saveProposals(ctx context.Context, proposalHistoryByPubKey map[[fieldparams.BLSPubkeyLength]byte]kv.ProposalHistoryForPubkey, validatorDB db.Database) error {
	for pubKey, proposalHistory := range proposalHistoryByPubKey {
		bar := initializeProgressBar(
			len(proposalHistory.Proposals),
			fmt.Sprintf("Importing proposals for validator public key %#x", bytesutil.Trunc(pubKey[:])),
		)

		for _, proposal := range proposalHistory.Proposals {
			if err := bar.Add(1); err != nil {
				log.WithError(err).Debug("Could not increase progress bar")
			}

			if err := validatorDB.SaveProposalHistoryForSlot(ctx, pubKey, proposal.Slot, proposal.SigningRoot); err != nil {
				return errors.Wrap(err, "could not save proposal history from imported JSON to database")
			}
		}
	}

	return nil
}

func saveAttestations(ctx context.Context, attestingHistoryByPubKey map[[fieldparams.BLSPubkeyLength]byte][]*kv.AttestationRecord, validatorDB db.Database) error {
	bar := initializeProgressBar(
		len(attestingHistoryByPubKey),
		"Importing attesting history for validator public keys",
	)

	for pubKey, attestations := range attestingHistoryByPubKey {
		if err := bar.Add(1); err != nil {
			log.WithError(err).Debug("Could not increase progress bar")
		}

		indexedAtts := make([]*ethpb.IndexedAttestation, len(attestations))
		signingRoots := make([][]byte, len(attestations))

		for i, att := range attestations {
			indexedAtt := createAttestation(att.Source, att.Target)
			indexedAtts[i] = indexedAtt
			signingRoots[i] = att.SigningRoot
		}

		if err := validatorDB.SaveAttestationsForPubKey(ctx, pubKey, signingRoots, indexedAtts); err != nil {
			return errors.Wrap(err, "could not save attestations from imported JSON to database")
		}
	}

	return nil
}

func validateMetadata(ctx context.Context, validatorDB db.Database, interchangeJSON *format.EIPSlashingProtectionFormat) error {
	// We need to ensure the version in the metadata field matches the one we support.
	version := interchangeJSON.Metadata.InterchangeFormatVersion
	if version != format.InterchangeFormatVersion {
		return fmt.Errorf(
			"slashing protection JSON version '%s' is not supported, wanted '%s'",
			version,
			format.InterchangeFormatVersion,
		)
	}

	// We need to verify the genesis validators root matches that of our chain data, otherwise
	// the imported slashing protection JSON was created on a different chain.
	gvr, err := RootFromHex(interchangeJSON.Metadata.GenesisValidatorsRoot)
	if err != nil {
		return fmt.Errorf("%#x is not a valid root: %w", interchangeJSON.Metadata.GenesisValidatorsRoot, err)
	}
	dbGvr, err := validatorDB.GenesisValidatorsRoot(ctx)
	if err != nil {
		return errors.Wrap(err, "could not retrieve genesis validators root to db")
	}
	if dbGvr == nil {
		if err = validatorDB.SaveGenesisValidatorsRoot(ctx, gvr[:]); err != nil {
			return errors.Wrap(err, "could not save genesis validators root to db")
		}
		return nil
	}
	if !bytes.Equal(dbGvr, gvr[:]) {
		return errors.New("genesis validators root doesn't match the one that is stored in slashing protection db. " +
			"Please make sure you import the protection data that is relevant to the chain you are on")
	}
	return nil
}

// We create a map of pubKey -> []*SignedBlock. Then, for each public key we observe,
// we append to this map. This allows us to handle valid input JSON data such as:
//
//	"0x2932232930: {
//	  SignedBlocks: [Slot: 5, Slot: 6, Slot: 7],
//	 },
//
//	"0x2932232930: {
//	  SignedBlocks: [Slot: 5, Slot: 10, Slot: 11],
//	 }
//
// Which should be properly parsed as:
//
//	"0x2932232930: {
//	  SignedBlocks: [Slot: 5, Slot: 5, Slot: 6, Slot: 7, Slot: 10, Slot: 11],
//	 }
func parseBlocksForUniquePublicKeys(data []*format.ProtectionData) (map[[fieldparams.BLSPubkeyLength]byte][]*format.SignedBlock, error) {
	signedBlocksByPubKey := make(map[[fieldparams.BLSPubkeyLength]byte][]*format.SignedBlock)
	for _, validatorData := range data {
		pubKey, err := PubKeyFromHex(validatorData.Pubkey)
		if err != nil {
			return nil, fmt.Errorf("%s is not a valid public key: %w", validatorData.Pubkey, err)
		}
		for _, sBlock := range validatorData.SignedBlocks {
			if sBlock == nil {
				continue
			}
			signedBlocksByPubKey[pubKey] = append(signedBlocksByPubKey[pubKey], sBlock)
		}
	}
	return signedBlocksByPubKey, nil
}

// We create a map of pubKey -> []*SignedAttestation. Then, for each public key we observe,
// we append to this map. This allows us to handle valid input JSON data such as:
//
//	"0x2932232930: {
//	  SignedAttestations: [{Source: 5, Target: 6}, {Source: 6, Target: 7}],
//	 },
//
//	"0x2932232930: {
//	  SignedAttestations: [{Source: 5, Target: 6}],
//	 }
//
// Which should be properly parsed as:
//
//	"0x2932232930: {
//	  SignedAttestations: [{Source: 5, Target: 6}, {Source: 5, Target: 6}, {Source: 6, Target: 7}],
//	 }
func parseAttestationsForUniquePublicKeys(data []*format.ProtectionData) (map[[fieldparams.BLSPubkeyLength]byte][]*format.SignedAttestation, error) {
	signedAttestationsByPubKey := make(map[[fieldparams.BLSPubkeyLength]byte][]*format.SignedAttestation)
	for _, validatorData := range data {
		pubKey, err := PubKeyFromHex(validatorData.Pubkey)
		if err != nil {
			return nil, fmt.Errorf("%s is not a valid public key: %w", validatorData.Pubkey, err)
		}
		for _, sAtt := range validatorData.SignedAttestations {
			if sAtt == nil {
				continue
			}
			signedAttestationsByPubKey[pubKey] = append(signedAttestationsByPubKey[pubKey], sAtt)
		}
	}
	return signedAttestationsByPubKey, nil
}

func filterSlashablePubKeysFromBlocks(_ context.Context, historyByPubKey map[[fieldparams.BLSPubkeyLength]byte]kv.ProposalHistoryForPubkey) [][fieldparams.BLSPubkeyLength]byte {
	// Given signing roots are optional in the EIP standard, we behave as follows:
	// For a given block:
	//   If we have a previous block with the same slot in our history:
	//     If signing root is nil, we consider that proposer public key as slashable
	//     If signing root is not nil , then we compare signing roots. If they are different,
	//     then we consider that proposer public key as slashable.
	slashablePubKeys := make([][fieldparams.BLSPubkeyLength]byte, 0)
	for pubKey, proposals := range historyByPubKey {
		seenSigningRootsBySlot := make(map[primitives.Slot][]byte)
		for _, blk := range proposals.Proposals {
			if signingRoot, ok := seenSigningRootsBySlot[blk.Slot]; ok {
				if signingRoot == nil || !bytes.Equal(signingRoot, blk.SigningRoot) {
					slashablePubKeys = append(slashablePubKeys, pubKey)
					break
				}
			}
			seenSigningRootsBySlot[blk.Slot] = blk.SigningRoot
		}
	}
	return slashablePubKeys
}

func filterSlashablePubKeysFromAttestations(
	ctx context.Context,
	validatorDB db.Database,
	signedAttsByPubKey map[[fieldparams.BLSPubkeyLength]byte][]*kv.AttestationRecord,
) ([][fieldparams.BLSPubkeyLength]byte, error) {
	slashablePubKeys := make([][fieldparams.BLSPubkeyLength]byte, 0)
	// First we need to find attestations that are slashable with respect to other
	// attestations within the same JSON import.
	for pubKey, signedAtts := range signedAttsByPubKey {
		signingRootsByTarget := make(map[primitives.Epoch][]byte)
		targetEpochsBySource := make(map[primitives.Epoch][]primitives.Epoch)
	Loop:
		for _, att := range signedAtts {
			// Check for double votes.
			if sr, ok := signingRootsByTarget[att.Target]; ok {
				if slashings.SigningRootsDiffer(sr, att.SigningRoot) {
					slashablePubKeys = append(slashablePubKeys, pubKey)
					break Loop
				}
			}
			// Check for surround voting.
			for source, targets := range targetEpochsBySource {
				for _, target := range targets {
					a := createAttestation(source, target)
					b := createAttestation(att.Source, att.Target)
					if slashings.IsSurround(a, b) || slashings.IsSurround(b, a) {
						slashablePubKeys = append(slashablePubKeys, pubKey)
						break Loop
					}
				}
			}
			signingRootsByTarget[att.Target] = att.SigningRoot
			targetEpochsBySource[att.Source] = append(targetEpochsBySource[att.Source], att.Target)
		}
	}
	// Then, we need to find attestations that are slashable with respect to our database.
	for pubKey, signedAtts := range signedAttsByPubKey {
		for _, att := range signedAtts {
			indexedAtt := createAttestation(att.Source, att.Target)

			// If slashable == NotSlashable and err != nil, then CheckSlashableAttestation failed.
			// If slashable != NotSlashable, then err contains the reason why the attestation is slashable.
			slashable, err := validatorDB.CheckSlashableAttestation(ctx, pubKey, att.SigningRoot, indexedAtt)
			if err != nil && slashable == kv.NotSlashable {
				return nil, err
			}

			if slashable != kv.NotSlashable {
				slashablePubKeys = append(slashablePubKeys, pubKey)
				break
			}
		}
	}
	return slashablePubKeys, nil
}

func transformSignedBlocks(_ context.Context, signedBlocks []*format.SignedBlock) (*kv.ProposalHistoryForPubkey, error) {
	proposals := make([]kv.Proposal, len(signedBlocks))
	for i, proposal := range signedBlocks {
		slot, err := SlotFromString(proposal.Slot)
		if err != nil {
			return nil, fmt.Errorf("%s is not a valid slot: %w", proposal.Slot, err)
		}

		// Signing roots are optional in the standard JSON file.
		// If the signing root is not provided, we use a default value which is a zero-length byte slice.
		signingRoot := make([]byte, 0, fieldparams.RootLength)

		if proposal.SigningRoot != "" {
			signingRoot32, err := RootFromHex(proposal.SigningRoot)
			if err != nil {
				return nil, fmt.Errorf("%s is not a valid root: %w", proposal.SigningRoot, err)
			}
			signingRoot = signingRoot32[:]
		}

		proposals[i] = kv.Proposal{
			Slot:        slot,
			SigningRoot: signingRoot,
		}
	}

	return &kv.ProposalHistoryForPubkey{
		Proposals: proposals,
	}, nil
}

func transformSignedAttestations(pubKey [fieldparams.BLSPubkeyLength]byte, atts []*format.SignedAttestation) ([]*kv.AttestationRecord, error) {
	historicalAtts := make([]*kv.AttestationRecord, 0)
	for _, attestation := range atts {
		target, err := EpochFromString(attestation.TargetEpoch)
		if err != nil {
			return nil, fmt.Errorf("%s is not a valid epoch: %w", attestation.TargetEpoch, err)
		}
		source, err := EpochFromString(attestation.SourceEpoch)
		if err != nil {
			return nil, fmt.Errorf("%s is not a valid epoch: %w", attestation.SourceEpoch, err)
		}

		// Signing roots are optional in the standard JSON file.
		// If the signing root is not provided, we use a default value which is a zero-length byte slice.
		signingRoot := make([]byte, 0, fieldparams.RootLength)

		if attestation.SigningRoot != "" {
			signingRoot32, err := RootFromHex(attestation.SigningRoot)
			if err != nil {
				return nil, fmt.Errorf("%s is not a valid root: %w", attestation.SigningRoot, err)
			}
			signingRoot = signingRoot32[:]
		}
		historicalAtts = append(historicalAtts, &kv.AttestationRecord{
			PubKey:      pubKey,
			Source:      source,
			Target:      target,
			SigningRoot: signingRoot,
		})
	}
	return historicalAtts, nil
}

func createAttestation(source, target primitives.Epoch) *ethpb.IndexedAttestation {
	return &ethpb.IndexedAttestation{
		Data: &ethpb.AttestationData{
			Source: &ethpb.Checkpoint{
				Epoch: source,
			},
			Target: &ethpb.Checkpoint{
				Epoch: target,
			},
		},
	}
}
