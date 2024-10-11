package accounts

import (
	"archive/zip"
	"context"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"

	"github.com/logrusorgru/aurora"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
)

var (
	au = aurora.NewAurora(true)
)

const (
	allAccountsText = "All accounts"
	// ArchiveFilename specifies the name of the backup. Exported for tests.
	ArchiveFilename = "backup.zip"
)

// Backup allows users to select validator accounts from their wallet
// and export them as a backup.zip file containing the keys as EIP-2335 compliant
// keystore.json files, which are compatible with importing in other Ethereum consensus clients.
func (acm *CLIManager) Backup(ctx context.Context) error {
	keystoresToBackup, err := acm.keymanager.ExtractKeystores(ctx, acm.filteredPubKeys, acm.backupsPassword)
	if err != nil {
		return errors.Wrap(err, "could not extract keys from keymanager")
	}
	return zipKeystoresToOutputDir(keystoresToBackup, acm.backupsDir)
}

// Zips a list of keystore into respective EIP-2335 keystore.json files and
// writes their zipped format into the specified output directory.
func zipKeystoresToOutputDir(keystoresToBackup []*keymanager.Keystore, outputDir string) error {
	if len(keystoresToBackup) == 0 {
		return errors.New("nothing to backup")
	}
	if err := file.MkdirAll(outputDir); err != nil {
		return errors.Wrapf(err, "could not create directory at path: %s", outputDir)
	}
	// Marshal and zip all keystore files together and write the zip file
	// to the specified output directory.
	archivePath := filepath.Join(outputDir, ArchiveFilename)
	if file.Exists(archivePath) {
		return errors.Errorf("Zip file already exists in directory: %s", archivePath)
	}
	// We create a new file to store our backup.zip.
	zipfile, err := os.Create(filepath.Clean(archivePath))
	if err != nil {
		return errors.Wrapf(err, "could not create zip file with path: %s", archivePath)
	}
	defer func() {
		if err := zipfile.Close(); err != nil {
			log.WithError(err).Error("Could not close zipfile")
		}
	}()
	// Using this zip file, we create a new zip writer which we write
	// files to directly from our marshaled keystores.
	writer := zip.NewWriter(zipfile)
	defer func() {
		// We close the zip writer when done.
		if err := writer.Close(); err != nil {
			log.WithError(err).Error("Could not close zip file after writing")
		}
	}()
	for i, k := range keystoresToBackup {
		encodedFile, err := json.MarshalIndent(k, "", "\t")
		if err != nil {
			return errors.Wrap(err, "could not marshal keystore to JSON file")
		}
		f, err := writer.Create(fmt.Sprintf("keystore-%d.json", i))
		if err != nil {
			return errors.Wrap(err, "could not write keystore file to zip")
		}
		if _, err = f.Write(encodedFile); err != nil {
			return errors.Wrap(err, "could not write keystore file contents")
		}
	}
	log.WithField(
		"backup-path", archivePath,
	).Infof("Successfully backed up %d accounts", len(keystoresToBackup))
	return nil
}
