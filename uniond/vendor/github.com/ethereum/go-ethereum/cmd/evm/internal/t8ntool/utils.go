// Copyright 2021 The go-ethereum Authors
// This file is part of go-ethereum.
//
// go-ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// go-ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with go-ethereum. If not, see <http://www.gnu.org/licenses/>.

package t8ntool

import (
	"encoding/json"
	"fmt"
	"os"

	"github.com/urfave/cli/v2"
)

// readFile reads the json-data in the provided path and marshals into dest.
func readFile(path, desc string, dest interface{}) error {
	inFile, err := os.Open(path)
	if err != nil {
		return NewError(ErrorIO, fmt.Errorf("failed reading %s file: %v", desc, err))
	}
	defer inFile.Close()
	decoder := json.NewDecoder(inFile)
	if err := decoder.Decode(dest); err != nil {
		return NewError(ErrorJson, fmt.Errorf("failed unmarshaling %s file: %v", desc, err))
	}
	return nil
}

// createBasedir makes sure the basedir exists, if user specified one.
func createBasedir(ctx *cli.Context) (string, error) {
	baseDir := ""
	if ctx.IsSet(OutputBasedir.Name) {
		if base := ctx.String(OutputBasedir.Name); len(base) > 0 {
			err := os.MkdirAll(base, 0755) // //rw-r--r--
			if err != nil {
				return "", err
			}
			baseDir = base
		}
	}
	return baseDir, nil
}
