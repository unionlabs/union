package zap_test

import (
	"encoding/json"
	kitzap "github.com/go-kit/kit/log/zap"
	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
	"strings"
	"testing"
)

func TestZapSugarLogger(t *testing.T) {
	// logger config
	encoderConfig := zap.NewDevelopmentEncoderConfig()
	encoder := zapcore.NewJSONEncoder(encoderConfig)
	levelKey := encoderConfig.LevelKey
	// basic test cases
	type testCase struct {
		level zapcore.Level
		kvs   []interface{}
		want  map[string]string
	}
	testCases := []testCase{
		{level: zapcore.DebugLevel, kvs: []interface{}{"key1", "value1"},
			want: map[string]string{levelKey: "DEBUG", "key1": "value1"}},

		{level: zapcore.InfoLevel, kvs: []interface{}{"key2", "value2"},
			want: map[string]string{levelKey: "INFO", "key2": "value2"}},

		{level: zapcore.WarnLevel, kvs: []interface{}{"key3", "value3"},
			want: map[string]string{levelKey: "WARN", "key3": "value3"}},

		{level: zapcore.ErrorLevel, kvs: []interface{}{"key4", "value4"},
			want: map[string]string{levelKey: "ERROR", "key4": "value4"}},

		{level: zapcore.DPanicLevel, kvs: []interface{}{"key5", "value5"},
			want: map[string]string{levelKey: "DPANIC", "key5": "value5"}},

		{level: zapcore.PanicLevel, kvs: []interface{}{"key6", "value6"},
			want: map[string]string{levelKey: "PANIC", "key6": "value6"}},
	}
	// test
	for _, testCase := range testCases {
		t.Run(testCase.level.String(), func(t *testing.T) {
			// make logger
			writer := &tbWriter{tb: t}
			logger := zap.New(
				zapcore.NewCore(encoder, zapcore.AddSync(writer), zap.DebugLevel),
				zap.Development())
			// check panic
			shouldPanic := testCase.level >= zapcore.DPanicLevel
			kitLogger := kitzap.NewZapSugarLogger(logger, testCase.level)
			defer func() {
				isPanic := recover() != nil
				if shouldPanic != isPanic {
					t.Errorf("test level %v should panic(%v), but %v", testCase.level, shouldPanic, isPanic)
				}
				// check log kvs
				logMap := make(map[string]string)
				err := json.Unmarshal([]byte(writer.sb.String()), &logMap)
				if err != nil {
					t.Errorf("unmarshal error: %v", err)
				} else {
					for k, v := range testCase.want {
						vv, ok := logMap[k]
						if !ok || v != vv {
							t.Error("error log")
						}
					}
				}
			}()
			kitLogger.Log(testCase.kvs...)
		})
	}
}

type tbWriter struct {
	tb testing.TB
	sb strings.Builder
}

func (w *tbWriter) Write(b []byte) (n int, err error) {
	w.tb.Logf(string(b))
	w.sb.Write(b)
	return len(b), nil
}
