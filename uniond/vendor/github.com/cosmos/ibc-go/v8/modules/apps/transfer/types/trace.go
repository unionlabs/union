package types

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"sort"
	"strings"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"

	tmbytes "github.com/cometbft/cometbft/libs/bytes"
	tmtypes "github.com/cometbft/cometbft/types"

	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
)

// ParseDenomTrace parses a string with the ibc prefix (denom trace) and the base denomination
// into a DenomTrace type.
//
// Examples:
//
// - "portidone/channel-0/uatom" => DenomTrace{Path: "portidone/channel-0", BaseDenom: "uatom"}
// - "portidone/channel-0/portidtwo/channel-1/uatom" => DenomTrace{Path: "portidone/channel-0/portidtwo/channel-1", BaseDenom: "uatom"}
// - "portidone/channel-0/gamm/pool/1" => DenomTrace{Path: "portidone/channel-0", BaseDenom: "gamm/pool/1"}
// - "gamm/pool/1" => DenomTrace{Path: "", BaseDenom: "gamm/pool/1"}
// - "uatom" => DenomTrace{Path: "", BaseDenom: "uatom"}
func ParseDenomTrace(rawDenom string) DenomTrace {
	denomSplit := strings.Split(rawDenom, "/")

	if denomSplit[0] == rawDenom {
		return DenomTrace{
			Path:      "",
			BaseDenom: rawDenom,
		}
	}

	path, baseDenom := extractPathAndBaseFromFullDenom(denomSplit)
	return DenomTrace{
		Path:      path,
		BaseDenom: baseDenom,
	}
}

// Hash returns the hex bytes of the SHA256 hash of the DenomTrace fields using the following formula:
//
// hash = sha256(tracePath + "/" + baseDenom)
func (dt DenomTrace) Hash() tmbytes.HexBytes {
	hash := sha256.Sum256([]byte(dt.GetFullDenomPath()))
	return hash[:]
}

// GetPrefix returns the receiving denomination prefix composed by the trace info and a separator.
func (dt DenomTrace) GetPrefix() string {
	return dt.Path + "/"
}

// IBCDenom a coin denomination for an ICS20 fungible token in the format
// 'ibc/{hash(tracePath + baseDenom)}'. If the trace is empty, it will return the base denomination.
func (dt DenomTrace) IBCDenom() string {
	if dt.Path != "" {
		return fmt.Sprintf("%s/%s", DenomPrefix, dt.Hash())
	}
	return dt.BaseDenom
}

// GetFullDenomPath returns the full denomination according to the ICS20 specification:
// tracePath + "/" + baseDenom
// If there exists no trace then the base denomination is returned.
func (dt DenomTrace) GetFullDenomPath() string {
	if dt.Path == "" {
		return dt.BaseDenom
	}
	return dt.GetPrefix() + dt.BaseDenom
}

// IsNativeDenom returns true if the denomination is native, thus containing no trace history.
func (dt DenomTrace) IsNativeDenom() bool {
	return dt.Path == ""
}

// extractPathAndBaseFromFullDenom returns the trace path and the base denom from
// the elements that constitute the complete denom.
func extractPathAndBaseFromFullDenom(fullDenomItems []string) (string, string) {
	var (
		pathSlice      []string
		baseDenomSlice []string
	)

	length := len(fullDenomItems)
	for i := 0; i < length; i += 2 {
		// The IBC specification does not guarantee the expected format of the
		// destination port or destination channel identifier. A short term solution
		// to determine base denomination is to expect the channel identifier to be the
		// one ibc-go specifies. A longer term solution is to separate the path and base
		// denomination in the ICS20 packet. If an intermediate hop prefixes the full denom
		// with a channel identifier format different from our own, the base denomination
		// will be incorrectly parsed, but the token will continue to be treated correctly
		// as an IBC denomination. The hash used to store the token internally on our chain
		// will be the same value as the base denomination being correctly parsed.
		if i < length-1 && length > 2 && channeltypes.IsValidChannelID(fullDenomItems[i+1]) {
			pathSlice = append(pathSlice, fullDenomItems[i], fullDenomItems[i+1])
		} else {
			baseDenomSlice = fullDenomItems[i:]
			break
		}
	}

	path := strings.Join(pathSlice, "/")
	baseDenom := strings.Join(baseDenomSlice, "/")

	return path, baseDenom
}

func validateTraceIdentifiers(identifiers []string) error {
	if len(identifiers) == 0 || len(identifiers)%2 != 0 {
		return fmt.Errorf("trace info must come in pairs of port and channel identifiers '{portID}/{channelID}', got the identifiers: %s", identifiers)
	}

	// validate correctness of port and channel identifiers
	for i := 0; i < len(identifiers); i += 2 {
		if err := host.PortIdentifierValidator(identifiers[i]); err != nil {
			return errorsmod.Wrapf(err, "invalid port ID at position %d", i)
		}
		if err := host.ChannelIdentifierValidator(identifiers[i+1]); err != nil {
			return errorsmod.Wrapf(err, "invalid channel ID at position %d", i)
		}
	}
	return nil
}

