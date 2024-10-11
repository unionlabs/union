/*
 *
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
 *
 */

package authz

import (
	"strings"
	"testing"

	v1xdsudpatypepb "github.com/cncf/xds/go/udpa/type/v1"
	"github.com/google/go-cmp/cmp"
	"google.golang.org/protobuf/testing/protocmp"
	"google.golang.org/protobuf/types/known/anypb"
	"google.golang.org/protobuf/types/known/structpb"

	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3rbacpb "github.com/envoyproxy/go-control-plane/envoy/config/rbac/v3"
	v3routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"
	v3matcherpb "github.com/envoyproxy/go-control-plane/envoy/type/matcher/v3"
)

func TestTranslatePolicy(t *testing.T) {
	tests := map[string]struct {
		authzPolicy    string
		wantErr        string
		wantPolicies   []*v3rbacpb.RBAC
		wantPolicyName string
	}{
		"valid policy": {
			authzPolicy: `{
						"name": "authz",
						"deny_rules": [
						{
							"name": "deny_policy_1",
							"source": {
								"principals":[
								"spiffe://foo.abc",
								"spiffe://bar*",
								"*baz",
								"spiffe://abc.*.com"
								]
							}
						}],
						"allow_rules": [
						{
							"name": "allow_policy_1",
							"source": {
								"principals":["*"]
							},
							"request": {
								"paths": ["path-foo*"]
							}
						},
						{
							"name": "allow_policy_2",
							"request": {
								"paths": [
								"path-bar",
								"*baz"
								],
								"headers": [
								{
									"key": "key-1",
									"values": ["foo", "*bar"]
								},
								{
									"key": "key-2",
									"values": ["baz*"]
								}
								]
							}
						}]
					}`,
			wantPolicies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_DENY,
					Policies: map[string]*v3rbacpb.Policy{
						"authz_deny_policy_1": {
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_OrIds{OrIds: &v3rbacpb.Principal_Set{
									Ids: []*v3rbacpb.Principal{
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: "spiffe://foo.abc"},
											}},
										}},
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_Prefix{Prefix: "spiffe://bar"},
											}},
										}},
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_Suffix{Suffix: "baz"},
											}},
										}},
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: "spiffe://abc.*.com"},
											}},
										}},
									},
								}}},
							},
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
						},
					},
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{},
				},
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"authz_allow_policy_1": {
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_OrIds{OrIds: &v3rbacpb.Principal_Set{
									Ids: []*v3rbacpb.Principal{
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_SafeRegex{SafeRegex: &v3matcherpb.RegexMatcher{Regex: ".+"}},
											}},
										}},
									},
								}}},
							},
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_AndRules{AndRules: &v3rbacpb.Permission_Set{
									Rules: []*v3rbacpb.Permission{
										{Rule: &v3rbacpb.Permission_OrRules{OrRules: &v3rbacpb.Permission_Set{
											Rules: []*v3rbacpb.Permission{
												{Rule: &v3rbacpb.Permission_UrlPath{
													UrlPath: &v3matcherpb.PathMatcher{Rule: &v3matcherpb.PathMatcher_Path{Path: &v3matcherpb.StringMatcher{
														MatchPattern: &v3matcherpb.StringMatcher_Prefix{Prefix: "path-foo"},
													}}},
												}},
											},
										}}},
									},
								}}},
							},
						},
						"authz_allow_policy_2": {
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_Any{Any: true}},
							},
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_AndRules{AndRules: &v3rbacpb.Permission_Set{
									Rules: []*v3rbacpb.Permission{
										{Rule: &v3rbacpb.Permission_OrRules{OrRules: &v3rbacpb.Permission_Set{
											Rules: []*v3rbacpb.Permission{
												{Rule: &v3rbacpb.Permission_UrlPath{
													UrlPath: &v3matcherpb.PathMatcher{Rule: &v3matcherpb.PathMatcher_Path{Path: &v3matcherpb.StringMatcher{
														MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: "path-bar"},
													}}},
												}},
												{Rule: &v3rbacpb.Permission_UrlPath{
													UrlPath: &v3matcherpb.PathMatcher{Rule: &v3matcherpb.PathMatcher_Path{Path: &v3matcherpb.StringMatcher{
														MatchPattern: &v3matcherpb.StringMatcher_Suffix{Suffix: "baz"},
													}}},
												}},
											},
										}}},
										{Rule: &v3rbacpb.Permission_AndRules{AndRules: &v3rbacpb.Permission_Set{
											Rules: []*v3rbacpb.Permission{
												{Rule: &v3rbacpb.Permission_OrRules{OrRules: &v3rbacpb.Permission_Set{
													Rules: []*v3rbacpb.Permission{
														{Rule: &v3rbacpb.Permission_Header{
															Header: &v3routepb.HeaderMatcher{
																Name: "key-1", HeaderMatchSpecifier: &v3routepb.HeaderMatcher_ExactMatch{ExactMatch: "foo"},
															},
														}},
														{Rule: &v3rbacpb.Permission_Header{
															Header: &v3routepb.HeaderMatcher{
																Name: "key-1", HeaderMatchSpecifier: &v3routepb.HeaderMatcher_SuffixMatch{SuffixMatch: "bar"},
															},
														}},
													},
												}}},
												{Rule: &v3rbacpb.Permission_OrRules{OrRules: &v3rbacpb.Permission_Set{
													Rules: []*v3rbacpb.Permission{
														{Rule: &v3rbacpb.Permission_Header{
															Header: &v3routepb.HeaderMatcher{
																Name: "key-2", HeaderMatchSpecifier: &v3routepb.HeaderMatcher_PrefixMatch{PrefixMatch: "baz"},
															},
														}},
													},
												}}},
											},
										}}},
									},
								}}},
							},
						},
					},
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{},
				},
			},
			wantPolicyName: "authz",
		},
		"allow authenticated": {
			authzPolicy: `{
						"name": "authz",
						"allow_rules": [
						{
							"name": "allow_authenticated",
							"source": {
								"principals":["*", ""]
							}
						}]
					}`,
			wantPolicies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"authz_allow_authenticated": {
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_OrIds{OrIds: &v3rbacpb.Principal_Set{
									Ids: []*v3rbacpb.Principal{
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_SafeRegex{SafeRegex: &v3matcherpb.RegexMatcher{Regex: ".+"}},
											}},
										}},
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: ""},
											}},
										}},
									},
								}}},
							},
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
						},
					},
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{},
				},
			},
		},
		"audit_logging_ALLOW empty config": {
			authzPolicy: `{
				"name": "authz",
				"allow_rules": [
				{
					"name": "allow_authenticated",
					"source": {
						"principals":["*", ""]
					}
				}],
				"deny_rules": [
				{
					"name": "deny_policy_1",
					"source": {
						"principals":[
						"spiffe://foo.abc"
						]
					}
				}],
				"audit_logging_options": {
					"audit_condition": "ON_ALLOW",
					"audit_loggers": [
						{
							"name": "stdout_logger",
							"config": {},
							"is_optional": false
						}
					]
				}
			}`,
			wantPolicies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_DENY,
					Policies: map[string]*v3rbacpb.Policy{
						"authz_deny_policy_1": {
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_OrIds{OrIds: &v3rbacpb.Principal_Set{
									Ids: []*v3rbacpb.Principal{
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: "spiffe://foo.abc"},
											}},
										}},
									},
								}}},
							},
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
						},
					},
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_NONE,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{Name: "stdout_logger", TypedConfig: anyPbHelper(t, map[string]any{}, "stdout_logger")},
								IsOptional: false,
							},
						},
					},
				},
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"authz_allow_authenticated": {
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_OrIds{OrIds: &v3rbacpb.Principal_Set{
									Ids: []*v3rbacpb.Principal{
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_SafeRegex{SafeRegex: &v3matcherpb.RegexMatcher{Regex: ".+"}},
											}},
										}},
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: ""},
											}},
										}},
									},
								}}},
							},
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
						},
					},
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_ON_ALLOW,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{Name: "stdout_logger", TypedConfig: anyPbHelper(t, map[string]any{}, "stdout_logger")},
								IsOptional: false,
							},
						},
					},
				},
			},
		},
		"audit_logging_DENY_AND_ALLOW": {
			authzPolicy: `{
				"name": "authz",
				"allow_rules": [
				{
					"name": "allow_authenticated",
					"source": {
						"principals":["*", ""]
					}
				}],
				"deny_rules": [
				{
					"name": "deny_policy_1",
					"source": {
						"principals":[
						"spiffe://foo.abc"
						]
					}
				}],
				"audit_logging_options": {
					"audit_condition": "ON_DENY_AND_ALLOW",
					"audit_loggers": [
						{
							"name": "stdout_logger",
							"config": {},
							"is_optional": false
						}
					]
				}
			}`,
			wantPolicies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_DENY,
					Policies: map[string]*v3rbacpb.Policy{
						"authz_deny_policy_1": {
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_OrIds{OrIds: &v3rbacpb.Principal_Set{
									Ids: []*v3rbacpb.Principal{
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: "spiffe://foo.abc"},
											}},
										}},
									},
								}}},
							},
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
						},
					},
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_ON_DENY,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{Name: "stdout_logger", TypedConfig: anyPbHelper(t, map[string]any{}, "stdout_logger")},
								IsOptional: false,
							},
						},
					},
				},
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"authz_allow_authenticated": {
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_OrIds{OrIds: &v3rbacpb.Principal_Set{
									Ids: []*v3rbacpb.Principal{
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_SafeRegex{SafeRegex: &v3matcherpb.RegexMatcher{Regex: ".+"}},
											}},
										}},
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: ""},
											}},
										}},
									},
								}}},
							},
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
						},
					},
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_ON_DENY_AND_ALLOW,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{Name: "stdout_logger", TypedConfig: anyPbHelper(t, map[string]any{}, "stdout_logger")},
								IsOptional: false,
							},
						},
					},
				},
			},
		},
		"audit_logging_NONE": {
			authzPolicy: `{
				"name": "authz",
				"allow_rules": [
				{
					"name": "allow_authenticated",
					"source": {
						"principals":["*", ""]
					}
				}],
				"deny_rules": [
				{
					"name": "deny_policy_1",
					"source": {
						"principals":[
						"spiffe://foo.abc"
						]
					}
				}],
				"audit_logging_options": {
					"audit_condition": "NONE",
					"audit_loggers": [
						{
							"name": "stdout_logger",
							"config": {},
							"is_optional": false
						}
					]
				}
			}`,
			wantPolicies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_DENY,
					Policies: map[string]*v3rbacpb.Policy{
						"authz_deny_policy_1": {
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_OrIds{OrIds: &v3rbacpb.Principal_Set{
									Ids: []*v3rbacpb.Principal{
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: "spiffe://foo.abc"},
											}},
										}},
									},
								}}},
							},
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
						},
					},
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_NONE,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{Name: "stdout_logger", TypedConfig: anyPbHelper(t, map[string]any{}, "stdout_logger")},
								IsOptional: false,
							},
						},
					},
				},
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"authz_allow_authenticated": {
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_OrIds{OrIds: &v3rbacpb.Principal_Set{
									Ids: []*v3rbacpb.Principal{
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_SafeRegex{SafeRegex: &v3matcherpb.RegexMatcher{Regex: ".+"}},
											}},
										}},
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: ""},
											}},
										}},
									},
								}}},
							},
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
						},
					},
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_NONE,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{Name: "stdout_logger", TypedConfig: anyPbHelper(t, map[string]any{}, "stdout_logger")},
								IsOptional: false,
							},
						},
					},
				},
			},
		},
		"audit_logging_custom_config simple": {
			authzPolicy: `{
				"name": "authz",
				"allow_rules": [
				{
					"name": "allow_authenticated",
					"source": {
						"principals":["*", ""]
					}
				}],
				"deny_rules": [
				{
					"name": "deny_policy_1",
					"source": {
						"principals":[
						"spiffe://foo.abc"
						]
					}
				}],
				"audit_logging_options": {
					"audit_condition": "NONE",
					"audit_loggers": [
						{
							"name": "stdout_logger",
							"config": {"abc":123, "xyz":"123"},
							"is_optional": false
						}
					]
				}
			}`,
			wantPolicies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_DENY,
					Policies: map[string]*v3rbacpb.Policy{
						"authz_deny_policy_1": {
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_OrIds{OrIds: &v3rbacpb.Principal_Set{
									Ids: []*v3rbacpb.Principal{
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: "spiffe://foo.abc"},
											}},
										}},
									},
								}}},
							},
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
						},
					},
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_NONE,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{Name: "stdout_logger", TypedConfig: anyPbHelper(t, map[string]any{"abc": 123, "xyz": "123"}, "stdout_logger")},
								IsOptional: false,
							},
						},
					},
				},
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"authz_allow_authenticated": {
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_OrIds{OrIds: &v3rbacpb.Principal_Set{
									Ids: []*v3rbacpb.Principal{
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_SafeRegex{SafeRegex: &v3matcherpb.RegexMatcher{Regex: ".+"}},
											}},
										}},
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: ""},
											}},
										}},
									},
								}}},
							},
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
						},
					},
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_NONE,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{Name: "stdout_logger", TypedConfig: anyPbHelper(t, map[string]any{"abc": 123, "xyz": "123"}, "stdout_logger")},
								IsOptional: false,
							},
						},
					},
				},
			},
		},
		"audit_logging_custom_config nested": {
			authzPolicy: `{
				"name": "authz",
				"allow_rules": [
				{
					"name": "allow_authenticated",
					"source": {
						"principals":["*", ""]
					}
				}],
				"audit_logging_options": {
					"audit_condition": "NONE",
					"audit_loggers": [
						{
							"name": "stdout_logger",
							"config": {"abc":123, "xyz":{"abc":123}},
							"is_optional": false
						}
					]
				}
			}`,
			wantPolicies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"authz_allow_authenticated": {
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_OrIds{OrIds: &v3rbacpb.Principal_Set{
									Ids: []*v3rbacpb.Principal{
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_SafeRegex{SafeRegex: &v3matcherpb.RegexMatcher{Regex: ".+"}},
											}},
										}},
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: ""},
											}},
										}},
									},
								}}},
							},
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
						},
					},
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_NONE,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{Name: "stdout_logger", TypedConfig: anyPbHelper(t, map[string]any{"abc": 123, "xyz": map[string]any{"abc": 123}}, "stdout_logger")},
								IsOptional: false,
							},
						},
					},
				},
			},
		},
		"missing audit logger config": {
			authzPolicy: `{
				"name": "authz",
				"allow_rules": [
				{
					"name": "allow_authenticated",
					"source": {
						"principals":["*", ""]
					}
				}],
				"audit_logging_options": {
					"audit_condition": "NONE"
				}
			}`,
			wantPolicies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"authz_allow_authenticated": {
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_OrIds{OrIds: &v3rbacpb.Principal_Set{
									Ids: []*v3rbacpb.Principal{
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_SafeRegex{SafeRegex: &v3matcherpb.RegexMatcher{Regex: ".+"}},
											}},
										}},
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: ""},
											}},
										}},
									},
								}}},
							},
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
						},
					},
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_NONE,
						LoggerConfigs:  []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{},
					},
				},
			},
		},
		"missing audit condition": {
			authzPolicy: `{
				"name": "authz",
				"allow_rules": [
				{
					"name": "allow_authenticated",
					"source": {
						"principals":["*", ""]
					}
				}],
				"audit_logging_options": {
					"audit_loggers": [
						{
							"name": "stdout_logger",
							"config": {},
							"is_optional": false
						}
					]
				}
			}`,
			wantPolicies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"authz_allow_authenticated": {
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_OrIds{OrIds: &v3rbacpb.Principal_Set{
									Ids: []*v3rbacpb.Principal{
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_SafeRegex{SafeRegex: &v3matcherpb.RegexMatcher{Regex: ".+"}},
											}},
										}},
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: ""},
											}},
										}},
									},
								}}},
							},
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
						},
					},
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_NONE,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{Name: "stdout_logger", TypedConfig: anyPbHelper(t, map[string]any{}, "stdout_logger")},
								IsOptional: false,
							},
						},
					},
				},
			},
		},
		"missing custom config audit logger": {
			authzPolicy: `{
				"name": "authz",
				"allow_rules": [
				{
					"name": "allow_authenticated",
					"source": {
						"principals":["*", ""]
					}
				}],
				"deny_rules": [
				{
					"name": "deny_policy_1",
					"source": {
						"principals":[
						"spiffe://foo.abc"
						]
					}
				}],
				"audit_logging_options": {
					"audit_condition": "ON_DENY",
					"audit_loggers": [
						{
							"name": "stdout_logger",
							"is_optional": false
						}
					]
				}
			}`,
			wantPolicies: []*v3rbacpb.RBAC{
				{
					Action: v3rbacpb.RBAC_DENY,
					Policies: map[string]*v3rbacpb.Policy{
						"authz_deny_policy_1": {
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_OrIds{OrIds: &v3rbacpb.Principal_Set{
									Ids: []*v3rbacpb.Principal{
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: "spiffe://foo.abc"},
											}},
										}},
									},
								}}},
							},
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
						},
					},
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_ON_DENY,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{Name: "stdout_logger", TypedConfig: anyPbHelper(t, map[string]any{}, "stdout_logger")},
								IsOptional: false,
							},
						},
					},
				},
				{
					Action: v3rbacpb.RBAC_ALLOW,
					Policies: map[string]*v3rbacpb.Policy{
						"authz_allow_authenticated": {
							Principals: []*v3rbacpb.Principal{
								{Identifier: &v3rbacpb.Principal_OrIds{OrIds: &v3rbacpb.Principal_Set{
									Ids: []*v3rbacpb.Principal{
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_SafeRegex{SafeRegex: &v3matcherpb.RegexMatcher{Regex: ".+"}},
											}},
										}},
										{Identifier: &v3rbacpb.Principal_Authenticated_{
											Authenticated: &v3rbacpb.Principal_Authenticated{PrincipalName: &v3matcherpb.StringMatcher{
												MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: ""},
											}},
										}},
									},
								}}},
							},
							Permissions: []*v3rbacpb.Permission{
								{Rule: &v3rbacpb.Permission_Any{Any: true}},
							},
						},
					},
					AuditLoggingOptions: &v3rbacpb.RBAC_AuditLoggingOptions{
						AuditCondition: v3rbacpb.RBAC_AuditLoggingOptions_ON_DENY,
						LoggerConfigs: []*v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
							{AuditLogger: &v3corepb.TypedExtensionConfig{Name: "stdout_logger", TypedConfig: anyPbHelper(t, map[string]any{}, "stdout_logger")},
								IsOptional: false,
							},
						},
					},
				},
			},
		},
		"unknown field": {
			authzPolicy: `{"random": 123}`,
			wantErr:     "failed to unmarshal policy",
		},
		"missing name field": {
			authzPolicy: `{}`,
			wantErr:     `"name" is not present`,
		},
		"invalid field type": {
			authzPolicy: `{"name": 123}`,
			wantErr:     "failed to unmarshal policy",
		},
		"missing allow rules field": {
			authzPolicy: `{"name": "authz-foo"}`,
			wantErr:     `"allow_rules" is not present`,
		},
		"missing rule name field": {
			authzPolicy: `{
				"name": "authz-foo",
				"allow_rules": [{}]
			}`,
			wantErr: `"allow_rules" 0: "name" is not present`,
		},
		"missing header key": {
			authzPolicy: `{
				"name": "authz",
				"allow_rules": [{
					"name": "allow_policy_1",
					"request": {"headers":[{"key":"key-a", "values": ["value-a"]}, {}]}
				}]
			}`,
			wantErr: `"allow_rules" 0: "headers" 1: "key" is not present`,
		},
		"missing header values": {
			authzPolicy: `{
				"name": "authz",
				"allow_rules": [{
					"name": "allow_policy_1",
					"request": {"headers":[{"key":"key-a"}]}
				}]
			}`,
			wantErr: `"allow_rules" 0: "headers" 0: "values" is not present`,
		},
		"unsupported header": {
			authzPolicy: `{
				"name": "authz",
				"allow_rules": [{
					"name": "allow_policy_1",
					"request": {"headers":[{"key":":method", "values":["GET"]}]}
				}]
			}`,
			wantErr: `"allow_rules" 0: "headers" 0: unsupported "key" :method`,
		},
		"bad audit condition": {
			authzPolicy: `{
				"name": "authz",
				"allow_rules": [
				{
					"name": "allow_authenticated",
					"source": {
						"principals":["*", ""]
					}
				}],
				"audit_logging_options": {
					"audit_condition": "ABC",
					"audit_loggers": [
						{
							"name": "stdout_logger",
							"config": {},
							"is_optional": false
						}
					]
				}
			}`,
			wantErr: `failed to parse AuditCondition ABC`,
		},
		"bad audit logger config": {
			authzPolicy: `{
				"name": "authz",
				"allow_rules": [
				{
					"name": "allow_authenticated",
					"source": {
						"principals":["*", ""]
					}
				}],
				"audit_logging_options": {
					"audit_condition": "NONE",
					"audit_loggers": [
						{
							"name": "stdout_logger",
							"config": "abc",
							"is_optional": false
						}
					]
				}
			}`,
			wantErr: `failed to unmarshal policy`,
		},
		"missing audit logger name": {
			authzPolicy: `{
				"name": "authz",
				"allow_rules": [
				{
					"name": "allow_authenticated",
					"source": {
						"principals":["*", ""]
					}
				}],
				"audit_logging_options": {
					"audit_condition": "NONE",
					"audit_loggers": [
						{
							"name": "",
							"config": {},
							"is_optional": false
						}
					]
				}
			}`,
			wantErr: `missing required field: name`,
		},
	}
	for name, test := range tests {
		t.Run(name, func(t *testing.T) {
			gotPolicies, gotPolicyName, gotErr := translatePolicy(test.authzPolicy)
			if gotErr != nil && !strings.HasPrefix(gotErr.Error(), test.wantErr) {
				t.Fatalf("unexpected error\nwant:%v\ngot:%v", test.wantErr, gotErr)
			}
			if diff := cmp.Diff(gotPolicies, test.wantPolicies, protocmp.Transform()); diff != "" {
				t.Fatalf("unexpected policy\ndiff (-want +got):\n%s", diff)
			}
			if test.wantPolicyName != "" && gotPolicyName != test.wantPolicyName {
				t.Fatalf("unexpected policy name\nwant:%v\ngot:%v", test.wantPolicyName, gotPolicyName)
			}
		})
	}
}

func anyPbHelper(t *testing.T, in map[string]any, name string) *anypb.Any {
	t.Helper()
	pb, err := structpb.NewStruct(in)
	typedStruct := &v1xdsudpatypepb.TypedStruct{
		TypeUrl: typeURLPrefix + name,
		Value:   pb,
	}
	if err != nil {
		t.Fatal(err)
	}
	customConfig, err := anypb.New(typedStruct)
	if err != nil {
		t.Fatal(err)
	}
	return customConfig
}
