// Copyright 2020 Coinbase, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package asserter

import (
	"fmt"

	"github.com/coinbase/rosetta-sdk-go/types"
)

// Error ensures a *types.Error matches some error
// provided in `/network/options`.
func (a *Asserter) Error(
	err *types.Error,
) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	if err := Error(err); err != nil {
		return err
	}

	val, ok := a.errorTypeMap[err.Code]
	if !ok {
		return fmt.Errorf(
			"code %d: %w",
			err.Code,
			ErrErrorUnexpectedCode,
		)
	}

	if val.Message != err.Message {
		return fmt.Errorf(
			"expected %s actual %s: %w",
			val.Message,
			err.Message,
			ErrErrorMessageMismatch,
		)
	}

	if val.Retriable != err.Retriable {
		return fmt.Errorf(
			"expected %s actual %s: %w",
			val.Message,
			err.Message,
			ErrErrorRetriableMismatch,
		)
	}

	return nil
}
