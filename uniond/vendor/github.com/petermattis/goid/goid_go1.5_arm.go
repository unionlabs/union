// Copyright 2016 Peter Mattis.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied. See the License for the specific language governing
// permissions and limitations under the License. See the AUTHORS file
// for names of contributors.

//go:build (arm || arm64) && gc && go1.5
// +build arm arm64
// +build gc
// +build go1.5

package goid

// Backdoor access to runtime·getg().
func getg() *g // in goid_go1.5_arm.s or goid_go1.5_arm64.s

func Get() int64 {
	return getg().goid
}
