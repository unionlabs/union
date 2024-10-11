// This tool allows for simple encrypting and decrypting of EIP-2335 compliant, BLS12-381
// keystore.json files which as password protected. This is helpful in development to inspect
// the contents of keystores created by Ethereum validator wallets or to easily produce keystores from a
// specified secret to move them around in a standard format between Ethereum consensus clients.
package main

import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"strings"

	"github.com/google/uuid"
	"github.com/logrusorgru/aurora"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/prysmaticlabs/prysm/v4/io/prompt"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	"github.com/urfave/cli/v2"
	keystorev4 "github.com/wealdtech/go-eth2-wallet-encryptor-keystorev4"
)

var (
	keystoresFlag = &cli.StringFlag{
		Name:     "keystores",
		Value:    "",
		Usage:    "Path to a file or directory containing keystore files",
		Required: true,
	}
	passwordFlag = &cli.StringFlag{
		Name:  "password",
		Value: "",
		Usage: "Password for the keystore(s)",
	}
	privateKeyFlag = &cli.StringFlag{
		Name:     "private-key",
		Value:    "",
		Usage:    "Hex string for the BLS12-381 private key you wish encrypt into a keystore file",
		Required: true,
	}
	outputPathFlag = &cli.StringFlag{
		Name:     "output-path",
		Value:    "",
		Usage:    "Output path to write the newly encrypted keystore file",
		Required: true,
	}
	au = aurora.NewAurora(true /* enable colors */)
)

func main() {
	app := &cli.App{
		Name:        "Keystore utility",
		Description: "Utility to encrypt and decrypt EIP-2335 compliant keystore.json files for BLS12-381 private keys",
		Usage:       "",
		Commands: []*cli.Command{
			{
				Name:  "decrypt",
				Usage: "decrypt a specified keystore file or directory containing keystore files",
				Flags: []cli.Flag{
					keystoresFlag,
					passwordFlag,
				},
				Action: decrypt,
			},
			{
				Name:  "encrypt",
				Usage: "encrypt a specified hex value of a BLS12-381 private key into a keystore file",
				Flags: []cli.Flag{
					passwordFlag,
					privateKeyFlag,
					outputPathFlag,
				},
				Action: encrypt,
			},
		},
	}
	err := app.Run(os.Args)
	if err != nil {
		log.Fatal(err)
	}
}

func decrypt(cliCtx *cli.Context) error {
	keystorePath := cliCtx.String(keystoresFlag.Name)
	if keystorePath == "" {
		return errors.New("--keystore must be set")
	}
	fullPath, err := file.ExpandPath(keystorePath)
	if err != nil {
		return errors.Wrapf(err, "could not expand path: %s", keystorePath)
	}
	password := cliCtx.String(passwordFlag.Name)
	isPasswordSet := cliCtx.IsSet(passwordFlag.Name)
	if !isPasswordSet {
		password, err = prompt.PasswordPrompt("Input the keystore(s) password", func(s string) error {
			// Any password is valid.
			return nil
		})
		if err != nil {
			return err
		}
	}
	isDir, err := file.HasDir(fullPath)
	if err != nil {
		return errors.Wrapf(err, "could not check if path exists: %s", fullPath)
	}
	if isDir {
		files, err := os.ReadDir(fullPath)
		if err != nil {
			return errors.Wrapf(err, "could not read directory: %s", fullPath)
		}
		for _, f := range files {
			if f.IsDir() {
				continue
			}
			keystorePath := filepath.Join(fullPath, f.Name())
			if err := readAndDecryptKeystore(keystorePath, password); err != nil {
				fmt.Printf("could not read nor decrypt keystore at path %s: %v\n", keystorePath, err)
			}
		}
		return nil
	}
	return readAndDecryptKeystore(fullPath, password)
}

