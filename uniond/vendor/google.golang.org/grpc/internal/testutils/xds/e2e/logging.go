/*
 *
 * Copyright 2022 gRPC authors.
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

package e2e

// serverLogger implements the Logger interface defined at
// envoyproxy/go-control-plane/pkg/log. This is passed to the Snapshot cache.
type serverLogger struct {
	logger interface {
		Logf(format string, args ...any)
	}
}

func (l serverLogger) Debugf(format string, args ...any) {
	l.logger.Logf(format, args...)
}
func (l serverLogger) Infof(format string, args ...any) {
	l.logger.Logf(format, args...)
}
func (l serverLogger) Warnf(format string, args ...any) {
	l.logger.Logf(format, args...)
}
func (l serverLogger) Errorf(format string, args ...any) {
	l.logger.Logf(format, args...)
}
