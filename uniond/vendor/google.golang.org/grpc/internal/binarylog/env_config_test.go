/*
 *
 * Copyright 2018 gRPC authors.
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

package binarylog

import (
	"fmt"
	"testing"
)

// This tests that when multiple configs are specified, all methods loggers will
// be set correctly. Correctness of each logger is covered by other unit tests.
func (s) TestNewLoggerFromConfigString(t *testing.T) {
	const (
		s1     = "s1"
		m1     = "m1"
		m2     = "m2"
		fullM1 = s1 + "/" + m1
		fullM2 = s1 + "/" + m2
	)
	c := fmt.Sprintf("*{h:1;m:2},%s{h},%s{m},%s{h;m}", s1+"/*", fullM1, fullM2)
	l := NewLoggerFromConfigString(c).(*logger)

	if l.config.All.Header != 1 || l.config.All.Message != 2 {
		t.Errorf("l.config.All = %#v, want headerLen: 1, messageLen: 2", l.config.All)
	}

	if ml, ok := l.config.Services[s1]; ok {
		if ml.Header != maxUInt || ml.Message != 0 {
			t.Errorf("want maxUInt header, 0 message, got header: %v, message: %v", ml.Header, ml.Message)
		}
	} else {
		t.Errorf("service/* is not set")
	}

	if ml, ok := l.config.Methods[fullM1]; ok {
		if ml.Header != 0 || ml.Message != maxUInt {
			t.Errorf("want 0 header, maxUInt message, got header: %v, message: %v", ml.Header, ml.Message)
		}
	} else {
		t.Errorf("service/method{h} is not set")
	}

	if ml, ok := l.config.Methods[fullM2]; ok {
		if ml.Header != maxUInt || ml.Message != maxUInt {
			t.Errorf("want maxUInt header, maxUInt message, got header: %v, message: %v", ml.Header, ml.Message)
		}
	} else {
		t.Errorf("service/method{h;m} is not set")
	}
}

func (s) TestNewLoggerFromConfigStringInvalid(t *testing.T) {
	testCases := []string{
		"",
		"*{}",
		"s/m,*{}",
		"s/m,s/m{a}",

		// Duplicate rules.
		"s/m,-s/m",
		"-s/m,s/m",
		"s/m,s/m",
		"s/m,s/m{h:1;m:1}",
		"s/m{h:1;m:1},s/m",
		"-s/m,-s/m",
		"s/*,s/*{h:1;m:1}",
		"*,*{h:1;m:1}",
	}
	for _, tc := range testCases {
		l := NewLoggerFromConfigString(tc)
		if l != nil {
			t.Errorf("With config %q, want logger %v, got %v", tc, nil, l)
		}
	}
}

func (s) TestParseMethodConfigAndSuffix(t *testing.T) {
	testCases := []struct {
		in, service, method, suffix string
	}{
		{
			in:      "p.s/m",
			service: "p.s", method: "m", suffix: "",
		},
		{
			in:      "p.s/m{h,m}",
			service: "p.s", method: "m", suffix: "{h,m}",
		},
		{
			in:      "p.s/*",
			service: "p.s", method: "*", suffix: "",
		},
		{
			in:      "p.s/*{h,m}",
			service: "p.s", method: "*", suffix: "{h,m}",
		},

		// invalid suffix will be detected by another function.
		{
			in:      "p.s/m{invalidsuffix}",
			service: "p.s", method: "m", suffix: "{invalidsuffix}",
		},
		{
			in:      "p.s/*{invalidsuffix}",
			service: "p.s", method: "*", suffix: "{invalidsuffix}",
		},
		{
			in:      "s/m*",
			service: "s", method: "m", suffix: "*",
		},
		{
			in:      "s/*m",
			service: "s", method: "*", suffix: "m",
		},
		{
			in:      "s/**",
			service: "s", method: "*", suffix: "*",
		},
	}
	for _, tc := range testCases {
		t.Logf("testing parseMethodConfigAndSuffix(%q)", tc.in)
		s, m, suffix, err := parseMethodConfigAndSuffix(tc.in)
		if err != nil {
			t.Errorf("returned error %v, want nil", err)
			continue
		}
		if s != tc.service {
			t.Errorf("service = %q, want %q", s, tc.service)
		}
		if m != tc.method {
			t.Errorf("method = %q, want %q", m, tc.method)
		}
		if suffix != tc.suffix {
			t.Errorf("suffix = %q, want %q", suffix, tc.suffix)
		}
	}
}

func (s) TestParseMethodConfigAndSuffixInvalid(t *testing.T) {
	testCases := []string{
		"*/m",
		"*/m{}",
	}
	for _, tc := range testCases {
		s, m, suffix, err := parseMethodConfigAndSuffix(tc)
		if err == nil {
			t.Errorf("Parsing %q got nil error with %q, %q, %q, want non-nil error", tc, s, m, suffix)
		}
	}
}

