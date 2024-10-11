/*
 *
 * Copyright 2021 gRPC authors.
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
 *
 */

package admin_test

import (
	"testing"

	"google.golang.org/grpc/admin/test"
	"google.golang.org/grpc/codes"
)

func TestRegisterNoCSDS(t *testing.T) {
	test.RunRegisterTests(t, test.ExpectedStatusCodes{
		ChannelzCode: codes.OK,
		// CSDS is not registered because xDS isn't imported.
		CSDSCode: codes.Unimplemented,
	})
}
