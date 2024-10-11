package kzg

import (
	_ "embed"
	"encoding/json"

	GoKZG "github.com/crate-crypto/go-kzg-4844"
	"github.com/pkg/errors"
)

var (
	//go:embed trusted_setup.json
	embeddedTrustedSetup []byte // 1.2Mb
	kzgContext           *GoKZG.Context
)

func Start() error {
	parsedSetup := GoKZG.JSONTrustedSetup{}
	err := json.Unmarshal(embeddedTrustedSetup, &parsedSetup)
	if err != nil {
		return errors.Wrap(err, "could not parse trusted setup JSON")
	}
	kzgContext, err = GoKZG.NewContext4096(&parsedSetup)
	if err != nil {
		return errors.Wrap(err, "could not initialize go-kzg context")
	}
	return nil
}
