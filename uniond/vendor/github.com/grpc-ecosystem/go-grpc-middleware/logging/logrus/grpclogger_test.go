package grpc_logrus

import (
	"testing"

	"github.com/sirupsen/logrus"
)

func Test_logrusGrpcLoggerV2_V(t *testing.T) {
	tests := []struct {
		name       string
		setupLevel logrus.Level
		inLevel    logrus.Level
		want       bool
	}{
		{
			name:       "WarnLevel setup when we have WarnLevel msg should return TRUE",
			setupLevel: logrus.WarnLevel,
			inLevel:    logrus.WarnLevel,
			want:       true,
		},
		{
			name:       "WarnLevel setup when we have ErrorLevel msg should return TRUE",
			setupLevel: logrus.WarnLevel,
			inLevel:    logrus.ErrorLevel,
			want:       true,
		},
		{
			name:       "WarnLevel setup when we have InfoLevel msg should return FALSE",
			setupLevel: logrus.WarnLevel,
			inLevel:    logrus.InfoLevel,
			want:       false,
		},
		{
			name:       "WarnLevel setup when we have DebugLevel msg should return FALSE",
			setupLevel: logrus.WarnLevel,
			inLevel:    logrus.DebugLevel,
			want:       false,
		},
		{
			name:       "WarnLevel setup when we have TraceLevel msg should return FALSE",
			setupLevel: logrus.WarnLevel,
			inLevel:    logrus.TraceLevel,
			want:       false,
		},
		{
			name:       "TraceLevel setup when we have WarnLevel msg should return TRUE",
			setupLevel: logrus.TraceLevel,
			inLevel:    logrus.WarnLevel,
			want:       true,
		},
		{
			name:       "TraceLevel setup when we have TraceLevel msg should return TRUE",
			setupLevel: logrus.TraceLevel,
			inLevel:    logrus.TraceLevel,
			want:       true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			lr := logrus.New()
			lr.SetLevel(tt.setupLevel)

			l := &logrusGrpcLoggerV2{logrus.NewEntry(lr)}

			if got := l.V(int(tt.inLevel)); got != tt.want {
				t.Errorf("V() = %v, want %v", got, tt.want)
			}
		})
	}
}