// Validate performs a basic validation of the DenomTrace fields.
func (dt DenomTrace) Validate() error {
	// empty trace is accepted when token lives on the original chain
	switch {
	case dt.Path == "" && dt.BaseDenom != "":
		return nil
	case strings.TrimSpace(dt.BaseDenom) == "":
		return fmt.Errorf("base denomination cannot be blank")
	}

	// NOTE: no base denomination validation

	identifiers := strings.Split(dt.Path, "/")
	return validateTraceIdentifiers(identifiers)
}

// Traces defines a wrapper type for a slice of DenomTrace.
type Traces []DenomTrace

// Validate performs a basic validation of each denomination trace info.
func (t Traces) Validate() error {
	seenTraces := make(map[string]bool)
	for i, trace := range t {
		hash := trace.Hash().String()
		if seenTraces[hash] {
			return fmt.Errorf("duplicated denomination trace with hash %s", trace.Hash())
		}

		if err := trace.Validate(); err != nil {
			return errorsmod.Wrapf(err, "failed denom trace %d validation", i)
		}
		seenTraces[hash] = true
	}
	return nil
}

var _ sort.Interface = (*Traces)(nil)

// Len implements sort.Interface for Traces
func (t Traces) Len() int { return len(t) }

// Less implements sort.Interface for Traces
func (t Traces) Less(i, j int) bool { return t[i].GetFullDenomPath() < t[j].GetFullDenomPath() }

// Swap implements sort.Interface for Traces
func (t Traces) Swap(i, j int) { t[i], t[j] = t[j], t[i] }

// Sort is a helper function to sort the set of denomination traces in-place
func (t Traces) Sort() Traces {
	sort.Sort(t)
	return t
}

// ValidatePrefixedDenom checks that the denomination for an IBC fungible token packet denom is correctly prefixed.
// The function will return no error if the given string follows one of the two formats:
//
//   - Prefixed denomination: '{portIDN}/{channelIDN}/.../{portID0}/{channelID0}/baseDenom'
//   - Unprefixed denomination: 'baseDenom'
//
// 'baseDenom' may or may not contain '/'s
func ValidatePrefixedDenom(denom string) error {
	denomSplit := strings.Split(denom, "/")
	if denomSplit[0] == denom && strings.TrimSpace(denom) != "" {
		// NOTE: no base denomination validation
		return nil
	}

	if strings.TrimSpace(denomSplit[len(denomSplit)-1]) == "" {
		return errorsmod.Wrap(ErrInvalidDenomForTransfer, "base denomination cannot be blank")
	}

	path, _ := extractPathAndBaseFromFullDenom(denomSplit)
	if path == "" {
		// NOTE: base denom contains slashes, so no base denomination validation
		return nil
	}

	identifiers := strings.Split(path, "/")
	return validateTraceIdentifiers(identifiers)
}

// ValidateIBCDenom validates that the given denomination is either:
//
//   - A valid base denomination (eg: 'uatom' or 'gamm/pool/1' as in https://github.com/cosmos/ibc-go/issues/894)
//   - A valid fungible token representation (i.e 'ibc/{hash}') per ADR 001 https://github.com/cosmos/ibc-go/blob/main/docs/architecture/adr-001-coin-source-tracing.md
func ValidateIBCDenom(denom string) error {
	if err := sdk.ValidateDenom(denom); err != nil {
		return err
	}

	denomSplit := strings.SplitN(denom, "/", 2)

	switch {
	case denom == DenomPrefix:
		return errorsmod.Wrapf(ErrInvalidDenomForTransfer, "denomination should be prefixed with the format 'ibc/{hash(trace + \"/\" + %s)}'", denom)

	case len(denomSplit) == 2 && denomSplit[0] == DenomPrefix:
		if strings.TrimSpace(denomSplit[1]) == "" {
			return errorsmod.Wrapf(ErrInvalidDenomForTransfer, "denomination should be prefixed with the format 'ibc/{hash(trace + \"/\" + %s)}'", denom)
		}

		if _, err := ParseHexHash(denomSplit[1]); err != nil {
			return errorsmod.Wrapf(err, "invalid denom trace hash %s", denomSplit[1])
		}
	}

	return nil
}

// ParseHexHash parses a hex hash in string format to bytes and validates its correctness.
func ParseHexHash(hexHash string) (tmbytes.HexBytes, error) {
	hash, err := hex.DecodeString(hexHash)
	if err != nil {
		return nil, err
	}

	if err := tmtypes.ValidateHash(hash); err != nil {
		return nil, err
	}

	return hash, nil
}