// Attempts to encrypt a passed-in BLS12-3381 private key into the EIP-2335
// keystore.json format. If a file at the specified output path exists, asks the user
// to confirm overwriting its contents. If the value passed in is not a valid BLS12-381
// private key, the function will fail.
func encrypt(cliCtx *cli.Context) error {
	var err error
	password := cliCtx.String(passwordFlag.Name)
	isPasswordSet := cliCtx.IsSet(passwordFlag.Name)
	if !isPasswordSet {
		password, err = prompt.PasswordPrompt("Input the keystore(s) password", func(s string) error {
			// Any password is valid.
			return nil
		})
		if err != nil {
			return err
		}
	}
	privateKeyString := cliCtx.String(privateKeyFlag.Name)
	if privateKeyString == "" {
		return errors.New("--private-key must not be empty")
	}
	outputPath := cliCtx.String(outputPathFlag.Name)
	if outputPath == "" {
		return errors.New("--output-path must be set")
	}
	fullPath, err := file.ExpandPath(outputPath)
	if err != nil {
		return errors.Wrapf(err, "could not expand path: %s", outputPath)
	}
	if file.Exists(fullPath) {
		response, err := prompt.ValidatePrompt(
			os.Stdin,
			fmt.Sprintf("file at path %s already exists, are you sure you want to overwrite it? [y/n]", fullPath),
			func(s string) error {
				input := strings.ToLower(s)
				if input != "y" && input != "n" {
					return errors.New("please confirm the above text")
				}
				return nil
			},
		)
		if err != nil {
			return errors.Wrap(err, "could not validate userprompt confirmation")
		}
		if response == "n" {
			return nil
		}
	}
	if len(privateKeyString) > 2 && strings.Contains(privateKeyString, "0x") {
		privateKeyString = privateKeyString[2:] // Strip the 0x prefix, if any.
	}
	bytesValue, err := hex.DecodeString(privateKeyString)
	if err != nil {
		return errors.Wrapf(err, "could not decode as hex string: %s", privateKeyString)
	}
	privKey, err := bls.SecretKeyFromBytes(bytesValue)
	if err != nil {
		return errors.Wrap(err, "not a valid BLS12-381 private key")
	}
	pubKey := fmt.Sprintf("%x", privKey.PublicKey().Marshal())
	encryptor := keystorev4.New()
	id, err := uuid.NewRandom()
	if err != nil {
		return errors.Wrap(err, "could not generate new random uuid")
	}
	cryptoFields, err := encryptor.Encrypt(bytesValue, password)
	if err != nil {
		return errors.Wrap(err, "could not encrypt into new keystore")
	}
	item := &keymanager.Keystore{
		Crypto:      cryptoFields,
		ID:          id.String(),
		Version:     encryptor.Version(),
		Pubkey:      pubKey,
		Description: encryptor.Name(),
	}
	encodedFile, err := json.MarshalIndent(item, "", "\t")
	if err != nil {
		return errors.Wrap(err, "could not json marshal keystore")
	}
	if err := file.WriteFile(fullPath, encodedFile); err != nil {
		return errors.Wrapf(err, "could not write file at path: %s", fullPath)
	}
	fmt.Printf(
		"\nWrote encrypted keystore file at path %s\n",
		au.BrightMagenta(fullPath),
	)
	fmt.Printf("Pubkey: %s\n", au.BrightGreen(
		fmt.Sprintf("%#x", privKey.PublicKey().Marshal()),
	))
	return nil
}

// Reads the keystore file at the provided path and attempts
// to decrypt it with the specified passwords.
func readAndDecryptKeystore(fullPath, password string) error {
	f, err := os.ReadFile(fullPath) // #nosec G304
	if err != nil {
		return errors.Wrapf(err, "could not read file at path: %s", fullPath)
	}
	decryptor := keystorev4.New()
	keystoreFile := &keymanager.Keystore{}
	if err := json.Unmarshal(f, keystoreFile); err != nil {
		return errors.Wrap(err, "could not JSON unmarshal keystore file")
	}
	// We extract the validator signing private key from the keystore
	// by utilizing the password.
	privKeyBytes, err := decryptor.Decrypt(keystoreFile.Crypto, password)
	if err != nil {
		if strings.Contains(err.Error(), "invalid checksum") {
			return fmt.Errorf("incorrect password for keystore at path: %s", fullPath)
		}
		return err
	}

	var pubKeyBytes []byte
	// Attempt to use the pubkey present in the keystore itself as a field. If unavailable,
	// then utilize the public key directly from the private key.
	if keystoreFile.Pubkey != "" {
		pubKeyBytes, err = hex.DecodeString(keystoreFile.Pubkey)
		if err != nil {
			return errors.Wrap(err, "could not decode pubkey from keystore")
		}
	} else {
		privKey, err := bls.SecretKeyFromBytes(privKeyBytes)
		if err != nil {
			return errors.Wrap(err, "could not initialize private key from bytes")
		}
		pubKeyBytes = privKey.PublicKey().Marshal()
	}
	fmt.Printf("\nDecrypted keystore %s\n", au.BrightMagenta(fullPath))
	fmt.Printf("Privkey: %#x\n", au.BrightGreen(privKeyBytes))
	fmt.Printf("Pubkey: %#x\n", au.BrightGreen(pubKeyBytes))
	return nil
}
