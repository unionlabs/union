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
 */

package outlierdetection

import (
	"reflect"
	"testing"
)

func TestSuccessRateEjection(t *testing.T) {
	fields := map[string]bool{
		"StdevFactor":           true,
		"EnforcementPercentage": true,
		"MinimumHosts":          true,
		"RequestVolume":         true,
	}
	typ := reflect.TypeOf(SuccessRateEjection{})
	for i := 0; i < typ.NumField(); i++ {
		if n := typ.Field(i).Name; !fields[n] {
			t.Errorf("New field in SuccessRateEjection %q, update this test and Equal", n)
		}
	}
}

func TestEqualFieldsFailurePercentageEjection(t *testing.T) {
	fields := map[string]bool{
		"Threshold":             true,
		"EnforcementPercentage": true,
		"MinimumHosts":          true,
		"RequestVolume":         true,
	}
	typ := reflect.TypeOf(FailurePercentageEjection{})
	for i := 0; i < typ.NumField(); i++ {
		if n := typ.Field(i).Name; !fields[n] {
			t.Errorf("New field in FailurePercentageEjection %q, update this test and Equal", n)
		}
	}
}

func TestEqualFieldsLBConfig(t *testing.T) {
	fields := map[string]bool{
		"LoadBalancingConfig":       true,
		"Interval":                  true,
		"BaseEjectionTime":          true,
		"MaxEjectionTime":           true,
		"MaxEjectionPercent":        true,
		"SuccessRateEjection":       true,
		"FailurePercentageEjection": true,
		"ChildPolicy":               true,
	}
	typ := reflect.TypeOf(LBConfig{})
	for i := 0; i < typ.NumField(); i++ {
		if n := typ.Field(i).Name; !fields[n] {
			t.Errorf("New field in LBConfig %q, update this test and EqualIgnoringChildPolicy", n)
		}
	}
}
