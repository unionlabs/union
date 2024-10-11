package plan

import (
	"encoding/json"
	"errors"
	"fmt"
	neturl "net/url"
	"os"
	"path/filepath"
	"regexp"
	"strings"

	"cosmossdk.io/x/upgrade/internal/conv"
)

// Info is the special structure that the Plan.Info string can be (as json).
type Info struct {
	parseConfig ParseConfig

	Binaries BinaryDownloadURLMap `json:"binaries"`
}

// BinaryDownloadURLMap is a map of os/architecture strings to a URL where the binary can be downloaded.
type BinaryDownloadURLMap map[string]string

// ParseConfig is used to configure the parsing of a Plan.Info string.
type ParseConfig struct {
	// EnforceChecksum, if true, will cause all downloaded files to be checked against their checksums.
	// When false, checksums are not enforced to be present in the url.
	EnforceChecksum bool
}

// ParseOption is used to configure the parsing of a Plan.Info string.
type ParseOption func(*ParseConfig)

// ParseOptionEnforceChecksum returns a ParseOption that sets the EnforceChecksum field of the ParseConfig.
func ParseOptionEnforceChecksum(enforce bool) ParseOption {
	return func(c *ParseConfig) {
		c.EnforceChecksum = enforce
	}
}

// ParseInfo parses an info string into a map of os/arch strings to URL string.
// If the infoStr is a url, an GET request will be made to it, and its response will be parsed instead.
func ParseInfo(infoStr string, opts ...ParseOption) (*Info, error) {
	parseConfig := &ParseConfig{}
	for _, opt := range opts {
		opt(parseConfig)
	}

	infoStr = strings.TrimSpace(infoStr)
	if len(infoStr) == 0 {
		return nil, errors.New("plan info must not be blank")
	}

	// If it's a url, download it and treat the result as the real info.
	if _, err := neturl.ParseRequestURI(infoStr); err == nil {
		if err := ValidateURL(infoStr, parseConfig.EnforceChecksum); err != nil {
			return nil, err
		}

		infoStr, err = DownloadURL(infoStr)
		if err != nil {
			return nil, err
		}
	}

	// Now, try to parse it into the expected structure.
	var planInfo Info
	if err := json.Unmarshal(conv.UnsafeStrToBytes(infoStr), &planInfo); err != nil {
		return nil, fmt.Errorf("could not parse plan info: %w", err)
	}

	planInfo.parseConfig = *parseConfig

	return &planInfo, nil
}

// ValidateFull does all possible validation of this Info.
// The provided daemonName is the name of the executable file expected in all downloaded directories.
// It checks that:
//   - Binaries.ValidateBasic() doesn't return an error
//   - Binaries.CheckURLs(daemonName) doesn't return an error.
//
// Warning: This is an expensive process. See BinaryDownloadURLMap.CheckURLs for more info.
func (m Info) ValidateFull(daemonName string) error {
	if err := m.Binaries.ValidateBasic(m.parseConfig.EnforceChecksum); err != nil {
		return err
	}
	if err := m.Binaries.CheckURLs(daemonName, m.parseConfig.EnforceChecksum); err != nil {
		return err
	}
	return nil
}

// ValidateBasic does stateless validation of this BinaryDownloadURLMap.
// It validates that:
//   - This has at least one entry.
//   - All entry keys have the format "os/arch" or are "any".
//   - All entry values are valid URLs.
//   - When `enforceChecksum` is true all URLs must contain a checksum query parameter.
func (m BinaryDownloadURLMap) ValidateBasic(enforceChecksum bool) error {
	// Make sure there's at least one.
	if len(m) == 0 {
		return errors.New("no \"binaries\" entries found")
	}

	osArchRx := regexp.MustCompile(`[a-zA-Z0-9]+/[a-zA-Z0-9]+`)
	for key, val := range m {
		if key != "any" && !osArchRx.MatchString(key) {
			return fmt.Errorf("invalid os/arch format in key \"%s\"", key)
		}

		if err := ValidateURL(val, enforceChecksum); err != nil {
			return fmt.Errorf("invalid url \"%s\" in binaries[%s]: %w", val, key, err)
		}
	}

	return nil
}

// CheckURLs checks that all entries have valid URLs that return expected data.
// The provided daemonName is the name of the executable file expected in all downloaded directories.
// Warning: This is an expensive process.
// It will make an HTTP GET request to each URL and download the response.
func (m BinaryDownloadURLMap) CheckURLs(daemonName string, enforceChecksum bool) error {
	tempDir, err := os.MkdirTemp("", "os-arch-downloads")
	if err != nil {
		return fmt.Errorf("could not create temp directory: %w", err)
	}
	defer os.RemoveAll(tempDir)
	for osArch, url := range m {
		dstRoot := filepath.Join(tempDir, strings.ReplaceAll(osArch, "/", "-"))
		if err := ValidateURL(url, enforceChecksum); err != nil {
			return fmt.Errorf("error validating url for os/arch %s: %w", osArch, err)
		}

		if err = DownloadUpgrade(dstRoot, url, daemonName); err != nil {
			return fmt.Errorf("error downloading binary for os/arch %s: %w", osArch, err)
		}
	}
	return nil
}
