package main

import (
	_ "embed"
	"fmt"
	"io"
	"net/http"
	"os"
	"path"
	"path/filepath"
	"regexp"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/urfave/cli/v2"
)

const baseUrl = "https://raw.githubusercontent.com/ethereum/consensus-specs/dev"

// Regex to find Python's code snippets in markdown.
var reg2 = regexp.MustCompile(`(?msU)^\x60\x60\x60python\n+def\s(.*)^\x60\x60\x60`)

func download(cliCtx *cli.Context) error {
	fmt.Print("Downloading specs:\n")
	baseDir := cliCtx.String(dirFlag.Name)
	for dirName, fileNames := range specDirs {
		if err := prepareDir(path.Join(baseDir, dirName)); err != nil {
			return err
		}
		for _, fileName := range fileNames {
			outFilePath := path.Join(baseDir, dirName, fileName)
			specDocUrl := fmt.Sprintf("%s/%s", baseUrl, fmt.Sprintf("%s/%s", dirName, fileName))
			fmt.Printf("- %s\n", specDocUrl)
			if err := getAndSaveFile(specDocUrl, outFilePath); err != nil {
				return err
			}
		}
	}

	return nil
}

func getAndSaveFile(specDocUrl, outFilePath string) error {
	// Create output file.
	f, err := os.OpenFile(filepath.Clean(outFilePath), os.O_CREATE|os.O_EXCL, params.BeaconIoConfig().ReadWritePermissions)
	if err != nil {
		return fmt.Errorf("cannot create output file: %w", err)
	}
	defer func() {
		if err := f.Close(); err != nil {
			fmt.Printf("cannot close output file: %v", err)
		}
	}()

	// Download spec doc.
	resp, err := http.Get(specDocUrl) // #nosec G107 -- False positive
	if err != nil {
		return err
	}
	defer func() {
		if err := resp.Body.Close(); err != nil {
			fmt.Printf("cannot close spec doc file: %v", err)
		}
	}()

	// Transform and save spec docs.
	specDoc, err := io.ReadAll(resp.Body)
	if err != nil {
		return err
	}
	specDocString := string(specDoc)
	for _, snippet := range reg2.FindAllString(specDocString, -1) {
		if _, err = f.WriteString(snippet + "\n"); err != nil {
			return err
		}
	}

	return nil
}

func prepareDir(dirPath string) error {
	return os.MkdirAll(dirPath, os.ModePerm)
}
