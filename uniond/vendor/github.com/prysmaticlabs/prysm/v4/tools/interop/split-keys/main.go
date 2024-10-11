// Package main provides a tool named split-keys which allows for generating any number of Ethereum validator keys
// from a list of BIP39 mnemonics and spreading them across any number of Prysm wallets. This is useful for creating
// custom allocations of keys across containers running in a cloud environment, such as for public testnets.
// An example of why you would use this tool is as follows. Let's say we have 1 mnemonic contained inside of a file.
// Then, we want to generate 10 keys from the mnemonic, and we want to spread them across 5 different wallets, each
// containing two keys. Then, you would run the tool as follows:
//
// ./main -mnemonics-file=/path/to/file.txt -keys-per-mnemonic=10 -num-wallets=5
//
// You can also specify the output directory for the wallet files using -out-dir and also the password
// used to encrypt the wallets in a text file using -wallet-password-file.
package main

import (
	"bufio"
	"context"
	"flag"
	"fmt"
	"log"
	"os"
	"path"

	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/wallet"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager/derived"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager/local"
	"github.com/tyler-smith/go-bip39"
	util "github.com/wealdtech/go-eth2-util"
)

var (
	mnemonicsFileFlag      = flag.String("mnemonics-file", "", "File containing mnemonics, one mnemonic per line")
	keysPerMnemonicFlag    = flag.Int("keys-per-mnemonic", 0, "The number of keys per mnemonic to generate")
	numberOfWalletsFlag    = flag.Int("num-wallets", 0, "Number of wallets to generate")
	walletOutDirFlag       = flag.String("out-dir", "", "Output directory for wallet files")
	walletPasswordFileFlag = flag.String("wallet-password-file", "", "File containing the password to encrypt all generated wallets")
)

// This application is run to generate keystores for testnets.
func main() {
	flag.Parse()
	f, err := os.Open(*mnemonicsFileFlag)
	if err != nil {
		log.Fatal(err)
	}
	defer func() {
		if err = f.Close(); err != nil {
			log.Fatal(err)
		}
	}()

	pubKeys, privKeys, err := generateKeysFromMnemonicList(bufio.NewScanner(f), *keysPerMnemonicFlag)
	if err != nil {
		log.Fatal(err)
	}

	log.Printf("Splitting %d keys across %d wallets\n", len(privKeys), *numberOfWalletsFlag)
	wPass, err := file.ReadFileAsBytes(*walletPasswordFileFlag)
	if err != nil {
		log.Fatal(err)
	}

	keysPerWallet := len(privKeys) / *numberOfWalletsFlag
	if err := spreadKeysAcrossLocalWallets(
		pubKeys,
		privKeys,
		*numberOfWalletsFlag,
		keysPerWallet,
		*walletOutDirFlag,
		string(wPass),
	); err != nil {
		log.Fatal(err)
	}
	log.Println("Done")
}

// Uses the provided mnemonic seed phrase to generate the
// appropriate seed file for recovering a derived wallets.
func seedFromMnemonic(mnemonic, mnemonicPassphrase string) ([]byte, error) {
	if ok := bip39.IsMnemonicValid(mnemonic); !ok {
		return nil, bip39.ErrInvalidMnemonic
	}
	return bip39.NewSeed(mnemonic, mnemonicPassphrase), nil
}

func generateKeysFromMnemonicList(mnemonicListFile *bufio.Scanner, keysPerMnemonic int) (pubKeys, privKeys [][]byte, err error) {
	pubKeys = make([][]byte, 0)
	privKeys = make([][]byte, 0)
	var seed []byte
	for mnemonicListFile.Scan() {
		log.Printf("Generating %d keys from mnemonic\n", keysPerMnemonic)
		mnemonic := mnemonicListFile.Text()
		seed, err = seedFromMnemonic(mnemonic, "" /* 25th word*/)
		if err != nil {
			return
		}
		for i := 0; i < keysPerMnemonic; i++ {
			if i%250 == 0 && i > 0 {
				log.Printf("%d/%d keys generated\n", i, keysPerMnemonic)
			}
			privKey, seedErr := util.PrivateKeyFromSeedAndPath(
				seed, fmt.Sprintf(derived.ValidatingKeyDerivationPathTemplate, i),
			)
			if seedErr != nil {
				err = seedErr
				return
			}
			privKeys = append(privKeys, privKey.Marshal())
			pubKeys = append(pubKeys, privKey.PublicKey().Marshal())
		}
	}
	return
}

func spreadKeysAcrossLocalWallets(
	pubKeys,
	privKeys [][]byte,
	numWallets,
	keysPerWallet int,
	walletOutputDir,
	walletPassword string,
) error {
	ctx := context.Background()
	for i := 0; i < numWallets; i++ {
		w := wallet.New(&wallet.Config{
			WalletDir:      path.Join(walletOutputDir, fmt.Sprintf("wallet_%d", i)),
			KeymanagerKind: keymanager.Local,
			WalletPassword: walletPassword,
		})
		km, err := local.NewKeymanager(ctx, &local.SetupConfig{
			Wallet: w,
		})
		if err != nil {
			return err
		}
		log.Printf("Importing %d keys into wallet %d\n", keysPerWallet, i)
		if err := km.ImportKeypairs(ctx, privKeys[i*keysPerWallet:(i+1)*keysPerWallet], pubKeys[i*keysPerWallet:(i+1)*keysPerWallet]); err != nil {
			return err
		}
	}
	return nil
}
