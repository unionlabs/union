package local

import (
	"context"
	"encoding/hex"
	"fmt"
	"strings"

	"github.com/k0kubun/go-ansi"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	"github.com/schollz/progressbar/v3"
	"github.com/sirupsen/logrus"
	keystorev4 "github.com/wealdtech/go-eth2-wallet-encryptor-keystorev4"
)

// ImportKeystores into the local keymanager from an external source.
// 1) Copy the in memory keystore
// 2) Update copied keystore with new keys
// 3) Save the copy to disk
// 4) Reinitialize account store and updating the keymanager
// 5) Return Statuses
func (km *Keymanager) ImportKeystores(
	ctx context.Context,
	keystores []*keymanager.Keystore,
	passwords []string,
) ([]*keymanager.KeyStatus, error) {
	if len(passwords) == 0 {
		return nil, ErrNoPasswords
	}
	if len(passwords) != len(keystores) {
		return nil, ErrMismatchedNumPasswords
	}
	decryptor := keystorev4.New()
	bar := initializeProgressBar(len(keystores), "Importing accounts...")
	keys := map[string]string{}
	statuses := make([]*keymanager.KeyStatus, len(keystores))
	var err error
	// 1) Copy the in memory keystore
	storeCopy := km.accountsStore.Copy()
	importedKeys := make([][]byte, 0)
	existingPubKeys := make(map[string]bool)
	for i := 0; i < len(storeCopy.PrivateKeys); i++ {
		existingPubKeys[string(storeCopy.PublicKeys[i])] = true
	}
	for i := 0; i < len(keystores); i++ {
		var privKeyBytes []byte
		var pubKeyBytes []byte
		privKeyBytes, pubKeyBytes, _, err = km.attemptDecryptKeystore(decryptor, keystores[i], passwords[i])
		if err != nil {
			statuses[i] = &keymanager.KeyStatus{
				Status:  keymanager.StatusError,
				Message: err.Error(),
			}
			continue
		}
		if err := bar.Add(1); err != nil {
			log.Error(err)
		}
		// if key exists prior to being added then output log that duplicate key was found
		_, isDuplicateInArray := keys[string(pubKeyBytes)]
		_, isDuplicateInExisting := existingPubKeys[string(pubKeyBytes)]
		if isDuplicateInArray || isDuplicateInExisting {
			log.Warnf("Duplicate key in import will be ignored: %#x", pubKeyBytes)
			statuses[i] = &keymanager.KeyStatus{
				Status: keymanager.StatusDuplicate,
			}
			continue
		}

		keys[string(pubKeyBytes)] = string(privKeyBytes)
		importedKeys = append(importedKeys, pubKeyBytes)
		statuses[i] = &keymanager.KeyStatus{
			Status: keymanager.StatusImported,
		}
	}
	if len(importedKeys) == 0 {
		log.Warn("no keys were imported")
		return statuses, nil
	}
	// 2) Update copied keystore with new keys,clear duplicates in existing set
	// duplicates,errored ones are already skipped
	for pubKey, privKey := range keys {
		storeCopy.PublicKeys = append(storeCopy.PublicKeys, []byte(pubKey))
		storeCopy.PrivateKeys = append(storeCopy.PrivateKeys, []byte(privKey))
	}
	// 3) & 4) save to disk and re-initializes keystore
	if err := km.SaveStoreAndReInitialize(ctx, storeCopy); err != nil {
		return nil, err
	}

	log.WithFields(logrus.Fields{
		"publicKeys": CreatePrintoutOfKeys(importedKeys),
	}).Info("Successfully imported validator key(s)")

	// 5) Return Statuses
	return statuses, nil
}

// ImportKeypairs directly into the keymanager.
func (km *Keymanager) ImportKeypairs(ctx context.Context, privKeys, pubKeys [][]byte) error {
	if len(privKeys) != len(pubKeys) {
		return fmt.Errorf(
			"number of private keys and public keys is not equal: %d != %d", len(privKeys), len(pubKeys),
		)
	}
	// 1) Copy the in memory keystore
	storeCopy := km.accountsStore.Copy()

	// 2) Update store and remove duplicates
	updateAccountsStoreKeys(storeCopy, privKeys, pubKeys)

	// 3) & 4) save to disk and re-initializes keystore
	if err := km.SaveStoreAndReInitialize(ctx, storeCopy); err != nil {
		return err
	}

	// 5) verify if store was not updated
	if len(km.accountsStore.PublicKeys) < len(storeCopy.PublicKeys) {
		return fmt.Errorf("keys were not imported successfully, expected %d got %d", len(storeCopy.PublicKeys), len(km.accountsStore.PublicKeys))
	}
	return nil
}

// Retrieves the private key and public key from an EIP-2335 keystore file
// by decrypting using a specified password. If the password fails,
// it prompts the user for the correct password until it confirms.
func (*Keymanager) attemptDecryptKeystore(
	enc *keystorev4.Encryptor, keystore *keymanager.Keystore, password string,
) ([]byte, []byte, string, error) {
	// Attempt to decrypt the keystore with the specifies password.
	var privKeyBytes []byte
	var err error
	privKeyBytes, err = enc.Decrypt(keystore.Crypto, password)
	doesNotDecrypt := err != nil && strings.Contains(err.Error(), keymanager.IncorrectPasswordErrMsg)
	if doesNotDecrypt {
		return nil, nil, "", fmt.Errorf(
			"incorrect password for key 0x%s",
			keystore.Pubkey,
		)
	}
	if err != nil && !strings.Contains(err.Error(), keymanager.IncorrectPasswordErrMsg) {
		return nil, nil, "", errors.Wrap(err, "could not decrypt keystore")
	}
	var pubKeyBytes []byte
	// Attempt to use the pubkey present in the keystore itself as a field. If unavailable,
	// then utilize the public key directly from the private key.
	if keystore.Pubkey != "" {
		pubKeyBytes, err = hex.DecodeString(keystore.Pubkey)
		if err != nil {
			return nil, nil, "", errors.Wrap(err, "could not decode pubkey from keystore")
		}
	} else {
		privKey, err := bls.SecretKeyFromBytes(privKeyBytes)
		if err != nil {
			return nil, nil, "", errors.Wrap(err, "could not initialize private key from bytes")
		}
		pubKeyBytes = privKey.PublicKey().Marshal()
	}
	return privKeyBytes, pubKeyBytes, password, nil
}

func initializeProgressBar(numItems int, msg string) *progressbar.ProgressBar {
	return progressbar.NewOptions(
		numItems,
		progressbar.OptionFullWidth(),
		progressbar.OptionSetWriter(ansi.NewAnsiStdout()),
		progressbar.OptionEnableColorCodes(true),
		progressbar.OptionSetTheme(progressbar.Theme{
			Saucer:        "[green]=[reset]",
			SaucerHead:    "[green]>[reset]",
			SaucerPadding: " ",
			BarStart:      "[",
			BarEnd:        "]",
		}),
		progressbar.OptionOnCompletion(func() { fmt.Println() }),
		progressbar.OptionSetDescription(msg),
	)
}
