// Copyright 2017 Michal Witkowski. All Rights Reserved.
// See LICENSE for licensing terms.

package grpc_logrus

import (
	"github.com/sirupsen/logrus"
	"google.golang.org/grpc/grpclog"
)

// ReplaceGrpcLogger sets the given logrus.Logger as a gRPC-level logger.
// This should be called *before* any other initialization, preferably from init() functions.
func ReplaceGrpcLogger(logger *logrus.Entry) {
	grpclog.SetLoggerV2(&logrusGrpcLoggerV2{
		logger.WithField("system", SystemField),
	})
}

type logrusGrpcLoggerV2 struct {
	*logrus.Entry
}

func (l *logrusGrpcLoggerV2) V(level int) bool {
	return l.Logger.IsLevelEnabled(logrus.Level(level))
}
