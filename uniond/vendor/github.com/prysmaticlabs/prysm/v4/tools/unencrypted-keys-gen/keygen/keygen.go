package keygen

import (
	"encoding/json"
	"fmt"
	"io"

	log "github.com/sirupsen/logrus"
)

// UnencryptedKeysContainer defines the structure of the unecrypted key JSON file.
type UnencryptedKeysContainer struct {
	Keys []*UnencryptedKeys `json:"keys"`
}

// UnencryptedKeys is the inner struct of the JSON file.
type UnencryptedKeys struct {
	ValidatorKey  []byte `json:"validator_key"`
	WithdrawalKey []byte `json:"withdrawal_key"`
}

// SaveUnencryptedKeysToFile JSON encodes the container and writes to the writer.
func SaveUnencryptedKeysToFile(w io.Writer, ctnr *UnencryptedKeysContainer) error {
	enc, err := json.Marshal(ctnr)
	if err != nil {
		log.Fatal(err)
	}
	n, err := w.Write(enc)
	if err != nil {
		return err
	}
	if n != len(enc) {
		return fmt.Errorf("failed to write %d bytes to file, wrote %d", len(enc), n)
	}
	return nil
}
