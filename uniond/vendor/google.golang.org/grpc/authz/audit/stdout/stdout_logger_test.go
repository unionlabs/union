/*
 *
 * Copyright 2023 gRPC authors.
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

package stdout

import (
	"bytes"
	"encoding/json"
	"log"
	"os"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc/authz/audit"
	"google.golang.org/grpc/internal/grpctest"
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

func (s) TestStdoutLogger_Log(t *testing.T) {
	tests := map[string]struct {
		event       *audit.Event
		wantMessage string
		wantErr     string
	}{
		"few fields": {
			event:       &audit.Event{PolicyName: "test policy", Principal: "test principal"},
			wantMessage: `{"fullMethodName":"","principal":"test principal","policyName":"test policy","matchedRule":"","authorized":false`,
		},
		"all fields": {
			event: &audit.Event{
				FullMethodName: "/helloworld.Greeter/SayHello",
				Principal:      "spiffe://example.org/ns/default/sa/default/backend",
				PolicyName:     "example-policy",
				MatchedRule:    "dev-access",
				Authorized:     true,
			},
			wantMessage: `{"fullMethodName":"/helloworld.Greeter/SayHello",` +
				`"principal":"spiffe://example.org/ns/default/sa/default/backend","policyName":"example-policy",` +
				`"matchedRule":"dev-access","authorized":true`,
		},
	}

	for name, test := range tests {
		t.Run(name, func(t *testing.T) {
			before := time.Now().Unix()
			var buf bytes.Buffer
			builder := &loggerBuilder{goLogger: log.New(&buf, "", 0)}
			auditLogger := builder.Build(nil)

			auditLogger.Log(test.event)

			var container map[string]any
			if err := json.Unmarshal(buf.Bytes(), &container); err != nil {
				t.Fatalf("Failed to unmarshal audit log event: %v", err)
			}
			innerEvent := extractEvent(container["grpc_audit_log"].(map[string]any))
			if innerEvent.Timestamp == "" {
				t.Fatalf("Resulted event has no timestamp: %v", innerEvent)
			}
			after := time.Now().Unix()
			innerEventUnixTime, err := time.Parse(time.RFC3339Nano, innerEvent.Timestamp)
			if err != nil {
				t.Fatalf("Failed to convert event timestamp into Unix time format: %v", err)
			}
			if before > innerEventUnixTime.Unix() || after < innerEventUnixTime.Unix() {
				t.Errorf("The audit event timestamp is outside of the test interval: test start %v, event timestamp %v, test end %v", before, innerEventUnixTime.Unix(), after)
			}
			if diff := cmp.Diff(trimEvent(innerEvent), test.event); diff != "" {
				t.Fatalf("Unexpected message\ndiff (-got +want):\n%s", diff)
			}
		})
	}
}

func (s) TestStdoutLoggerBuilder_NilConfig(t *testing.T) {
	builder := &loggerBuilder{
		goLogger: log.New(os.Stdout, "", log.LstdFlags),
	}
	config, err := builder.ParseLoggerConfig(nil)
	if err != nil {
		t.Fatalf("Failed to parse stdout logger configuration: %v", err)
	}
	if l := builder.Build(config); l == nil {
		t.Fatal("Failed to build stdout audit logger")
	}
}

func (s) TestStdoutLoggerBuilder_Registration(t *testing.T) {
	if audit.GetLoggerBuilder("stdout_logger") == nil {
		t.Fatal("stdout logger is not registered")
	}
}

// extractEvent extracts an stdout.event from a map
// unmarshalled from a logged json message.
func extractEvent(container map[string]any) event {
	return event{
		FullMethodName: container["rpc_method"].(string),
		Principal:      container["principal"].(string),
		PolicyName:     container["policy_name"].(string),
		MatchedRule:    container["matched_rule"].(string),
		Authorized:     container["authorized"].(bool),
		Timestamp:      container["timestamp"].(string),
	}
}

// trimEvent converts a logged stdout.event into an audit.Event
// by removing Timestamp field. It is used for comparing events during testing.
func trimEvent(testEvent event) *audit.Event {
	return &audit.Event{
		FullMethodName: testEvent.FullMethodName,
		Principal:      testEvent.Principal,
		PolicyName:     testEvent.PolicyName,
		MatchedRule:    testEvent.MatchedRule,
		Authorized:     testEvent.Authorized,
	}
}
