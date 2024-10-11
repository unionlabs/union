//go:build windows && arm
// +build windows,arm

package etw

// NewProviderWithID returns a nil provider on unsupported platforms.
func NewProviderWithOptions(name string, options ...ProviderOpt) (provider *Provider, err error) {
	return nil, nil
}
