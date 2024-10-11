package jwt

import (
	"errors"
	"path/filepath"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/prysmaticlabs/prysm/v4/cmd"
	"github.com/prysmaticlabs/prysm/v4/crypto/rand"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/sirupsen/logrus"
	"github.com/urfave/cli/v2"
)

const (
	secretFileName = "jwt.hex"
)

var Commands = &cli.Command{
	Name:        "generate-auth-secret",
	Usage:       "creates a random, 32 byte hex string in a plaintext file to be used for authenticating JSON-RPC requests. If no --output-file flag is defined, the file will be created in the current working directory",
	Description: `creates a random, 32 byte hex string in a plaintext file to be used for authenticating JSON-RPC requests. If no --output-file flag is defined, the file will be created in the current working directory`,
	Flags: cmd.WrapFlags([]cli.Flag{
		cmd.JwtOutputFileFlag,
	}),
	Action: func(cliCtx *cli.Context) error {
		if err := generateAuthSecretInFile(cliCtx); err != nil {
			logrus.WithError(err).Fatal("Could not generate jwt")
		}
		return nil
	},
}

func generateAuthSecretInFile(c *cli.Context) error {
	fileName := secretFileName
	specifiedFilePath := c.String(cmd.JwtOutputFileFlag.Name)
	if len(specifiedFilePath) > 0 {
		fileName = specifiedFilePath
	}
	var err error
	fileName, err = file.ExpandPath(fileName)
	if err != nil {
		return err
	}
	fileDir := filepath.Dir(fileName)
	exists, err := file.HasDir(fileDir)
	if err != nil {
		return err
	}
	if !exists {
		if err := file.MkdirAll(fileDir); err != nil {
			return err
		}
	}
	secret, err := generateRandomHexString()
	if err != nil {
		return err
	}
	if err := file.WriteFile(fileName, []byte(secret)); err != nil {
		return err
	}
	logrus.Infof("Successfully wrote JSON-RPC authentication secret to file %s", fileName)
	return nil
}

func generateRandomHexString() (string, error) {
	secret := make([]byte, 32)
	randGen := rand.NewGenerator()
	n, err := randGen.Read(secret)
	if err != nil {
		return "", err
	} else if n <= 0 {
		return "", errors.New("rand: unexpected length")
	}
	return hexutil.Encode(secret), nil
}
