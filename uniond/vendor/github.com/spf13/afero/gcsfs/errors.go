// Copyright © 2021 Vasily Ovchinnikov <vasily@remerge.io>.
//
// The code in this file is derived from afero fork github.com/Zatte/afero by Mikael Rapp
// licensed under Apache License 2.0.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package gcsfs

import (
	"errors"
	"syscall"
)

var (
	ErrNoBucketInName     = errors.New("no bucket name found in the name")
	ErrFileClosed         = errors.New("file is closed")
	ErrOutOfRange         = errors.New("out of range")
	ErrObjectDoesNotExist = errors.New("storage: object doesn't exist")
	ErrEmptyObjectName    = errors.New("storage: object name is empty")
	ErrFileNotFound       = syscall.ENOENT
)