func (s) TestParseHeaderMessageLengthConfig(t *testing.T) {
	testCases := []struct {
		in       string
		hdr, msg uint64
	}{
		{
			in:  "",
			hdr: maxUInt, msg: maxUInt,
		},
		{
			in:  "{h}",
			hdr: maxUInt, msg: 0,
		},
		{
			in:  "{h:314}",
			hdr: 314, msg: 0,
		},
		{
			in:  "{m}",
			hdr: 0, msg: maxUInt,
		},
		{
			in:  "{m:213}",
			hdr: 0, msg: 213,
		},
		{
			in:  "{h;m}",
			hdr: maxUInt, msg: maxUInt,
		},
		{
			in:  "{h:314;m}",
			hdr: 314, msg: maxUInt,
		},
		{
			in:  "{h;m:213}",
			hdr: maxUInt, msg: 213,
		},
		{
			in:  "{h:314;m:213}",
			hdr: 314, msg: 213,
		},
	}
	for _, tc := range testCases {
		t.Logf("testing parseHeaderMessageLengthConfig(%q)", tc.in)
		hdr, msg, err := parseHeaderMessageLengthConfig(tc.in)
		if err != nil {
			t.Errorf("returned error %v, want nil", err)
			continue
		}
		if hdr != tc.hdr {
			t.Errorf("header length = %v, want %v", hdr, tc.hdr)
		}
		if msg != tc.msg {
			t.Errorf("message length = %v, want %v", msg, tc.msg)
		}
	}
}
func (s) TestParseHeaderMessageLengthConfigInvalid(t *testing.T) {
	testCases := []string{
		"{}",
		"{h;a}",
		"{h;m;b}",
	}
	for _, tc := range testCases {
		_, _, err := parseHeaderMessageLengthConfig(tc)
		if err == nil {
			t.Errorf("Parsing %q got nil error, want non-nil error", tc)
		}
	}
}

func (s) TestFillMethodLoggerWithConfigStringBlacklist(t *testing.T) {
	testCases := []string{
		"p.s/m",
		"service/method",
	}
	for _, tc := range testCases {
		c := "-" + tc
		t.Logf("testing fillMethodLoggerWithConfigString(%q)", c)
		l := newEmptyLogger()
		if err := l.fillMethodLoggerWithConfigString(c); err != nil {
			t.Errorf("returned err %v, want nil", err)
			continue
		}
		_, ok := l.config.Blacklist[tc]
		if !ok {
			t.Errorf("blacklist[%q] is not set", tc)
		}
	}
}

func (s) TestFillMethodLoggerWithConfigStringGlobal(t *testing.T) {
	testCases := []struct {
		in       string
		hdr, msg uint64
	}{
		{
			in:  "",
			hdr: maxUInt, msg: maxUInt,
		},
		{
			in:  "{h}",
			hdr: maxUInt, msg: 0,
		},
		{
			in:  "{h:314}",
			hdr: 314, msg: 0,
		},
		{
			in:  "{m}",
			hdr: 0, msg: maxUInt,
		},
		{
			in:  "{m:213}",
			hdr: 0, msg: 213,
		},
		{
			in:  "{h;m}",
			hdr: maxUInt, msg: maxUInt,
		},
		{
			in:  "{h:314;m}",
			hdr: 314, msg: maxUInt,
		},
		{
			in:  "{h;m:213}",
			hdr: maxUInt, msg: 213,
		},
		{
			in:  "{h:314;m:213}",
			hdr: 314, msg: 213,
		},
	}
	for _, tc := range testCases {
		c := "*" + tc.in
		t.Logf("testing fillMethodLoggerWithConfigString(%q)", c)
		l := newEmptyLogger()
		if err := l.fillMethodLoggerWithConfigString(c); err != nil {
			t.Errorf("returned err %v, want nil", err)
			continue
		}
		if l.config.All == nil {
			t.Errorf("l.config.All is not set")
			continue
		}
		if hdr := l.config.All.Header; hdr != tc.hdr {
			t.Errorf("header length = %v, want %v", hdr, tc.hdr)

		}
		if msg := l.config.All.Message; msg != tc.msg {
			t.Errorf("message length = %v, want %v", msg, tc.msg)
		}
	}
}

