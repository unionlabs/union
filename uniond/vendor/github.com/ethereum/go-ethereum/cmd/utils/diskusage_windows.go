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

package utils

import (
	"fmt"

	"golang.org/x/sys/windows"
)

func getFreeDiskSpace(path string) (uint64, error) {

	cwd, err := windows.UTF16PtrFromString(path)
	if err != nil {
		return 0, fmt.Errorf("failed to call UTF16PtrFromString: %v", err)
	}

	var freeBytesAvailableToCaller, totalNumberOfBytes, totalNumberOfFreeBytes uint64
	if err := windows.GetDiskFreeSpaceEx(cwd, &freeBytesAvailableToCaller, &totalNumberOfBytes, &totalNumberOfFreeBytes); err != nil {
		return 0, fmt.Errorf("failed to call GetDiskFreeSpaceEx: %v", err)
	}

	return freeBytesAvailableToCaller, nil
}
