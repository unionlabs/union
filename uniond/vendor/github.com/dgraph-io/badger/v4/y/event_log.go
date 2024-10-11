/*
 * Copyright 2019 Dgraph Labs, Inc. and Contributors
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

package y

import "golang.org/x/net/trace"

var (
	NoEventLog trace.EventLog = nilEventLog{}
)

type nilEventLog struct{}

func (nel nilEventLog) Printf(format string, a ...interface{}) {}

func (nel nilEventLog) Errorf(format string, a ...interface{}) {}

func (nel nilEventLog) Finish() {}
