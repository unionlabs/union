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

package logr_test

import (
	"github.com/go-logr/logr"
)

// ComplexObjectRef contains more fields than it wants to get logged.
type ComplexObjectRef struct {
	Name      string
	Namespace string
	Secret    string
}

func (ref ComplexObjectRef) MarshalLog() any {
	return struct {
		Name, Namespace string
	}{
		Name:      ref.Name,
		Namespace: ref.Namespace,
	}
}

var _ logr.Marshaler = ComplexObjectRef{}

func ExampleMarshaler_secret() {
	l := NewStdoutLogger()
	secret := ComplexObjectRef{Namespace: "kube-system", Name: "some-secret", Secret: "do-not-log-me"}
	l.Info("simplified", "secret", secret)
	// Output:
	// "level"=0 "msg"="simplified" "secret"={"Name"="some-secret" "Namespace"="kube-system"}
}
