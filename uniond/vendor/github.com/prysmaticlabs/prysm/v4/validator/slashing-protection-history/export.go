package history

import (
	"context"
	"fmt"
	"sort"
	"strings"

	"github.com/pkg/errors"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/monitoring/progress"
	"github.com/prysmaticlabs/prysm/v4/validator/db"
	"github.com/prysmaticlabs/prysm/v4/validator/slashing-protection-history/format"
)

// ExportStandardProtectionJSON extracts all slashing protection data from a validator database
// and packages it into an EIP-3076 compliant, standard
func ExportStandardProtectionJSON(
	ctx context.Context,
	validatorDB db.Database,
	filteredKeys ...[]byte,
) (*format.EIPSlashingProtectionFormat, error) {
	interchangeJSON := &format.EIPSlashingProtectionFormat{}
	genesisValidatorsRoot, err := validatorDB.GenesisValidatorsRoot(ctx)
	if err != nil {
		return nil, errors.Wrap(err, "could not get genesis validators root from DB")
	}
	if genesisValidatorsRoot == nil || !bytesutil.IsValidRoot(genesisValidatorsRoot) {
		return nil, errors.New(
			"genesis validators root is empty, perhaps you are not connected to your beacon node",
		)
	}
	genesisRootHex, err := rootToHexString(genesisValidatorsRoot)
	if err != nil {
		return nil, errors.Wrap(err, "could not convert genesis validators root to hex string")
	}
	interchangeJSON.Metadata.GenesisValidatorsRoot = genesisRootHex
	interchangeJSON.Metadata.InterchangeFormatVersion = format.InterchangeFormatVersion

	// Allow for filtering data for the keys we wish to export.
	filteredKeysMap := make(map[string]bool, len(filteredKeys))
	for _, k := range filteredKeys {
		filteredKeysMap[string(k)] = true
	}

	// Extract the existing public keys in our database.
	proposedPublicKeys, err := validatorDB.ProposedPublicKeys(ctx)
	if err != nil {
		return nil, errors.Wrap(err, "could not retrieve proposer public keys from DB")
	}
	attestedPublicKeys, err := validatorDB.AttestedPublicKeys(ctx)
	if err != nil {
		return nil, errors.Wrap(err, "could not retrieve attested public keys from DB")
	}
	dataByPubKey := make(map[[fieldparams.BLSPubkeyLength]byte]*format.ProtectionData)

	// Extract the signed proposals by public key.
	bar := progress.InitializeProgressBar(
		len(proposedPublicKeys), "Extracting signed blocks by validator public key",
	)
	for _, pubKey := range proposedPublicKeys {
		if _, ok := filteredKeysMap[string(pubKey[:])]; len(filteredKeys) > 0 && !ok {
			continue
		}
		pubKeyHex, err := pubKeyToHexString(pubKey[:])
		if err != nil {
			return nil, errors.Wrap(err, "could not convert public key to hex string")
		}
		signedBlocks, err := signedBlocksByPubKey(ctx, validatorDB, pubKey)
		if err != nil {
			return nil, errors.Wrapf(err, "could not retrieve signed blocks for public key %s", pubKeyHex)
		}
		dataByPubKey[pubKey] = &format.ProtectionData{
			Pubkey:             pubKeyHex,
			SignedBlocks:       signedBlocks,
			SignedAttestations: nil,
		}
		if err := bar.Add(1); err != nil {
			return nil, err
		}
	}

	// Extract the signed attestations by public key.
	bar = progress.InitializeProgressBar(
		len(attestedPublicKeys), "Extracting signed attestations by validator public key",
	)
	for _, pubKey := range attestedPublicKeys {
		if _, ok := filteredKeysMap[string(pubKey[:])]; len(filteredKeys) > 0 && !ok {
			continue
		}
		pubKeyHex, err := pubKeyToHexString(pubKey[:])
		if err != nil {
			return nil, errors.Wrap(err, "could not convert public key to hex string")
		}
		signedAttestations, err := signedAttestationsByPubKey(ctx, validatorDB, pubKey)
		if err != nil {
			return nil, errors.Wrapf(err, "could not retrieve signed attestations for public key %s", pubKeyHex)
		}
		if _, ok := dataByPubKey[pubKey]; ok {
			dataByPubKey[pubKey].SignedAttestations = signedAttestations
		} else {
			dataByPubKey[pubKey] = &format.ProtectionData{
				Pubkey:             pubKeyHex,
				SignedBlocks:       nil,
				SignedAttestations: signedAttestations,
			}
		}
		if err := bar.Add(1); err != nil {
			return nil, err
		}
	}

	// Next we turn our map into a slice as expected by the EIP-3076 JSON standard.
	dataList := make([]*format.ProtectionData, 0)
	for _, item := range dataByPubKey {
		if item.SignedAttestations == nil {
			item.SignedAttestations = make([]*format.SignedAttestation, 0)
		}
		if item.SignedBlocks == nil {
			item.SignedBlocks = make([]*format.SignedBlock, 0)
		}
		dataList = append(dataList, item)
	}
	sort.Slice(dataList, func(i, j int) bool {
		return strings.Compare(dataList[i].Pubkey, dataList[j].Pubkey) < 0
	})
	interchangeJSON.Data = dataList
	return interchangeJSON, nil
}

