// Used for converting keys.yaml files from eth2.0-pm for interop testing.
// See: https://github.com/ethereum/eth2.0-pm/tree/master/interop/mocked_start
//
// This code can be discarded after interop testing.
package main

import (
	"encoding/hex"
	"fmt"
	"os"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/tools/unencrypted-keys-gen/keygen"
	log "github.com/sirupsen/logrus"
	"gopkg.in/yaml.v2"
)

// KeyPair with hex encoded data.
type KeyPair struct {
	Priv string `yaml:"privkey"`
	Pub  string `yaml:"pubkey"`
}

// KeyPairs represent the data format in the upstream yaml.
type KeyPairs []KeyPair

func main() {
	if len(os.Args) < 3 {
		fmt.Println("Usage: convert-keys path/to/keys.yaml path/to/output.json")
		return
	}
	inFile := os.Args[1]

	in, err := os.ReadFile(inFile) // #nosec G304
	if err != nil {
		log.WithError(err).Fatalf("Failed to read file %s", inFile)
	}
	data := make(KeyPairs, 0)
	if err := yaml.UnmarshalStrict(in, &data); err != nil {
		log.WithError(err).Fatal("Failed to unmarshal yaml")
	}

	out := &keygen.UnencryptedKeysContainer{}
	for _, key := range data {
		pk, err := hex.DecodeString(key.Priv[2:])
		if err != nil {
			log.WithError(err).Fatalf("Failed to decode hex string %s", key.Priv)
		}

		out.Keys = append(out.Keys, &keygen.UnencryptedKeys{
			ValidatorKey:  pk,
			WithdrawalKey: pk,
		})
	}

	outFile, err := os.OpenFile(os.Args[2], os.O_CREATE|os.O_EXCL, params.BeaconIoConfig().ReadWritePermissions)
	if err != nil {
		log.WithError(err).Fatalf("Failed to create file at %s", os.Args[2])
	}
	cleanup := func() {
		if err := outFile.Close(); err != nil {
			panic(err)
		}
	}
	defer cleanup()
	if err := keygen.SaveUnencryptedKeysToFile(outFile, out); err != nil {
		// log.Fatalf will prevent defer from being called
		cleanup()
		log.WithError(err).Fatal("Failed to save")
	}
	log.Printf("Wrote %s\n", os.Args[2])
}
