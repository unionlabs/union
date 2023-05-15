// Copyright 2020 ConsenSys AG
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package constraint

import (
	"strings"
)

// LogEntry is used as a shared data structure between the frontend and the backend
// to represent string values (in logs or debug info) where a value is not known at compile time
// (which is the case for variables that need to be resolved in the R1CS)
type LogEntry struct {
	Caller    string
	Format    string
	ToResolve []LinearExpression // TODO @gbotrel we could store here a struct with a flag that says if we expand or evaluate the expression
	Stack     []int
}

func (l *LogEntry) WriteVariable(le LinearExpression, sbb *strings.Builder) {
	// 77 correspond to the ~len(4 word modulus) in base10 string
	const elSize = 77
	sbb.Grow(len(le) * elSize)
	sbb.WriteString("%s")
	l.ToResolve = append(l.ToResolve, le)
}
