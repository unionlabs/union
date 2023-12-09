//go:build finder

package viper

import (
	"fmt"

	"github.com/sagikazarmark/locafero"
)

// Search all configPaths for any config file.
// Returns the first path that exists (and is a config file).
func (v *Viper) findConfigFile() (string, error) {
	var names []string

	if v.configType != "" {
		names = locafero.NameWithOptionalExtensions(v.configName, SupportedExts...)
	} else {
		names = locafero.NameWithExtensions(v.configName, SupportedExts...)
	}

	finder := locafero.Finder{
		Paths: v.configPaths,
		Names: names,
		Type:  locafero.FileTypeFile,
	}

	results, err := finder.Find(v.fs)
	if err != nil {
		return "", err
	}

	if len(results) == 0 {
		return "", ConfigFileNotFoundError{v.configName, fmt.Sprintf("%s", v.configPaths)}
	}

	return results[0], nil
}
