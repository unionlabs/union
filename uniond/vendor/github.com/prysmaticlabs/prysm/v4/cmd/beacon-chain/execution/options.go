package execution

import (
	"encoding/hex"
	"fmt"
	"strings"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/execution"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/flags"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	log "github.com/sirupsen/logrus"
	"github.com/urfave/cli/v2"
)

// FlagOptions for execution service flag configurations.
func FlagOptions(c *cli.Context) ([]execution.Option, error) {
	endpoint, err := parseExecutionChainEndpoint(c)
	if err != nil {
		return nil, err
	}
	jwtSecret, err := parseJWTSecretFromFile(c)
	if err != nil {
		return nil, errors.Wrap(err, "could not read JWT secret file for authenticating execution API")
	}
	headers := strings.Split(c.String(flags.ExecutionEngineHeaders.Name), ",")
	opts := []execution.Option{
		execution.WithHttpEndpoint(endpoint),
		execution.WithEth1HeaderRequestLimit(c.Uint64(flags.Eth1HeaderReqLimit.Name)),
		execution.WithHeaders(headers),
	}
	if len(jwtSecret) > 0 {
		opts = append(opts, execution.WithHttpEndpointAndJWTSecret(endpoint, jwtSecret))
	}
	return opts, nil
}

// Parses a JWT secret from a file path. This secret is required when connecting to execution nodes
// over HTTP, and must be the same one used in Prysm and the execution node server Prysm is connecting to.
// The engine API specification here https://github.com/ethereum/execution-apis/blob/main/src/engine/authentication.md
// Explains how we should validate this secret and the format of the file a user can specify.
//
// The secret must be stored as a hex-encoded string within a file in the filesystem.
// If the --jwt-secret flag is provided to Prysm, but the file cannot be read, or does not contain a hex-encoded
// key of at least 256 bits, the client should treat this as an error and abort the startup.
func parseJWTSecretFromFile(c *cli.Context) ([]byte, error) {
	jwtSecretFile := c.String(flags.ExecutionJWTSecretFlag.Name)
	if jwtSecretFile == "" {
		return nil, nil
	}
	enc, err := file.ReadFileAsBytes(jwtSecretFile)
	if err != nil {
		return nil, err
	}
	strData := strings.TrimSpace(string(enc))
	if len(strData) == 0 {
		return nil, fmt.Errorf("provided JWT secret in file %s cannot be empty", jwtSecretFile)
	}
	secret, err := hex.DecodeString(strings.TrimPrefix(strData, "0x"))
	if err != nil {
		return nil, err
	}
	if len(secret) < 32 {
		return nil, errors.New("provided JWT secret should be a hex string of at least 32 bytes")
	}
	log.Infof("Finished reading JWT secret from %s", jwtSecretFile)
	return secret, nil
}

func parseExecutionChainEndpoint(c *cli.Context) (string, error) {
	if c.String(flags.ExecutionEngineEndpoint.Name) == "" {
		return "", fmt.Errorf(
			"you need to specify %s to provide a connection endpoint to an Ethereum execution client "+
				"for your Prysm beacon node. This is a requirement for running a node. You can read more about "+
				"how to configure this execution client connection in our docs here "+
				"https://docs.prylabs.network/docs/install/install-with-script",
			flags.ExecutionEngineEndpoint.Name,
		)
	}
	return c.String(flags.ExecutionEngineEndpoint.Name), nil
}