func (s) TestFillMethodLoggerWithConfigStringPerService(t *testing.T) {
	testCases := []struct {
		in       string
		hdr, msg uint64
	}{
		{
			in:  "",
			hdr: maxUInt, msg: maxUInt,
		},
		{
			in:  "{h}",
			hdr: maxUInt, msg: 0,
		},
		{
			in:  "{h:314}",
			hdr: 314, msg: 0,
		},
		{
			in:  "{m}",
			hdr: 0, msg: maxUInt,
		},
		{
			in:  "{m:213}",
			hdr: 0, msg: 213,
		},
		{
			in:  "{h;m}",
			hdr: maxUInt, msg: maxUInt,
		},
		{
			in:  "{h:314;m}",
			hdr: 314, msg: maxUInt,
		},
		{
			in:  "{h;m:213}",
			hdr: maxUInt, msg: 213,
		},
		{
			in:  "{h:314;m:213}",
			hdr: 314, msg: 213,
		},
	}
	const serviceName = "service"
	for _, tc := range testCases {
		c := serviceName + "/*" + tc.in
		t.Logf("testing fillMethodLoggerWithConfigString(%q)", c)
		l := newEmptyLogger()
		if err := l.fillMethodLoggerWithConfigString(c); err != nil {
			t.Errorf("returned err %v, want nil", err)
			continue
		}
		ml, ok := l.config.Services[serviceName]
		if !ok {
			t.Errorf("l.service[%q] is not set", serviceName)
			continue
		}
		if hdr := ml.Header; hdr != tc.hdr {
			t.Errorf("header length = %v, want %v", hdr, tc.hdr)

		}
		if msg := ml.Message; msg != tc.msg {
			t.Errorf("message length = %v, want %v", msg, tc.msg)
		}
	}
}

func (s) TestFillMethodLoggerWithConfigStringPerMethod(t *testing.T) {
	testCases := []struct {
		in       string
		hdr, msg uint64
	}{
		{
			in:  "",
			hdr: maxUInt, msg: maxUInt,
		},
		{
			in:  "{h}",
			hdr: maxUInt, msg: 0,
		},
		{
			in:  "{h:314}",
			hdr: 314, msg: 0,
		},
		{
			in:  "{m}",
			hdr: 0, msg: maxUInt,
		},
		{
			in:  "{m:213}",
			hdr: 0, msg: 213,
		},
		{
			in:  "{h;m}",
			hdr: maxUInt, msg: maxUInt,
		},
		{
			in:  "{h:314;m}",
			hdr: 314, msg: maxUInt,
		},
		{
			in:  "{h;m:213}",
			hdr: maxUInt, msg: 213,
		},
		{
			in:  "{h:314;m:213}",
			hdr: 314, msg: 213,
		},
	}
	const (
		serviceName    = "service"
		methodName     = "method"
		fullMethodName = serviceName + "/" + methodName
	)
	for _, tc := range testCases {
		c := fullMethodName + tc.in
		t.Logf("testing fillMethodLoggerWithConfigString(%q)", c)
		l := newEmptyLogger()
		if err := l.fillMethodLoggerWithConfigString(c); err != nil {
			t.Errorf("returned err %v, want nil", err)
			continue
		}
		ml, ok := l.config.Methods[fullMethodName]
		if !ok {
			t.Errorf("l.config.Methods[%q] is not set", fullMethodName)
			continue
		}
		if hdr := ml.Header; hdr != tc.hdr {
			t.Errorf("header length = %v, want %v", hdr, tc.hdr)

		}
		if msg := ml.Message; msg != tc.msg {
			t.Errorf("message length = %v, want %v", msg, tc.msg)
		}
	}
}

func (s) TestFillMethodLoggerWithConfigStringInvalid(t *testing.T) {
	testCases := []string{
		"",
		"{}",
		"p.s/m{}",
		"p.s/m{a}",
		"p.s/m*",
		"p.s/**",
		"*/m",

		"-p.s/*",
		"-p.s/m{h}",
	}
	l := &logger{}
	for _, tc := range testCases {
		if err := l.fillMethodLoggerWithConfigString(tc); err == nil {
			t.Errorf("fillMethodLoggerWithConfigString(%q) returned nil error, want non-nil", tc)
		}
	}
}
