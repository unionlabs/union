// Copyright 2021 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package main

import (
	"encoding/json"
	"log"
	"os"
	"path/filepath"
)

func prettyJSON(v interface{}) []byte {
	data, err := json.MarshalIndent(v, "", "\t")
	if err != nil {
		log.Fatal(err)
	}
	return data
}

// walkDir recursively walks the given directory (following any symlinks it may
// encounter), running the handleFn on each file.
func walkDir(dir string, handleFn func(path, pathRel string, info os.FileInfo) error) error {
	var walkFn filepath.WalkFunc
	walkFn = func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if info == nil {
			return nil
		}
		if !info.Mode().IsRegular() && !info.Mode().IsDir() {
			info, err = os.Stat(path)
			if err == nil && info.Mode().IsDir() {
				_ = filepath.Walk(path+string(os.PathSeparator), walkFn)
			}
			return nil
		}

		rel, err := filepath.Rel(dir, path)
		if err != nil {
			return err
		}

		return handleFn(path, rel, info)
	}
	return filepath.Walk(dir, walkFn)
}
