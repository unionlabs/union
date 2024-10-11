package config

import (
	"errors"
	"fmt"
)

var (
	ErrEmptyRPCServerEntry             = errors.New("found empty rpc_servers entry")
	ErrNotEnoughRPCServers             = errors.New("at least two rpc_servers entries are required")
	ErrInsufficientDiscoveryTime       = errors.New("snapshot discovery time must be at least five seconds")
	ErrInsufficientChunkRequestTimeout = errors.New("timeout for re-requesting a chunk (chunk_request_timeout) is less than 5 seconds")
	ErrUnknownLogFormat                = errors.New("unknown log_format (must be 'plain' or 'json')")
	ErrSubscriptionBufferSizeInvalid   = fmt.Errorf("experimental_subscription_buffer_size must be >= %d", minSubscriptionBufferSize)
)

// ErrInSection is returned if validate basic does not pass for any underlying config service.
type ErrInSection struct {
	Err     error
	Section string
}

func (e ErrInSection) Error() string {
	return fmt.Sprintf("error in [%s] section: %s", e.Section, e.Err.Error())
}

func (e ErrInSection) Unwrap() error {
	return e.Err
}

type ErrDeprecatedBlocksyncVersion struct {
	Version string
	Allowed []string
}

func (e ErrDeprecatedBlocksyncVersion) Error() string {
	return fmt.Sprintf("blocksync version %s has been deprecated. Please use %s instead", e.Version, e.Allowed)
}

type ErrUnknownBlocksyncVersion struct {
	Version string
}

func (e ErrUnknownBlocksyncVersion) Error() string {
	return "unknown blocksync version " + e.Version
}
