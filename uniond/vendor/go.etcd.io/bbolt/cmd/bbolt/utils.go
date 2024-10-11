package main

import (
	"fmt"
	"os"
)

func checkSourceDBPath(srcPath string) (os.FileInfo, error) {
	fi, err := os.Stat(srcPath)
	if os.IsNotExist(err) {
		return nil, fmt.Errorf("source database file %q doesn't exist", srcPath)
	} else if err != nil {
		return nil, fmt.Errorf("failed to open source database file %q: %v", srcPath, err)
	}
	return fi, nil
}
