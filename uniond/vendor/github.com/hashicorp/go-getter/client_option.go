package getter

import (
	"context"
	"os"
)

// ClientOption is used to configure a client.
type ClientOption func(*Client) error

// Configure applies all of the given client options, along with any default
// behavior including context, decompressors, detectors, and getters used by
// the client.
func (c *Client) Configure(opts ...ClientOption) error {
	// If the context has not been configured use the background context.
	if c.Ctx == nil {
		c.Ctx = context.Background()
	}

	// Store the options used to configure this client.
	c.Options = opts

	// Apply all of the client options.
	for _, opt := range opts {
		err := opt(c)
		if err != nil {
			return err
		}
	}

	// If the client was not configured with any Decompressors, Detectors,
	// or Getters, use the default values for each.
	if c.Decompressors == nil {
		c.Decompressors = Decompressors
	}
	if c.Detectors == nil {
		c.Detectors = Detectors
	}
	if c.Getters == nil {
		c.Getters = Getters
	}

	// Set the client for each getter, so the top-level client can know
	// the getter-specific client functions or progress tracking.
	for _, getter := range c.Getters {
		getter.SetClient(c)
	}

	return nil
}

// WithContext allows to pass a context to operation
// in order to be able to cancel a download in progress.
func WithContext(ctx context.Context) ClientOption {
	return func(c *Client) error {
		c.Ctx = ctx
		return nil
	}
}

// WithDecompressors specifies which Decompressor are available.
func WithDecompressors(decompressors map[string]Decompressor) ClientOption {
	return func(c *Client) error {
		c.Decompressors = decompressors
		return nil
	}
}

// WithDecompressors specifies which compressors are available.
func WithDetectors(detectors []Detector) ClientOption {
	return func(c *Client) error {
		c.Detectors = detectors
		return nil
	}
}

// WithGetters specifies which getters are available.
func WithGetters(getters map[string]Getter) ClientOption {
	return func(c *Client) error {
		c.Getters = getters
		return nil
	}
}

// WithMode specifies which client mode the getters should operate in.
func WithMode(mode ClientMode) ClientOption {
	return func(c *Client) error {
		c.Mode = mode
		return nil
	}
}

// WithUmask specifies how to mask file permissions when storing local
// files or decompressing an archive.
func WithUmask(mode os.FileMode) ClientOption {
	return func(c *Client) error {
		c.Umask = mode
		return nil
	}
}
