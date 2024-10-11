/*
 *
 * Copyright 2015 gRPC authors.
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

// Package glogger defines glog-based logging for grpc.
// Importing this package will install glog as the logger used by grpclog.
package glogger

import (
	"fmt"

	"github.com/golang/glog"
	"google.golang.org/grpc/grpclog"
)

const d = 2

func init() {
	grpclog.SetLoggerV2(&glogger{})
}

type glogger struct{}

func (g *glogger) Info(args ...any) {
	glog.InfoDepth(d, args...)
}

func (g *glogger) Infoln(args ...any) {
	glog.InfoDepth(d, fmt.Sprintln(args...))
}

func (g *glogger) Infof(format string, args ...any) {
	glog.InfoDepth(d, fmt.Sprintf(format, args...))
}

func (g *glogger) InfoDepth(depth int, args ...any) {
	glog.InfoDepth(depth+d, args...)
}

func (g *glogger) Warning(args ...any) {
	glog.WarningDepth(d, args...)
}

func (g *glogger) Warningln(args ...any) {
	glog.WarningDepth(d, fmt.Sprintln(args...))
}

func (g *glogger) Warningf(format string, args ...any) {
	glog.WarningDepth(d, fmt.Sprintf(format, args...))
}

func (g *glogger) WarningDepth(depth int, args ...any) {
	glog.WarningDepth(depth+d, args...)
}

func (g *glogger) Error(args ...any) {
	glog.ErrorDepth(d, args...)
}

func (g *glogger) Errorln(args ...any) {
	glog.ErrorDepth(d, fmt.Sprintln(args...))
}

func (g *glogger) Errorf(format string, args ...any) {
	glog.ErrorDepth(d, fmt.Sprintf(format, args...))
}

func (g *glogger) ErrorDepth(depth int, args ...any) {
	glog.ErrorDepth(depth+d, args...)
}

func (g *glogger) Fatal(args ...any) {
	glog.FatalDepth(d, args...)
}

func (g *glogger) Fatalln(args ...any) {
	glog.FatalDepth(d, fmt.Sprintln(args...))
}

func (g *glogger) Fatalf(format string, args ...any) {
	glog.FatalDepth(d, fmt.Sprintf(format, args...))
}

func (g *glogger) FatalDepth(depth int, args ...any) {
	glog.FatalDepth(depth+d, args...)
}

func (g *glogger) V(l int) bool {
	return bool(glog.V(glog.Level(l)))
}
