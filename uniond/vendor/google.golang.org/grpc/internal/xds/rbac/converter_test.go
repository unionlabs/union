/*
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
 */

package rbac

import (
	"reflect"
	"strings"
	"testing"

	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3rbacpb "github.com/envoyproxy/go-control-plane/envoy/config/rbac/v3"
	v3auditloggersstreampb "github.com/envoyproxy/go-control-plane/envoy/extensions/rbac/audit_loggers/stream/v3"
	"google.golang.org/grpc/authz/audit"
	"google.golang.org/grpc/authz/audit/stdout"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/protobuf/types/known/anypb"
)

func (s) TestBuildLoggerErrors(t *testing.T) {
	tests := []struct {
		name           string
		loggerConfig   *v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig
		expectedLogger audit.Logger
		expectedError  string
	}{
		{
			name: "nil typed config",
			loggerConfig: &v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
				AuditLogger: &v3corepb.TypedExtensionConfig{
					TypedConfig: nil,
				},
			},
			expectedError: "missing required field: TypedConfig",
		},
		{
			name: "Unsupported Type",
			loggerConfig: &v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
				AuditLogger: &v3corepb.TypedExtensionConfig{
					Name:        "TestAuditLoggerBuffer",
					TypedConfig: testutils.MarshalAny(t, &v3rbacpb.RBAC_AuditLoggingOptions{}),
				},
			},
			expectedError: "custom config not implemented for type ",
		},
		{
			name: "Empty name",
			loggerConfig: &v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
				AuditLogger: &v3corepb.TypedExtensionConfig{
					Name:        "TestAuditLoggerBuffer",
					TypedConfig: createUDPATypedStruct(t, map[string]any{}, ""),
				},
			},
			expectedError: "field TypedConfig.TypeURL cannot be an empty string",
		},
		{
			name: "No registered logger",
			loggerConfig: &v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
				AuditLogger: &v3corepb.TypedExtensionConfig{
					Name:        "UnregisteredLogger",
					TypedConfig: createUDPATypedStruct(t, map[string]any{}, "UnregisteredLogger"),
				},
				IsOptional: false,
			},
			expectedError: "no builder registered for UnregisteredLogger",
		},
		{
			name: "fail to parse custom config",
			loggerConfig: &v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
				AuditLogger: &v3corepb.TypedExtensionConfig{
					Name:        "TestAuditLoggerCustomConfig",
					TypedConfig: createUDPATypedStruct(t, map[string]any{"abc": "BADVALUE", "xyz": "123"}, "fail to parse custom config_TestAuditLoggerCustomConfig")},
				IsOptional: false,
			},
			expectedError: "custom config could not be parsed",
		},
		{
			name: "no registered logger but optional passes",
			loggerConfig: &v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
				AuditLogger: &v3corepb.TypedExtensionConfig{
					Name:        "UnregisteredLogger",
					TypedConfig: createUDPATypedStruct(t, map[string]any{}, "no registered logger but optional passes_UnregisteredLogger"),
				},
				IsOptional: true,
			},
			expectedLogger: nil,
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			b := TestAuditLoggerCustomConfigBuilder{testName: test.name}
			audit.RegisterLoggerBuilder(&b)
			logger, err := buildLogger(test.loggerConfig)
			if err != nil && !strings.HasPrefix(err.Error(), test.expectedError) {
				t.Fatalf("expected error: %v. got error: %v", test.expectedError, err)
			}
			if logger != test.expectedLogger {
				t.Fatalf("expected logger: %v. got logger: %v", test.expectedLogger, logger)
			}

		})
	}
}

func (s) TestBuildLoggerKnownTypes(t *testing.T) {
	tests := []struct {
		name         string
		loggerConfig *v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig
		expectedType reflect.Type
	}{
		{
			name: "stdout logger",
			loggerConfig: &v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
				AuditLogger: &v3corepb.TypedExtensionConfig{
					Name:        stdout.Name,
					TypedConfig: createStdoutPb(t),
				},
				IsOptional: false,
			},
			expectedType: reflect.TypeOf(audit.GetLoggerBuilder(stdout.Name).Build(nil)),
		},
		{
			name: "stdout logger with generic TypedConfig",
			loggerConfig: &v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
				AuditLogger: &v3corepb.TypedExtensionConfig{
					Name:        stdout.Name,
					TypedConfig: createXDSTypedStruct(t, map[string]any{}, stdout.Name),
				},
				IsOptional: false,
			},
			expectedType: reflect.TypeOf(audit.GetLoggerBuilder(stdout.Name).Build(nil)),
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			logger, err := buildLogger(test.loggerConfig)
			if err != nil {
				t.Fatalf("expected success. got error: %v", err)
			}
			loggerType := reflect.TypeOf(logger)
			if test.expectedType != loggerType {
				t.Fatalf("logger not of expected type. want: %v got: %v", test.expectedType, loggerType)
			}
		})
	}
}

// Builds stdout config for audit logger proto.
func createStdoutPb(t *testing.T) *anypb.Any {
	t.Helper()
	pb := &v3auditloggersstreampb.StdoutAuditLog{}
	customConfig, err := anypb.New(pb)
	if err != nil {
		t.Fatalf("createStdoutPb failed during anypb.New: %v", err)
	}
	return customConfig
}
