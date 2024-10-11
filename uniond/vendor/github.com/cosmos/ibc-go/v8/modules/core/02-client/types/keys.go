package types

import (
	"fmt"
	"regexp"
	"strconv"
	"strings"

	errorsmod "cosmossdk.io/errors"

	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

const (
	// SubModuleName defines the IBC client name
	SubModuleName string = "client"

	// RouterKey is the message route for IBC client
	RouterKey string = SubModuleName

	// QuerierRoute is the querier route for IBC client
	QuerierRoute string = SubModuleName

	// KeyNextClientSequence is the key used to store the next client sequence in
	// the keeper.
	KeyNextClientSequence = "nextClientSequence"

	// ParamsKey is the store key for the IBC client parameters
	ParamsKey = "clientParams"

	// AllowAllClients is the value that if set in AllowedClients param
	// would allow any wired up light client modules to be allowed
	AllowAllClients = "*"
)

// FormatClientIdentifier returns the client identifier with the sequence appended.
// This is a SDK specific format not enforced by IBC protocol.
func FormatClientIdentifier(clientType string, sequence uint64) string {
	return fmt.Sprintf("%s-%d", clientType, sequence)
}

// IsClientIDFormat checks if a clientID is in the format required on the SDK for
// parsing client identifiers. The client identifier must be in the form: `{client-type}-{N}
// which per the specification only permits ASCII for the {client-type} segment and
// 1 to 20 digits for the {N} segment.
// `([\w-]+\w)?` allows for a letter or hyphen, with the {client-type} starting with a letter
// and ending with a letter, i.e. `letter+(letter|hypen+letter)?`.
var IsClientIDFormat = regexp.MustCompile(`^\w+([\w-]+\w)?-[0-9]{1,20}$`).MatchString

// IsValidClientID checks if the clientID is valid and can be parsed into the client
// identifier format.
func IsValidClientID(clientID string) bool {
	_, _, err := ParseClientIdentifier(clientID)
	return err == nil
}

// ParseClientIdentifier parses the client type and sequence from the client identifier.
func ParseClientIdentifier(clientID string) (string, uint64, error) {
	if clientID == exported.LocalhostClientID {
		return clientID, 0, nil
	}

	if !IsClientIDFormat(clientID) {
		return "", 0, errorsmod.Wrapf(host.ErrInvalidID, "invalid client identifier %s is not in format: `{client-type}-{N}`", clientID)
	}

	splitStr := strings.Split(clientID, "-")
	lastIndex := len(splitStr) - 1

	clientType := strings.Join(splitStr[:lastIndex], "-")
	if strings.TrimSpace(clientType) == "" {
		return "", 0, errorsmod.Wrap(host.ErrInvalidID, "client identifier must be in format: `{client-type}-{N}` and client type cannot be blank")
	}

	sequence, err := strconv.ParseUint(splitStr[lastIndex], 10, 64)
	if err != nil {
		return "", 0, errorsmod.Wrap(err, "failed to parse client identifier sequence")
	}

	return clientType, sequence, nil
}
