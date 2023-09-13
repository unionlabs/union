// Copyright 2021 The Cockroach Authors.
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
// permissions and limitations under the License.

// Package redact is a helper internal package that provides the
// Unsafe and Safe value wrappers. Its name is 'redact' even though a
// better name would be 'wrappers' so as to ensure that the type name
// of generated values contains the 'redact.' prefix.
package redact