func signedAttestationsByPubKey(ctx context.Context, validatorDB db.Database, pubKey [fieldparams.BLSPubkeyLength]byte) ([]*format.SignedAttestation, error) {
	// If a key does not have an attestation history in our database, we return nil.
	// This way, a user will be able to export their slashing protection history
	// even if one of their keys does not have a history of signed attestations.
	history, err := validatorDB.AttestationHistoryForPubKey(ctx, pubKey)
	if err != nil {
		return nil, errors.Wrap(err, "cannot get attestation history for public key")
	}
	if history == nil {
		return nil, nil
	}
	signedAttestations := make([]*format.SignedAttestation, 0)
	for i := 0; i < len(history); i++ {
		att := history[i]
		// Special edge case due to a bug in Prysm's old slashing protection schema. The bug
		// manifests itself as the first entry in attester slashing protection history
		// having a target epoch greater than the next entry in the list. If this manifests,
		// we skip it to protect users. This check is the best trade-off we can make at
		// the moment without creating any false positive slashable attestation exports.
		// More information on the bug can found in https://github.com/prysmaticlabs/prysm/issues/8893.
		if i == 0 && len(history) > 1 {
			nextEntryTargetEpoch := history[1].Target
			if att.Target > nextEntryTargetEpoch && att.Source == 0 {
				continue
			}
		}
		var root string
		if len(att.SigningRoot) != 0 {
			root, err = rootToHexString(att.SigningRoot)
			if err != nil {
				return nil, errors.Wrap(err, "could not convert signing root to hex string")
			}
		}
		signedAttestations = append(signedAttestations, &format.SignedAttestation{
			TargetEpoch: fmt.Sprintf("%d", att.Target),
			SourceEpoch: fmt.Sprintf("%d", att.Source),
			SigningRoot: root,
		})
	}
	return signedAttestations, nil
}

func signedBlocksByPubKey(ctx context.Context, validatorDB db.Database, pubKey [fieldparams.BLSPubkeyLength]byte) ([]*format.SignedBlock, error) {
	// If a key does not have a lowest or highest signed proposal history
	// in our database, we return nil. This way, a user will be able to export their
	// slashing protection history even if one of their keys does not have a history
	// of signed blocks.
	proposalHistory, err := validatorDB.ProposalHistoryForPubKey(ctx, pubKey)
	if err != nil {
		return nil, errors.Wrapf(err, "could not get proposal history for public key: %#x", pubKey)
	}
	signedBlocks := make([]*format.SignedBlock, 0)
	for _, proposal := range proposalHistory {
		if ctx.Err() != nil {
			return nil, errors.Wrap(err, "context canceled")
		}
		signingRootHex, err := rootToHexString(proposal.SigningRoot)
		if err != nil {
			return nil, errors.Wrap(err, "could not convert signing root to hex string")
		}
		signedBlocks = append(signedBlocks, &format.SignedBlock{
			Slot:        fmt.Sprintf("%d", proposal.Slot),
			SigningRoot: signingRootHex,
		})
	}
	return signedBlocks, nil
}
