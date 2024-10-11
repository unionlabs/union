// Copyright 2018 The go-ethereum Authors
// This file is part of go-ethereum.
//
// go-ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// go-ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with go-ethereum. If not, see <http://www.gnu.org/licenses/>.

package main

import (
	"fmt"
	"os"
	"strings"

	"github.com/ethereum/go-ethereum/accounts/keystore"
	"github.com/ethereum/go-ethereum/cmd/utils"
	"github.com/urfave/cli/v2"
)

var newPassphraseFlag = &cli.StringFlag{
	Name:  "newpasswordfile",
	Usage: "the file that contains the new password for the keyfile",
}

var commandChangePassphrase = &cli.Command{
	Name:      "changepassword",
	Usage:     "change the password on a keyfile",
	ArgsUsage: "<keyfile>",
	Description: `
Change the password of a keyfile.`,
	Flags: []cli.Flag{
		passphraseFlag,
		newPassphraseFlag,
	},
	Action: func(ctx *cli.Context) error {
		keyfilepath := ctx.Args().First()

		// Read key from file.
		keyjson, err := os.ReadFile(keyfilepath)
		if err != nil {
			utils.Fatalf("Failed to read the keyfile at '%s': %v", keyfilepath, err)
		}

		// Decrypt key with passphrase.
		passphrase := getPassphrase(ctx, false)
		key, err := keystore.DecryptKey(keyjson, passphrase)
		if err != nil {
			utils.Fatalf("Error decrypting key: %v", err)
		}

		// Get a new passphrase.
		fmt.Println("Please provide a new password")
		var newPhrase string
		if passFile := ctx.String(newPassphraseFlag.Name); passFile != "" {
			content, err := os.ReadFile(passFile)
			if err != nil {
				utils.Fatalf("Failed to read new password file '%s': %v", passFile, err)
			}
			newPhrase = strings.TrimRight(string(content), "\r\n")
		} else {
			newPhrase = utils.GetPassPhrase("", true)
		}

		// Encrypt the key with the new passphrase.
		newJson, err := keystore.EncryptKey(key, newPhrase, keystore.StandardScryptN, keystore.StandardScryptP)
		if err != nil {
			utils.Fatalf("Error encrypting with new password: %v", err)
		}

		// Then write the new keyfile in place of the old one.
		if err := os.WriteFile(keyfilepath, newJson, 0600); err != nil {
			utils.Fatalf("Error writing new keyfile to disk: %v", err)
		}

		// Don't print anything.  Just return successfully,
		// producing a positive exit code.
		return nil
	},
}
