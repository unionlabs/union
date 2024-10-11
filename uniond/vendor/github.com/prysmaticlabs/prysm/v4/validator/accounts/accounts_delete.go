package accounts

import (
	"context"
	"fmt"
	"strings"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/io/prompt"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
)

// Delete the accounts that the user requests to be deleted from the wallet.
func (acm *CLIManager) Delete(ctx context.Context) error {
	rawPublicKeys := make([][]byte, len(acm.filteredPubKeys))
	formattedPubKeys := make([]string, len(acm.filteredPubKeys))
	for i, pk := range acm.filteredPubKeys {
		pubKeyBytes := pk.Marshal()
		rawPublicKeys[i] = pubKeyBytes
		formattedPubKeys[i] = fmt.Sprintf("%#x", bytesutil.Trunc(pubKeyBytes))
	}
	allAccountStr := strings.Join(formattedPubKeys, ", ")
	if !acm.deletePublicKeys {
		if len(acm.filteredPubKeys) == 1 {
			promptText := "Are you sure you want to delete 1 account? (%s) Y/N"
			resp, err := prompt.ValidatePrompt(
				acm.inputReader, fmt.Sprintf(promptText, au.BrightGreen(formattedPubKeys[0])), prompt.ValidateYesOrNo,
			)
			if err != nil {
				return err
			}
			if strings.EqualFold(resp, "n") {
				return nil
			}
		} else {
			promptText := "Are you sure you want to delete %d accounts? (%s) Y/N"
			if len(acm.filteredPubKeys) == acm.walletKeyCount {
				promptText = fmt.Sprintf("Are you sure you want to delete all accounts? Y/N (%s)", au.BrightGreen(allAccountStr))
			} else {
				promptText = fmt.Sprintf(promptText, len(acm.filteredPubKeys), au.BrightGreen(allAccountStr))
			}
			resp, err := prompt.ValidatePrompt(acm.inputReader, promptText, prompt.ValidateYesOrNo)
			if err != nil {
				return err
			}
			if strings.EqualFold(resp, "n") {
				return nil
			}
		}
	}
	if err := DeleteAccount(ctx, &DeleteConfig{
		Keymanager:       acm.keymanager,
		DeletePublicKeys: rawPublicKeys,
	}); err != nil {
		return err
	}
	log.WithField("publicKeys", allAccountStr).Warn(
		"Attempted to delete accounts. IMPORTANT: please run `validator accounts list` to ensure " +
			"the public keys are indeed deleted. If they are still there, please file an issue at " +
			"https://github.com/prysmaticlabs/prysm/issues/new")
	return nil
}

// DeleteAccount performs the deletion on the Keymanager.
func DeleteAccount(ctx context.Context, cfg *DeleteConfig) error {
	if len(cfg.DeletePublicKeys) == 1 {
		log.Info("Deleting account...")
	} else {
		log.Info("Deleting accounts...")
	}
	statuses, err := cfg.Keymanager.DeleteKeystores(ctx, cfg.DeletePublicKeys)
	if err != nil {
		return errors.Wrap(err, "could not delete accounts")
	}
	for i, status := range statuses {
		switch status.Status {
		case keymanager.StatusError:
			log.Errorf("Error deleting key %#x: %s", bytesutil.Trunc(cfg.DeletePublicKeys[i]), status.Message)
		case keymanager.StatusNotActive:
			log.Warnf("Duplicate key %#x found in delete request", bytesutil.Trunc(cfg.DeletePublicKeys[i]))
		case keymanager.StatusNotFound:
			log.Warnf("Could not find keystore for %#x", bytesutil.Trunc(cfg.DeletePublicKeys[i]))
		}
	}
	return nil
}
