//go:build viper_logger
// +build viper_logger

package viper

// WithLogger sets a custom logger.
func WithLogger(l Logger) Option {
	return optionFunc(func(v *Viper) {
		v.logger = l
	})
}
