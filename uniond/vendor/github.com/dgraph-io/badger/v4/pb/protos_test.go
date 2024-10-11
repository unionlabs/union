/* Copyright 2021 Dgraph Labs, Inc. and Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package pb

import (
	"os/exec"
	"testing"

	"github.com/stretchr/testify/require"
)

func Exec(argv ...string) error {
	cmd := exec.Command(argv[0], argv[1:]...)

	if err := cmd.Start(); err != nil {
		return err
	}
	return cmd.Wait()
}

func TestProtosRegenerate(t *testing.T) {
	err := Exec("./gen.sh")
	require.NoError(t, err, "Got error while regenerating protos: %v\n", err)

	generatedProtos := "badgerpb4.pb.go"
	err = Exec("git", "diff", "--quiet", "--", generatedProtos)
	require.NoError(t, err, "badgerpb4.pb.go changed after regenerating")
}
