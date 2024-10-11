/*
Copyright 2021 The logr Authors.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

package testing

import (
	"fmt"
	"testing"

	"github.com/go-logr/logr"
)

func TestLogger(t *testing.T) {
	log := NewTestLogger(t)
	log.Info("info")
	log.V(0).Info("V(0).info")
	log.V(1).Info("v(1).info")
	log.Error(fmt.Errorf("error"), "error")
	log.WithName("testing").Info("with prefix")
	Helper(log, "hello world")

	log = NewTestLoggerWithOptions(t, Options{
		LogTimestamp: true,
		Verbosity:    1,
	})
	log.V(1).Info("v(1).info with options")
}

func Helper(log logr.Logger, msg string) {
	helper, log := log.WithCallStackHelper()
	helper()
	helper2(log, msg)
}

func helper2(log logr.Logger, msg string) {
	helper, log := log.WithCallStackHelper()
	helper()
	log.Info(msg)
}
