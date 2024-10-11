/*
 *
 * Copyright 2020 gRPC authors.
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

package grpctest

import (
	"testing"

	"google.golang.org/grpc/grpclog"
)

type s struct {
	Tester
}

func Test(t *testing.T) {
	RunSubTests(t, s{})
}

func (s) TestInfo(*testing.T) {
	grpclog.Info("Info", "message.")
}

func (s) TestInfoln(*testing.T) {
	grpclog.Infoln("Info", "message.")
}

func (s) TestInfof(*testing.T) {
	grpclog.Infof("%v %v.", "Info", "message")
}

func (s) TestInfoDepth(*testing.T) {
	grpclog.InfoDepth(0, "Info", "depth", "message.")
}

func (s) TestWarning(*testing.T) {
	grpclog.Warning("Warning", "message.")
}

func (s) TestWarningln(*testing.T) {
	grpclog.Warningln("Warning", "message.")
}

func (s) TestWarningf(*testing.T) {
	grpclog.Warningf("%v %v.", "Warning", "message")
}

func (s) TestWarningDepth(*testing.T) {
	grpclog.WarningDepth(0, "Warning", "depth", "message.")
}

func (s) TestError(*testing.T) {
	const numErrors = 10
	TLogger.ExpectError("Expected error")
	TLogger.ExpectError("Expected ln error")
	TLogger.ExpectError("Expected formatted error")
	TLogger.ExpectErrorN("Expected repeated error", numErrors)
	grpclog.Error("Expected", "error")
	grpclog.Errorln("Expected", "ln", "error")
	grpclog.Errorf("%v %v %v", "Expected", "formatted", "error")
	for i := 0; i < numErrors; i++ {
		grpclog.Error("Expected repeated error")
	}
}
