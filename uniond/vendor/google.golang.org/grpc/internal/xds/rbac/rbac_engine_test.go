/*
 * Copyright 2021 gRPC authors.
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
	"context"
	"crypto/tls"
	"crypto/x509"
	"crypto/x509/pkix"
	"encoding/json"
	"fmt"
	"net"
	"net/url"
	"reflect"
	"testing"
	"time"

	v1xdsudpatypepb "github.com/cncf/xds/go/udpa/type/v1"
	v3xdsxdstypepb "github.com/cncf/xds/go/xds/type/v3"
	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3rbacpb "github.com/envoyproxy/go-control-plane/envoy/config/rbac/v3"
	v3routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"
	v3matcherpb "github.com/envoyproxy/go-control-plane/envoy/type/matcher/v3"
	v3typepb "github.com/envoyproxy/go-control-plane/envoy/type/v3"
	"google.golang.org/grpc"
	"google.golang.org/grpc/authz/audit"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/peer"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/anypb"
	"google.golang.org/protobuf/types/known/structpb"
	"google.golang.org/protobuf/types/known/wrapperspb"
)

const defaultTestTimeout = 10 * time.Second

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

type addr struct {
	ipAddress string
}

func (addr) Network() string   { return "" }
func (a *addr) String() string { return a.ipAddress }

// TestNewChainEngine tests the construction of the ChainEngine. Due to some
// types of RBAC configuration being logically wrong and returning an error
// rather than successfully constructing the RBAC Engine, this test tests both
// RBAC Configurations deemed successful and also RBAC Configurations that will
// raise errors.
func (s) TestNewChainEngine(t *testing.T) {
	tests := []struct {
		name       string
		policies   []*v3rbacpb.RBAC
		wantErr    bool
		policyName string
	}{
		{
			name: "SuccessCaseAnyMatchSingular",
			policies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"anyone": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
				},
			},
		},
		{
			name: "SuccessCaseAnyMatchMultiple",
			policies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"anyone": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
				},
				{
					Action: v3rbacpb.RBAC_DENY,
					Policies: map[string]*v3rbacpb.Policy{
						"anyone": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
				},
			},
		},
		{
			name: "SuccessCaseSimplePolicySingular",
			policies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"localhost-fan": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_DestinationPort{DestinationPort: 8080}},
								{Rule: &v3rbacpb.Permission_UrlPath{UrlPath: &v3matcherpb.PathMatcher{Rule: &v3matcherpb.PathMatcher_Path{Path: &v3matcherpb.StringMatcher{MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: "localhost-fan-page"}}}}}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
				},
			},
		},
		// SuccessCaseSimplePolicyTwoPolicies tests the construction of the
		// chained engines in the case where there are two policies in a list,
		// one with an allow policy and one with a deny policy. A situation
		// where two policies (allow and deny) is a very common use case for
		// this API, and should successfully build.
		{
			name: "SuccessCaseSimplePolicyTwoPolicies",
			policies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"localhost-fan": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_DestinationPort{DestinationPort: 8080}},
								{Rule: &v3rbacpb.Permission_UrlPath{UrlPath: &v3matcherpb.PathMatcher{Rule: &v3matcherpb.PathMatcher_Path{Path: &v3matcherpb.StringMatcher{MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: "localhost-fan-page"}}}}}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
				},
				{
					Action: v3rbacpb.RBAC_DENY,
					Policies: map[string]*v3rbacpb.Policy{
						"localhost-fan": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_DestinationPort{DestinationPort: 8080}},
								{Rule: &v3rbacpb.Permission_UrlPath{UrlPath: &v3matcherpb.PathMatcher{Rule: &v3matcherpb.PathMatcher_Path{Path: &v3matcherpb.StringMatcher{MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: "localhost-fan-page"}}}}}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
				},
			},
		},
		{
			name: "SuccessCaseEnvoyExampleSingular",
			policies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"service-admin": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Authenticated_{Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: "cluster.local/ns/default/sa/admin"}}}}},
								{Identifier: &v3rbacpb.Principal_Authenticated_{Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: "cluster.local/ns/default/sa/superuser"}}}}},
							},
						},
						"product-viewer": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_AndRules{AndRules: &v3rbacpb.Permission_Set{
									Rules: []*v3rbacpb.Permission{
										{Rule: &v3rbacpb.Permission_Header{Header: &v3routepb.HeaderMatcher{Name: ":method", HeaderMatchSpecifier: &v3routepb.HeaderMatcher_ExactMatch{ExactMatch: "GET"}}}},
										{Rule: &v3rbacpb.Permission_UrlPath{UrlPath: &v3matcherpb.PathMatcher{Rule: &v3matcherpb.PathMatcher_Path{Path: &v3matcherpb.StringMatcher{MatchPattern: &v3matcherpb.StringMatcher_Prefix{Prefix: "/products"}}}}}},
										{Rule: &v3rbacpb.Permission_OrRules{OrRules: &v3rbacpb.Permission_Set{
											Rules: []*v3rbacpb.Permission{
												{Rule: &v3rbacpb.Permission_DestinationPort{DestinationPort: 80}},
												{Rule: &v3rbacpb.Permission_DestinationPort{DestinationPort: 443}},
											},
										},
										},
										},
									},
								},
								},
								},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
				},
			},
		},
		{
			name: "SourceIpMatcherSuccessSingular",
			policies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"certain-source-ip": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_DirectRemoteIp{DirectRemoteIp: &v3corepb.CidrRange{AddressPrefix: "0.0.0.0", PrefixLen: &wrapperspb.UInt32Value{Value: uint32(10)}}}},
							},
						},
					},
				},
			},
		},
		{
			name: "SourceIpMatcherFailureSingular",
			policies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"certain-source-ip": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_DirectRemoteIp{DirectRemoteIp: &v3corepb.CidrRange{AddressPrefix: "not a correct address", PrefixLen: &wrapperspb.UInt32Value{Value: uint32(10)}}}},
							},
						},
					},
				},
			},
			wantErr: true,
		},
		{
			name: "DestinationIpMatcherSuccess",
			policies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"certain-destination-ip": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_DestinationIp{DestinationIp: &v3corepb.CidrRange{AddressPrefix: "0.0.0.0", PrefixLen: &wrapperspb.UInt32Value{Value: uint32(10)}}}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
				},
			},
		},
		{
			name: "DestinationIpMatcherFailure",
			policies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"certain-destination-ip": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_DestinationIp{DestinationIp: &v3corepb.CidrRange{AddressPrefix: "not a correct address", PrefixLen: &wrapperspb.UInt32Value{Value: uint32(10)}}}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
				},
			},
			wantErr: true,
		},
		{
			name: "MatcherToNotPolicy",
			policies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"not-secret-content": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_NotRule{NotRule: &v3rbacpb.Permission{Rule: &v3rbacpb.Permission_UrlPath{UrlPath: &v3matcherpb.PathMatcher{Rule: &v3matcherpb.PathMatcher_Path{Path: &v3matcherpb.StringMatcher{MatchPattern: &v3matcherpb.StringMatcher_Prefix{Prefix: "/secret-content"}}}}}}}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
				},
			},
		},
		{
			name: "MatcherToNotPrincipal",
			policies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"not-from-certain-ip": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_NotId{NotId: &v3rbacpb.Principal{Identifier: &v3rbacpb.Principal_DirectRemoteIp{DirectRemoteIp: &v3corepb.CidrRange{AddressPrefix: "0.0.0.0", PrefixLen: &wrapperspb.UInt32Value{Value: uint32(10)}}}}}},
							},
						},
					},
				},
			},
		},
		// PrincipalProductViewer tests the construction of a chained engine
		// with a policy that allows any downstream to send a GET request on a
		// certain path.
		{
			name: "PrincipalProductViewer",
			policies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"product-viewer": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{
									Identifier: &v3rbacpb.Principal_AndIds{AndIds: &v3rbacpb.Principal_Set{Ids: []*v3rbacpb.Principal{
										{Identifier: &v3rbacpb.Principal_Header{Header: &v3routepb.HeaderMatcher{Name: ":method", HeaderMatchSpecifier: &v3routepb.HeaderMatcher_ExactMatch{ExactMatch: "GET"}}}},
										{Identifier: &v3rbacpb.Principal_OrIds{OrIds: &v3rbacpb.Principal_Set{
											Ids: []*v3rbacpb.Principal{
												{Identifier: &v3rbacpb.Principal_UrlPath{UrlPath: &v3matcherpb.PathMatcher{Rule: &v3matcherpb.PathMatcher_Path{Path: &v3matcherpb.StringMatcher{MatchPattern: &v3matcherpb.StringMatcher_Prefix{Prefix: "/books"}}}}}},
												{Identifier: &v3rbacpb.Principal_UrlPath{UrlPath: &v3matcherpb.PathMatcher{Rule: &v3matcherpb.PathMatcher_Path{Path: &v3matcherpb.StringMatcher{MatchPattern: &v3matcherpb.StringMatcher_Prefix{Prefix: "/cars"}}}}}},
											},
										}}},
									}}},
								},
							},
						},
					},
				},
			},
		},
		// Certain Headers tests the construction of a chained engine with a
		// policy that allows any downstream to send an HTTP request with
		// certain headers.
		{
			name: "CertainHeaders",
			policies: []*v3rbacpb.RBAC{
				{
					Policies: map[string]*v3rbacpb.Policy{
						"certain-headers": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{
									Identifier: &v3rbacpb.Principal_OrIds{OrIds: &v3rbacpb.Principal_Set{Ids: []*v3rbacpb.Principal{
										{Identifier: &v3rbacpb.Principal_Header{Header: &v3routepb.HeaderMatcher{Name: ":method", HeaderMatchSpecifier: &v3routepb.HeaderMatcher_ExactMatch{ExactMatch: "GET"}}}},
										{Identifier: &v3rbacpb.Principal_Header{Header: &v3routepb.HeaderMatcher{Name: ":method", HeaderMatchSpecifier: &v3routepb.HeaderMatcher_SafeRegexMatch{SafeRegexMatch: &v3matcherpb.RegexMatcher{Regex: "GET"}}}}},
										{Identifier: &v3rbacpb.Principal_Header{Header: &v3routepb.HeaderMatcher{Name: ":method", HeaderMatchSpecifier: &v3routepb.HeaderMatcher_RangeMatch{RangeMatch: &v3typepb.Int64Range{
											Start: 0,
											End:   64,
										}}}}},
										{Identifier: &v3rbacpb.Principal_Header{Header: &v3routepb.HeaderMatcher{Name: ":method", HeaderMatchSpecifier: &v3routepb.HeaderMatcher_PresentMatch{PresentMatch: true}}}},
										{Identifier: &v3rbacpb.Principal_Header{Header: &v3routepb.HeaderMatcher{Name: ":method", HeaderMatchSpecifier: &v3routepb.HeaderMatcher_PrefixMatch{PrefixMatch: "GET"}}}},
										{Identifier: &v3rbacpb.Principal_Header{Header: &v3routepb.HeaderMatcher{Name: ":method", HeaderMatchSpecifier: &v3routepb.HeaderMatcher_SuffixMatch{SuffixMatch: "GET"}}}},
										{Identifier: &v3rbacpb.Principal_Header{Header: &v3routepb.HeaderMatcher{Name: ":method", HeaderMatchSpecifier: &v3routepb.HeaderMatcher_ContainsMatch{ContainsMatch: "GET"}}}},
										{Identifier: &v3rbacpb.Principal_Header{Header: &v3routepb.HeaderMatcher{Name: ":method", HeaderMatchSpecifier: &v3routepb.HeaderMatcher_ContainsMatch{ContainsMatch: "GET"}}}},
									}}},
								},
							},
						},
					},
				},
			},
		},
		{
			name: "LogAction",
			policies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_LOG,
					Policies: map[string]*v3rbacpb.Policy{
						"anyone": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
				},
			},
			wantErr: true,
		},
		{
			name: "ActionNotSpecified",
			policies: []*v3rbacpb.RBAC{
				{
					Policies: map[string]*v3rbacpb.Policy{
						"anyone": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
				},
			},
		},
		{
			name: "SimpleAuditLogger",
			policies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"anyone": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_ON_ALLOW,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{
								Name:        "TestAuditLoggerBuffer",
								TypedConfig: createUDPATypedStruct(t, map[string]any{}, "SimpleAuditLogger_TestAuditLoggerBuffer")},
								IsOptional: false,
							},
						},
					},
				},
			},
		},
		{
			name: "AuditLoggerCustomConfig",
			policies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"anyone": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_ON_ALLOW,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{
								Name:        "TestAuditLoggerCustomConfig",
								TypedConfig: createUDPATypedStruct(t, map[string]any{"abc": 123, "xyz": "123"}, "AuditLoggerCustomConfig_TestAuditLoggerCustomConfig")},
								IsOptional: false,
							},
						},
					},
				},
			},
			policyName: "test_policy",
		},
		{
			name: "AuditLoggerCustomConfigXdsTypedStruct",
			policies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"anyone": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_ON_ALLOW,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{
								Name:        "TestAuditLoggerCustomConfig",
								TypedConfig: createXDSTypedStruct(t, map[string]any{"abc": 123, "xyz": "123"}, "AuditLoggerCustomConfigXdsTypedStruct_TestAuditLoggerCustomConfig")},
								IsOptional: false,
							},
						},
					},
				},
			},
			policyName: "test_policy",
		},
		{
			name: "Missing Optional AuditLogger doesn't fail",
			policies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"anyone": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_ON_ALLOW,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{
								Name:        "UnsupportedLogger",
								TypedConfig: createUDPATypedStruct(t, map[string]any{}, "Missing Optional AuditLogger doesn't fail_UnsupportedLogger")},
								IsOptional: true,
							},
						},
					},
				},
			},
		},
		{
			name: "Missing Non-Optional AuditLogger fails",
			policies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"anyone": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_ON_ALLOW,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{
								Name:        "UnsupportedLogger",
								TypedConfig: createUDPATypedStruct(t, map[string]any{}, "Missing Non-Optional AuditLogger fails_UnsupportedLogger")},
								IsOptional: false,
							},
						},
					},
				},
			},
			wantErr: true,
		},
		{
			name: "Cannot_parse_missing_CustomConfig",
			policies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"anyone": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_ON_ALLOW,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{
								Name: "TestAuditLoggerCustomConfig",
							},
								IsOptional: false,
							},
						},
					},
				},
			},
			wantErr: true,
		},
		{
			name: "Cannot_parse_bad_CustomConfig",
			policies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"anyone": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_ON_ALLOW,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{
								Name:        "TestAuditLoggerCustomConfig",
								TypedConfig: createUDPATypedStruct(t, map[string]any{"abc": "BADVALUE", "xyz": "123"}, "Cannot_parse_bad_CustomConfig_TestAuditLoggerCustomConfig")},
								IsOptional: false,
							},
						},
					},
				},
			},
			wantErr: true,
		},
		{
			name: "Cannot_parse_missing_typedConfig_name",
			policies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"anyone": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_ON_ALLOW,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{
								Name:        "TestAuditLoggerCustomConfig",
								TypedConfig: createUDPATypedStruct(t, map[string]any{"abc": 123, "xyz": "123"}, "")},
								IsOptional: false,
							},
						},
					},
				},
			},
			wantErr: true,
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			b := TestAuditLoggerBufferBuilder{testName: test.name}
			audit.RegisterLoggerBuilder(&b)
			b2 := TestAuditLoggerCustomConfigBuilder{testName: test.name}
			audit.RegisterLoggerBuilder(&b2)
			if _, err := NewChainEngine(test.policies, test.policyName); (err != nil) != test.wantErr {
				t.Fatalf("NewChainEngine(%+v) returned err: %v, wantErr: %v", test.policies, err, test.wantErr)
			}
		})
	}
}

type rbacQuery struct {
	rpcData         *rpcData
	wantStatusCode  codes.Code
	wantAuditEvents []*audit.Event
}

// TestChainEngine tests the chain of RBAC Engines by configuring the chain of
// engines in a certain way in different scenarios. After configuring the chain
// of engines in a certain way, this test pings the chain of engines with
// different types of data representing incoming RPC's (piped into a context),
// and verifies that it works as expected.
func (s) TestChainEngine(t *testing.T) {
	defer func(gc func(ctx context.Context) net.Conn) {
		getConnection = gc
	}(getConnection)
	tests := []struct {
		name        string
		rbacConfigs []*v3rbacpb.RBAC
		rbacQueries []rbacQuery
		policyName  string
	}{
		// SuccessCaseAnyMatch tests a single RBAC Engine instantiated with
		// a config with a policy with any rules for both permissions and
		// principals, meaning that any data about incoming RPC's that the RBAC
		// Engine is queried with should match that policy.
		{
			name: "SuccessCaseAnyMatch",
			rbacConfigs: []*v3rbacpb.RBAC{
				{
					Policies: map[string]*v3rbacpb.Policy{
						"anyone": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
				},
			},
			rbacQueries: []rbacQuery{
				{
					rpcData: &rpcData{
						fullMethod: "some method",
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "0.0.0.0"},
						},
					},
					wantStatusCode: codes.OK,
				},
			},
		},
		// SuccessCaseSimplePolicy is a test that tests a single policy
		// that only allows an rpc to proceed if the rpc is calling with a certain
		// path.
		{
			name: "SuccessCaseSimplePolicy",
			rbacConfigs: []*v3rbacpb.RBAC{
				{
					Policies: map[string]*v3rbacpb.Policy{
						"localhost-fan": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_UrlPath{UrlPath: &v3matcherpb.PathMatcher{Rule: &v3matcherpb.PathMatcher_Path{Path: &v3matcherpb.StringMatcher{MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: "localhost-fan-page"}}}}}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
				},
			},
			rbacQueries: []rbacQuery{
				// This RPC should match with the local host fan policy. Thus,
				// this RPC should be allowed to proceed.
				{
					rpcData: &rpcData{
						fullMethod: "localhost-fan-page",
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "0.0.0.0"},
						},
					},
					wantStatusCode: codes.OK,
				},

				// This RPC shouldn't match with the local host fan policy. Thus,
				// this rpc shouldn't be allowed to proceed.
				{
					rpcData: &rpcData{
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "0.0.0.0"},
						},
					},
					wantStatusCode: codes.PermissionDenied,
				},
			},
		},
		// SuccessCaseEnvoyExample is a test based on the example provided
		// in the EnvoyProxy docs. The RBAC Config contains two policies,
		// service admin and product viewer, that provides an example of a real
		// RBAC Config that might be configured for a given for a given backend
		// service.
		{
			name: "SuccessCaseEnvoyExample",
			rbacConfigs: []*v3rbacpb.RBAC{
				{
					Policies: map[string]*v3rbacpb.Policy{
						"service-admin": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Authenticated_{Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: "//cluster.local/ns/default/sa/admin"}}}}},
								{Identifier: &v3rbacpb.Principal_Authenticated_{Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: "//cluster.local/ns/default/sa/superuser"}}}}},
							},
						},
						"product-viewer": {
							Permissions: []*v3rbacpb.Permission{
								{
									Rule: &v3rbacpb.Permission_AndRules{AndRules: &v3rbacpb.Permission_Set{
										Rules: []*v3rbacpb.Permission{
											{Rule: &v3rbacpb.Permission_Header{Header: &v3routepb.HeaderMatcher{Name: ":method", HeaderMatchSpecifier: &v3routepb.HeaderMatcher_ExactMatch{ExactMatch: "GET"}}}},
											{Rule: &v3rbacpb.Permission_UrlPath{UrlPath: &v3matcherpb.PathMatcher{Rule: &v3matcherpb.PathMatcher_Path{Path: &v3matcherpb.StringMatcher{MatchPattern: &v3matcherpb.StringMatcher_Prefix{Prefix: "/products"}}}}}},
										},
									},
									},
								},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
				},
			},
			rbacQueries: []rbacQuery{
				// This incoming RPC Call should match with the service admin
				// policy.
				{
					rpcData: &rpcData{
						fullMethod: "some method",
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "0.0.0.0"},
							AuthInfo: credentials.TLSInfo{
								State: tls.ConnectionState{
									PeerCertificates: []*x509.Certificate{
										{
											URIs: []*url.URL{
												{
													Host: "cluster.local",
													Path: "/ns/default/sa/admin",
												},
											},
										},
									},
								},
							},
						},
					},
					wantStatusCode: codes.OK,
				},
				// These incoming RPC calls should not match any policy.
				{
					rpcData: &rpcData{
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "0.0.0.0"},
						},
					},
					wantStatusCode: codes.PermissionDenied,
				},
				{
					rpcData: &rpcData{
						fullMethod: "get-product-list",
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "0.0.0.0"},
						},
					},
					wantStatusCode: codes.PermissionDenied,
				},
				{
					rpcData: &rpcData{
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "0.0.0.0"},
							AuthInfo: credentials.TLSInfo{
								State: tls.ConnectionState{
									PeerCertificates: []*x509.Certificate{
										{
											Subject: pkix.Name{
												CommonName: "localhost",
											},
										},
									},
								},
							},
						},
					},
					wantStatusCode: codes.PermissionDenied,
				},
			},
		},
		{
			name: "NotMatcher",
			rbacConfigs: []*v3rbacpb.RBAC{
				{
					Policies: map[string]*v3rbacpb.Policy{
						"not-secret-content": {
							Permissions: []*v3rbacpb.Permission{
								{
									Rule: &v3rbacpb.Permission_NotRule{
										NotRule: &v3rbacpb.Permission{Rule: &v3rbacpb.Permission_UrlPath{UrlPath: &v3matcherpb.PathMatcher{Rule: &v3matcherpb.PathMatcher_Path{Path: &v3matcherpb.StringMatcher{MatchPattern: &v3matcherpb.StringMatcher_Prefix{Prefix: "/secret-content"}}}}}},
									},
								},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
				},
			},
			rbacQueries: []rbacQuery{
				// This incoming RPC Call should match with the not-secret-content policy.
				{
					rpcData: &rpcData{
						fullMethod: "/regular-content",
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "0.0.0.0"},
						},
					},
					wantStatusCode: codes.OK,
				},
				// This incoming RPC Call shouldn't match with the not-secret-content-policy.
				{
					rpcData: &rpcData{
						fullMethod: "/secret-content",
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "0.0.0.0"},
						},
					},
					wantStatusCode: codes.PermissionDenied,
				},
			},
		},
		{
			name: "DirectRemoteIpMatcher",
			rbacConfigs: []*v3rbacpb.RBAC{
				{
					Policies: map[string]*v3rbacpb.Policy{
						"certain-direct-remote-ip": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_DirectRemoteIp{DirectRemoteIp: &v3corepb.CidrRange{AddressPrefix: "0.0.0.0", PrefixLen: &wrapperspb.UInt32Value{Value: uint32(10)}}}},
							},
						},
					},
				},
			},
			rbacQueries: []rbacQuery{
				// This incoming RPC Call should match with the certain-direct-remote-ip policy.
				{
					rpcData: &rpcData{
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "0.0.0.0"},
						},
					},
					wantStatusCode: codes.OK,
				},
				// This incoming RPC Call shouldn't match with the certain-direct-remote-ip policy.
				{
					rpcData: &rpcData{
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "10.0.0.0"},
						},
					},
					wantStatusCode: codes.PermissionDenied,
				},
			},
		},
		// This test tests a RBAC policy configured with a remote-ip policy.
		// This should be logically equivalent to configuring a Engine with a
		// direct-remote-ip policy, as per A41 - "allow equating RBAC's
		// direct_remote_ip and remote_ip."
		{
			name: "RemoteIpMatcher",
			rbacConfigs: []*v3rbacpb.RBAC{
				{
					Policies: map[string]*v3rbacpb.Policy{
						"certain-remote-ip": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_RemoteIp{RemoteIp: &v3corepb.CidrRange{AddressPrefix: "0.0.0.0", PrefixLen: &wrapperspb.UInt32Value{Value: uint32(10)}}}},
							},
						},
					},
				},
			},
			rbacQueries: []rbacQuery{
				// This incoming RPC Call should match with the certain-remote-ip policy.
				{
					rpcData: &rpcData{
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "0.0.0.0"},
						},
					},
					wantStatusCode: codes.OK,
				},
				// This incoming RPC Call shouldn't match with the certain-remote-ip policy.
				{
					rpcData: &rpcData{
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "10.0.0.0"},
						},
					},
					wantStatusCode: codes.PermissionDenied,
				},
			},
		},
		{
			name: "DestinationIpMatcher",
			rbacConfigs: []*v3rbacpb.RBAC{
				{
					Policies: map[string]*v3rbacpb.Policy{
						"certain-destination-ip": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_DestinationIp{DestinationIp: &v3corepb.CidrRange{AddressPrefix: "0.0.0.0", PrefixLen: &wrapperspb.UInt32Value{Value: uint32(10)}}}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
				},
			},
			rbacQueries: []rbacQuery{
				// This incoming RPC Call shouldn't match with the
				// certain-destination-ip policy, as the test listens on local
				// host.
				{
					rpcData: &rpcData{
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "10.0.0.0"},
						},
					},
					wantStatusCode: codes.PermissionDenied,
				},
			},
		},
		// AllowAndDenyPolicy tests a policy with an allow (on path) and
		// deny (on port) policy chained together. This represents how a user
		// configured interceptor would use this, and also is a potential
		// configuration for a dynamic xds interceptor.
		{
			name: "AllowAndDenyPolicy",
			rbacConfigs: []*v3rbacpb.RBAC{
				{
					Policies: map[string]*v3rbacpb.Policy{
						"certain-source-ip": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_DirectRemoteIp{DirectRemoteIp: &v3corepb.CidrRange{AddressPrefix: "0.0.0.0", PrefixLen: &wrapperspb.UInt32Value{Value: uint32(10)}}}},
							},
						},
					},
					Action: v3rbacpb.RBAC_ALLOW,
				},
				{
					Policies: map[string]*v3rbacpb.Policy{
						"localhost-fan": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_UrlPath{UrlPath: &v3matcherpb.PathMatcher{Rule: &v3matcherpb.PathMatcher_Path{Path: &v3matcherpb.StringMatcher{MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: "localhost-fan-page"}}}}}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
					Action: v3rbacpb.RBAC_DENY,
				},
			},
			rbacQueries: []rbacQuery{
				// This RPC should match with the allow policy, and shouldn't
				// match with the deny and thus should be allowed to proceed.
				{
					rpcData: &rpcData{
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "0.0.0.0"},
						},
					},
					wantStatusCode: codes.OK,
				},
				// This RPC should match with both the allow policy and deny policy
				// and thus shouldn't be allowed to proceed as matched with deny.
				{
					rpcData: &rpcData{
						fullMethod: "localhost-fan-page",
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "0.0.0.0"},
						},
					},
					wantStatusCode: codes.PermissionDenied,
				},
				// This RPC shouldn't match with either policy, and thus
				// shouldn't be allowed to proceed as didn't match with allow.
				{
					rpcData: &rpcData{
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "10.0.0.0"},
						},
					},
					wantStatusCode: codes.PermissionDenied,
				},
				// This RPC shouldn't match with allow, match with deny, and
				// thus shouldn't be allowed to proceed.
				{
					rpcData: &rpcData{
						fullMethod: "localhost-fan-page",
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "10.0.0.0"},
						},
					},
					wantStatusCode: codes.PermissionDenied,
				},
			},
		},
		// This test tests that when there are no SANs or Subject's
		// distinguished name in incoming RPC's, that authenticated matchers
		// match against the empty string.
		{
			name: "default-matching-no-credentials",
			rbacConfigs: []*v3rbacpb.RBAC{
				{
					Policies: map[string]*v3rbacpb.Policy{
						"service-admin": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Authenticated_{Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: ""}}}}},
							},
						},
					},
				},
			},
			rbacQueries: []rbacQuery{
				// This incoming RPC Call should match with the service admin
				// policy. No authentication info is provided, so the
				// authenticated matcher should match to the string matcher on
				// the empty string, matching to the service-admin policy.
				{
					rpcData: &rpcData{
						fullMethod: "some method",
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "0.0.0.0"},
							AuthInfo: credentials.TLSInfo{
								State: tls.ConnectionState{
									PeerCertificates: []*x509.Certificate{
										{
											URIs: []*url.URL{
												{
													Host: "cluster.local",
													Path: "/ns/default/sa/admin",
												},
											},
										},
									},
								},
							},
						},
					},
					wantStatusCode: codes.OK,
				},
			},
		},
		// This test tests that an RBAC policy configured with a metadata
		// matcher as a permission doesn't match with any incoming RPC.
		{
			name: "metadata-never-matches",
			rbacConfigs: []*v3rbacpb.RBAC{
				{
					Policies: map[string]*v3rbacpb.Policy{
						"metadata-never-matches": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Metadata{
									Metadata: &v3matcherpb.MetadataMatcher{},
								}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
				},
			},
			rbacQueries: []rbacQuery{
				{
					rpcData: &rpcData{
						fullMethod: "some method",
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "0.0.0.0"},
						},
					},
					wantStatusCode: codes.PermissionDenied,
				},
			},
		},
		// This test tests that an RBAC policy configured with a metadata
		// matcher with invert set to true as a permission always matches with
		// any incoming RPC.
		{
			name: "metadata-invert-always-matches",
			rbacConfigs: []*v3rbacpb.RBAC{
				{
					Policies: map[string]*v3rbacpb.Policy{
						"metadata-invert-always-matches": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Metadata{
									Metadata: &v3matcherpb.MetadataMatcher{Invert: true},
								}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
				},
			},
			rbacQueries: []rbacQuery{
				{
					rpcData: &rpcData{
						fullMethod: "some method",
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "0.0.0.0"},
						},
					},
					wantStatusCode: codes.OK,
				},
			},
		},
		// AllowAndDenyPolicy tests a policy with an allow (on path) and
		// deny (on port) policy chained together. This represents how a user
		// configured interceptor would use this, and also is a potential
		// configuration for a dynamic xds interceptor.  Further, it tests that
		// the audit logger works properly in each scenario.
		{
			name:       "AuditLoggingAllowAndDenyPolicy_ON_ALLOW",
			policyName: "test_policy",
			rbacConfigs: []*v3rbacpb.RBAC{
				{
					Policies: map[string]*v3rbacpb.Policy{
						"localhost-fan": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_UrlPath{UrlPath: &v3matcherpb.PathMatcher{Rule: &v3matcherpb.PathMatcher_Path{Path: &v3matcherpb.StringMatcher{MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: "localhost-fan-page"}}}}}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
					Action: v3rbacpb.RBAC_DENY,
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_NONE,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{
								Name:        "TestAuditLoggerBuffer",
								TypedConfig: createUDPATypedStruct(t, map[string]any{}, "AuditLoggingAllowAndDenyPolicy_ON_ALLOW_TestAuditLoggerBuffer")},
								IsOptional: false,
							},
						},
					},
				},
				{
					Policies: map[string]*v3rbacpb.Policy{
						"certain-source-ip": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_DirectRemoteIp{DirectRemoteIp: &v3corepb.CidrRange{AddressPrefix: "0.0.0.0", PrefixLen: &wrapperspb.UInt32Value{Value: uint32(10)}}}},
							},
						},
					},
					Action: v3rbacpb.RBAC_ALLOW,
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_ON_ALLOW,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{
								Name:        "TestAuditLoggerBuffer",
								TypedConfig: createUDPATypedStruct(t, map[string]any{}, "AuditLoggingAllowAndDenyPolicy_ON_ALLOW_TestAuditLoggerBuffer")},
								IsOptional: false,
							},
						},
					},
				},
			},
			rbacQueries: []rbacQuery{
				// This RPC should match with the allow policy, and shouldn't
				// match with the deny and thus should be allowed to proceed.
				{
					rpcData: &rpcData{
						fullMethod: "",
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "0.0.0.0"},
							AuthInfo: credentials.TLSInfo{
								State: tls.ConnectionState{
									PeerCertificates: []*x509.Certificate{
										{
											URIs: []*url.URL{
												{
													Scheme: "spiffe",
													Host:   "cluster.local",
													Path:   "/ns/default/sa/admin",
												},
											},
										},
									},
								},
								SPIFFEID: &url.URL{
									Scheme: "spiffe",
									Host:   "cluster.local",
									Path:   "/ns/default/sa/admin",
								},
							},
						},
					},
					wantStatusCode: codes.OK,
					wantAuditEvents: []*audit.Event{
						{
							FullMethodName: "",
							Principal:      "spiffe://cluster.local/ns/default/sa/admin",
							PolicyName:     "test_policy",
							MatchedRule:    "certain-source-ip",
							Authorized:     true,
						},
					},
				},
				// This RPC should match with both the allow policy and deny policy
				// and thus shouldn't be allowed to proceed as matched with deny.
				{
					rpcData: &rpcData{
						fullMethod: "localhost-fan-page",
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "0.0.0.0"},
						},
					},
					wantStatusCode: codes.PermissionDenied,
				},
				// This RPC shouldn't match with either policy, and thus
				// shouldn't be allowed to proceed as didn't match with allow.
				{
					rpcData: &rpcData{
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "10.0.0.0"},
						},
					},
					wantStatusCode: codes.PermissionDenied,
				},
				// This RPC shouldn't match with allow, match with deny, and
				// thus shouldn't be allowed to proceed.
				{
					rpcData: &rpcData{
						fullMethod: "localhost-fan-page",
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "10.0.0.0"},
						},
					},
					wantStatusCode: codes.PermissionDenied,
				},
			},
		},
		{
			name:       "AuditLoggingAllowAndDenyPolicy_ON_DENY",
			policyName: "test_policy",
			rbacConfigs: []*v3rbacpb.RBAC{
				{
					Policies: map[string]*v3rbacpb.Policy{
						"localhost-fan": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_UrlPath{UrlPath: &v3matcherpb.PathMatcher{Rule: &v3matcherpb.PathMatcher_Path{Path: &v3matcherpb.StringMatcher{MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: "localhost-fan-page"}}}}}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
					Action: v3rbacpb.RBAC_DENY,
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_ON_DENY,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{
								Name:        "TestAuditLoggerBuffer",
								TypedConfig: createUDPATypedStruct(t, map[string]any{}, "AuditLoggingAllowAndDenyPolicy_ON_DENY_TestAuditLoggerBuffer")},
								IsOptional: false,
							},
						},
					},
				},
				{
					Policies: map[string]*v3rbacpb.Policy{
						"certain-source-ip": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_DirectRemoteIp{DirectRemoteIp: &v3corepb.CidrRange{AddressPrefix: "0.0.0.0", PrefixLen: &wrapperspb.UInt32Value{Value: uint32(10)}}}},
							},
						},
					},
					Action: v3rbacpb.RBAC_ALLOW,
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_ON_DENY,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{
								Name:        "TestAuditLoggerBuffer",
								TypedConfig: createUDPATypedStruct(t, map[string]any{}, "AuditLoggingAllowAndDenyPolicy_ON_DENY_TestAuditLoggerBuffer")},
								IsOptional: false,
							},
						},
					},
				},
			},
			rbacQueries: []rbacQuery{
				// This RPC should match with the allow policy, and shouldn't
				// match with the deny and thus should be allowed to proceed.
				// Audit logging matches with nothing.
				{
					rpcData: &rpcData{
						fullMethod: "",
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "0.0.0.0"},
						},
					},
					wantStatusCode: codes.OK,
				},
				// This RPC should match with both the allow policy and deny policy
				// and thus shouldn't be allowed to proceed as matched with deny.
				// Audit logging matches with deny and short circuits.
				{
					rpcData: &rpcData{
						fullMethod: "localhost-fan-page",
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "0.0.0.0"},
							AuthInfo: credentials.TLSInfo{
								State: tls.ConnectionState{
									PeerCertificates: []*x509.Certificate{
										{
											URIs: []*url.URL{
												{
													Host: "cluster.local",
													Path: "/ns/default/sa/admin",
												},
											},
										},
									},
								},
							},
						},
					},
					wantStatusCode: codes.PermissionDenied,
					wantAuditEvents: []*audit.Event{
						{
							FullMethodName: "localhost-fan-page",
							PolicyName:     "test_policy",
							MatchedRule:    "localhost-fan",
							Authorized:     false,
						},
					},
				},
				// This RPC shouldn't match with either policy, and thus
				// shouldn't be allowed to proceed as didn't match with allow.
				// Audit logging matches with the allow policy.
				{
					rpcData: &rpcData{
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "10.0.0.0"},
						},
					},
					wantStatusCode: codes.PermissionDenied,
					wantAuditEvents: []*audit.Event{
						{
							FullMethodName: "",
							PolicyName:     "test_policy",
							MatchedRule:    "",
							Authorized:     false,
						},
					},
				},
				// This RPC shouldn't match with allow, match with deny, and
				// thus shouldn't be allowed to proceed.
				// Audit logging will have the deny logged.
				{
					rpcData: &rpcData{
						fullMethod: "localhost-fan-page",
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "10.0.0.0"},
						},
					},
					wantStatusCode: codes.PermissionDenied,
					wantAuditEvents: []*audit.Event{
						{
							FullMethodName: "localhost-fan-page",
							PolicyName:     "test_policy",
							MatchedRule:    "localhost-fan",
							Authorized:     false,
						},
					},
				},
			},
		},
		{
			name:       "AuditLoggingAllowAndDenyPolicy_NONE",
			policyName: "test_policy",
			rbacConfigs: []*v3rbacpb.RBAC{
				{
					Policies: map[string]*v3rbacpb.Policy{
						"localhost-fan": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_UrlPath{UrlPath: &v3matcherpb.PathMatcher{Rule: &v3matcherpb.PathMatcher_Path{Path: &v3matcherpb.StringMatcher{MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: "localhost-fan-page"}}}}}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
					Action: v3rbacpb.RBAC_DENY,
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_NONE,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{
								Name:        "TestAuditLoggerBuffer",
								TypedConfig: createUDPATypedStruct(t, map[string]any{}, "AuditLoggingAllowAndDenyPolicy_NONE_TestAuditLoggerBuffer")},
								IsOptional: false,
							},
						},
					},
				},
				{
					Policies: map[string]*v3rbacpb.Policy{
						"certain-source-ip": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_DirectRemoteIp{DirectRemoteIp: &v3corepb.CidrRange{AddressPrefix: "0.0.0.0", PrefixLen: &wrapperspb.UInt32Value{Value: uint32(10)}}}},
							},
						},
					},
					Action: v3rbacpb.RBAC_ALLOW,
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_NONE,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{
								Name:        "TestAuditLoggerBuffer",
								TypedConfig: createUDPATypedStruct(t, map[string]any{}, "AuditLoggingAllowAndDenyPolicy_NONE_TestAuditLoggerBuffer")},
								IsOptional: false,
							},
						},
					},
				},
			},
			rbacQueries: []rbacQuery{
				// This RPC should match with the allow policy, and shouldn't
				// match with the deny and thus should be allowed to proceed.
				// Audit logging is NONE.
				{
					rpcData: &rpcData{
						fullMethod: "",
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "0.0.0.0"},
						},
					},
					wantStatusCode: codes.OK,
				},
				// This RPC should match with both the allow policy and deny policy
				// and thus shouldn't be allowed to proceed as matched with deny.
				// Audit logging is NONE.
				{
					rpcData: &rpcData{
						fullMethod: "localhost-fan-page",
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "0.0.0.0"},
						},
					},
					wantStatusCode: codes.PermissionDenied,
				},
				// This RPC shouldn't match with either policy, and thus
				// shouldn't be allowed to proceed as didn't match with allow.
				// Audit logging is NONE.
				{
					rpcData: &rpcData{
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "10.0.0.0"},
						},
					},
					wantStatusCode: codes.PermissionDenied,
				},
				// This RPC shouldn't match with allow, match with deny, and
				// thus shouldn't be allowed to proceed.
				// Audit logging is NONE.
				{
					rpcData: &rpcData{
						fullMethod: "localhost-fan-page",
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "10.0.0.0"},
						},
					},
					wantStatusCode: codes.PermissionDenied,
				},
			},
		},
		{
			name:       "AuditLoggingAllowAndDenyPolicy_ON_DENY_AND_ALLOW",
			policyName: "test_policy",
			rbacConfigs: []*v3rbacpb.RBAC{
				{
					Policies: map[string]*v3rbacpb.Policy{
						"localhost-fan": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_UrlPath{UrlPath: &v3matcherpb.PathMatcher{Rule: &v3matcherpb.PathMatcher_Path{Path: &v3matcherpb.StringMatcher{MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: "localhost-fan-page"}}}}}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
						},
					},
					Action: v3rbacpb.RBAC_DENY,
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_ON_DENY,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{
								Name:        "TestAuditLoggerBuffer",
								TypedConfig: createUDPATypedStruct(t, map[string]any{}, "AuditLoggingAllowAndDenyPolicy_ON_DENY_AND_ALLOW_TestAuditLoggerBuffer")},
								IsOptional: false,
							},
						},
					},
				},
				{
					Policies: map[string]*v3rbacpb.Policy{
						"certain-source-ip": {
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_DirectRemoteIp{DirectRemoteIp: &v3corepb.CidrRange{AddressPrefix: "0.0.0.0", PrefixLen: &wrapperspb.UInt32Value{Value: uint32(10)}}}},
							},
						},
					},
					Action: v3rbacpb.RBAC_ALLOW,
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_ON_DENY_AND_ALLOW,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{
								Name:        "TestAuditLoggerBuffer",
								TypedConfig: createUDPATypedStruct(t, map[string]any{}, "AuditLoggingAllowAndDenyPolicy_ON_DENY_AND_ALLOW_TestAuditLoggerBuffer")},
								IsOptional: false,
							},
						},
					},
				},
			},
			rbacQueries: []rbacQuery{
				// This RPC should match with the allow policy, and shouldn't
				// match with the deny and thus should be allowed to proceed.
				// Audit logging matches with nothing.
				{
					rpcData: &rpcData{
						fullMethod: "",
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "0.0.0.0"},
						},
					},
					wantStatusCode: codes.OK,
					wantAuditEvents: []*audit.Event{
						{
							FullMethodName: "",
							PolicyName:     "test_policy",
							MatchedRule:    "certain-source-ip",
							Authorized:     true,
						},
					},
				},
				// This RPC should match with both the allow policy and deny policy
				// and thus shouldn't be allowed to proceed as matched with deny.
				// Audit logging matches with deny and short circuits.
				{
					rpcData: &rpcData{
						fullMethod: "localhost-fan-page",
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "0.0.0.0"},
						},
					},
					wantStatusCode: codes.PermissionDenied,
					wantAuditEvents: []*audit.Event{
						{
							FullMethodName: "localhost-fan-page",
							PolicyName:     "test_policy",
							MatchedRule:    "localhost-fan",
							Authorized:     false,
						},
					},
				},
				// This RPC shouldn't match with either policy, and thus
				// shouldn't be allowed to proceed as didn't match with allow.
				// Audit logging matches with the allow policy.
				{
					rpcData: &rpcData{
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "10.0.0.0"},
						},
					},
					wantStatusCode: codes.PermissionDenied,
					wantAuditEvents: []*audit.Event{
						{
							FullMethodName: "",
							PolicyName:     "test_policy",
							MatchedRule:    "",
							Authorized:     false,
						},
					},
				},
				// This RPC shouldn't match with allow, match with deny, and
				// thus shouldn't be allowed to proceed.
				// Audit logging will have the deny logged.
				{
					rpcData: &rpcData{
						fullMethod: "localhost-fan-page",
						peerInfo: &peer.Peer{
							Addr: &addr{ipAddress: "10.0.0.0"},
						},
					},
					wantStatusCode: codes.PermissionDenied,
					wantAuditEvents: []*audit.Event{
						{
							FullMethodName: "localhost-fan-page",
							PolicyName:     "test_policy",
							MatchedRule:    "localhost-fan",
							Authorized:     false,
						},
					},
				},
			},
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			b := TestAuditLoggerBufferBuilder{testName: test.name}
			audit.RegisterLoggerBuilder(&b)
			b2 := TestAuditLoggerCustomConfigBuilder{testName: test.name}
			audit.RegisterLoggerBuilder(&b2)

			// Instantiate the chainedRBACEngine with different configurations that are
			// interesting to test and to query.
			cre, err := NewChainEngine(test.rbacConfigs, test.policyName)
			if err != nil {
				t.Fatalf("Error constructing RBAC Engine: %v", err)
			}
			// Query the created chain of RBAC Engines with different args to see
			// if the chain of RBAC Engines configured as such works as intended.
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			for _, data := range test.rbacQueries {
				func() {
					// Construct the context with three data points that have enough
					// information to represent incoming RPC's. This will be how a
					// user uses this API. A user will have to put MD, PeerInfo, and
					// the connection the RPC is sent on in the context.
					ctx = metadata.NewIncomingContext(ctx, data.rpcData.md)
					// Make a TCP connection with a certain destination port. The
					// address/port of this connection will be used to populate the
					// destination ip/port in RPCData struct. This represents what
					// the user of ChainEngine will have to place into context,
					// as this is only way to get destination ip and port.
					lis, err := net.Listen("tcp", "localhost:0")
					if err != nil {
						t.Fatalf("Error listening: %v", err)
					}
					defer lis.Close()
					connCh := make(chan net.Conn, 1)
					go func() {
						conn, err := lis.Accept()
						if err != nil {
							t.Errorf("Error accepting connection: %v", err)
							return
						}
						connCh <- conn
					}()
					_, err = net.Dial("tcp", lis.Addr().String())
					if err != nil {
						t.Fatalf("Error dialing: %v", err)
					}
					conn := <-connCh
					defer conn.Close()
					getConnection = func(context.Context) net.Conn {
						return conn
					}
					ctx = peer.NewContext(ctx, data.rpcData.peerInfo)
					stream := &ServerTransportStreamWithMethod{
						method: data.rpcData.fullMethod,
					}

					ctx = grpc.NewContextWithServerTransportStream(ctx, stream)
					err = cre.IsAuthorized(ctx)
					if gotCode := status.Code(err); gotCode != data.wantStatusCode {
						t.Fatalf("IsAuthorized(%+v, %+v) returned (%+v), want(%+v)", ctx, data.rpcData.fullMethod, gotCode, data.wantStatusCode)
					}
					if !reflect.DeepEqual(b.auditEvents, data.wantAuditEvents) {
						t.Fatalf("Unexpected audit event for query:%v", data)
					}

					// This builder's auditEvents can be shared for several queries, make sure it's empty.
					b.auditEvents = nil
				}()
			}
		})
	}
}

type ServerTransportStreamWithMethod struct {
	method string
}

func (sts *ServerTransportStreamWithMethod) Method() string {
	return sts.method
}

func (sts *ServerTransportStreamWithMethod) SetHeader(metadata.MD) error {
	return nil
}

func (sts *ServerTransportStreamWithMethod) SendHeader(metadata.MD) error {
	return nil
}

func (sts *ServerTransportStreamWithMethod) SetTrailer(metadata.MD) error {
	return nil
}

// An audit logger that will log to the auditEvents slice.
type TestAuditLoggerBuffer struct {
	auditEvents *[]*audit.Event
}

func (logger *TestAuditLoggerBuffer) Log(e *audit.Event) {
	*(logger.auditEvents) = append(*(logger.auditEvents), e)
}

// Builds TestAuditLoggerBuffer.
type TestAuditLoggerBufferBuilder struct {
	auditEvents []*audit.Event
	testName    string
}

// The required config for TestAuditLoggerBuffer.
type TestAuditLoggerBufferConfig struct {
	audit.LoggerConfig
}

func (b *TestAuditLoggerBufferBuilder) ParseLoggerConfig(json.RawMessage) (config audit.LoggerConfig, err error) {
	return TestAuditLoggerBufferConfig{}, nil
}

func (b *TestAuditLoggerBufferBuilder) Build(audit.LoggerConfig) audit.Logger {
	return &TestAuditLoggerBuffer{auditEvents: &b.auditEvents}
}

func (b *TestAuditLoggerBufferBuilder) Name() string {
	return b.testName + "_TestAuditLoggerBuffer"
}

// An audit logger to test using a custom config.
type TestAuditLoggerCustomConfig struct{}

func (logger *TestAuditLoggerCustomConfig) Log(*audit.Event) {}

// Build TestAuditLoggerCustomConfig. This builds a TestAuditLoggerCustomConfig
// logger that uses a custom config.
type TestAuditLoggerCustomConfigBuilder struct {
	testName string
}

// The custom config for the TestAuditLoggerCustomConfig logger.
type TestAuditLoggerCustomConfigConfig struct {
	audit.LoggerConfig
	Abc int
	Xyz string
}

// Parses TestAuditLoggerCustomConfigConfig. Hard-coded to match with it's test
// case above.
func (b TestAuditLoggerCustomConfigBuilder) ParseLoggerConfig(configJSON json.RawMessage) (audit.LoggerConfig, error) {
	c := TestAuditLoggerCustomConfigConfig{}
	err := json.Unmarshal(configJSON, &c)
	if err != nil {
		return nil, fmt.Errorf("could not parse custom config: %v", err)
	}
	return c, nil
}

func (b *TestAuditLoggerCustomConfigBuilder) Build(audit.LoggerConfig) audit.Logger {
	return &TestAuditLoggerCustomConfig{}
}

func (b *TestAuditLoggerCustomConfigBuilder) Name() string {
	return b.testName + "_TestAuditLoggerCustomConfig"
}

// Builds custom configs for audit logger RBAC protos.
func createUDPATypedStruct(t *testing.T, in map[string]any, name string) *anypb.Any {
	t.Helper()
	pb, err := structpb.NewStruct(in)
	if err != nil {
		t.Fatalf("createUDPATypedStructFailed during structpb.NewStruct: %v", err)
	}
	typedURL := ""
	if name != "" {
		typedURL = typeURLPrefix + name
	}
	typedStruct := &v1xdsudpatypepb.TypedStruct{
		TypeUrl: typedURL,
		Value:   pb,
	}
	customConfig, err := anypb.New(typedStruct)
	if err != nil {
		t.Fatalf("createUDPATypedStructFailed during anypb.New: %v", err)
	}
	return customConfig
}

// Builds custom configs for audit logger RBAC protos.
func createXDSTypedStruct(t *testing.T, in map[string]any, name string) *anypb.Any {
	t.Helper()
	pb, err := structpb.NewStruct(in)
	if err != nil {
		t.Fatalf("createXDSTypedStructFailed during structpb.NewStruct: %v", err)
	}
	typedStruct := &v3xdsxdstypepb.TypedStruct{
		TypeUrl: typeURLPrefix + name,
		Value:   pb,
	}
	customConfig, err := anypb.New(typedStruct)
	if err != nil {
		t.Fatalf("createXDSTypedStructFailed during anypb.New: %v", err)
	}
	return customConfig
}
