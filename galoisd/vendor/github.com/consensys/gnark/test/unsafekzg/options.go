package unsafekzg

import (
	"os"
	"path/filepath"

	"github.com/consensys/gnark/logger"
)

type Option func(*config) error

// WithCacheDir enables the filesystem cache and sets the cache directory
// to ~/.gnark/kzg by default.
func WithFSCache() Option {
	return func(opt *config) error {
		opt.fsCache = true
		return nil
	}
}

type config struct {
	fsCache  bool
	cacheDir string
}

// default options
func options(opts ...Option) (config, error) {
	var opt config

	// apply user provided options.
	for _, option := range opts {
		err := option(&opt)
		if err != nil {
			return opt, err
		}
	}

	// default value for cacheDir is ~/.gnark/kzg
	if opt.fsCache {
		if opt.cacheDir == "" {
			homeDir, err := os.UserHomeDir()
			if err != nil {
				panic(err)
			}
			opt.cacheDir = filepath.Join(homeDir, ".gnark", "kzg")
		}
		initCache(opt.cacheDir)
	}

	return opt, nil
}

func initCache(cacheDir string) {
	// get gnark logger
	log := logger.Logger()

	// populate cache from disk
	log.Warn().Str("cacheDir", cacheDir).Msg("using kzg srs cache")

	if _, err := os.Stat(cacheDir); os.IsNotExist(err) {
		err := os.MkdirAll(cacheDir, 0700)
		if err != nil {
			panic(err)
		}
	}
}
