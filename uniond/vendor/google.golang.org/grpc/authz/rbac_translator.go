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

// Package authz exposes methods to manage authorization within gRPC.
//
// # Experimental
//
// Notice: This package is EXPERIMENTAL and may be changed or removed
// in a later release.
package authz

import (
	"bytes"
	"encoding/json"
	"fmt"
	"strings"

	v1xdsudpatypepb "github.com/cncf/xds/go/udpa/type/v1"
	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3rbacpb "github.com/envoyproxy/go-control-plane/envoy/config/rbac/v3"
	v3routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"
	v3matcherpb "github.com/envoyproxy/go-control-plane/envoy/type/matcher/v3"
	"google.golang.org/protobuf/types/known/anypb"
	"google.golang.org/protobuf/types/known/structpb"
)

// This is used when converting a custom config from raw JSON to a TypedStruct
// The TypeURL of the TypeStruct will be "grpc.authz.audit_logging/<name>"
const typeURLPrefix = "grpc.authz.audit_logging/"

type header struct {
	Key    string
	Values []string
}

type peer struct {
	Principals []string
}

type request struct {
	Paths   []string
	Headers []header
}

type rule struct {
	Name    string
	Source  peer
	Request request
}

type auditLogger struct {
	Name       string           `json:"name"`
	Config     *structpb.Struct `json:"config"`
	IsOptional bool             `json:"is_optional"`
}

type auditLoggingOptions struct {
	AuditCondition string         `json:"audit_condition"`
	AuditLoggers   []*auditLogger `json:"audit_loggers"`
}

// Represents the SDK authorization policy provided by user.
type authorizationPolicy struct {
	Name                string
	DenyRules           []rule              `json:"deny_rules"`
	AllowRules          []rule              `json:"allow_rules"`
	AuditLoggingOptions auditLoggingOptions `json:"audit_logging_options"`
}

func principalOr(principals []*v3rbacpb.Principal) *v3rbacpb.Principal {
	return &v3rbacpb.Principal{
		Identifier: &v3rbacpb.Principal_OrIds{
			OrIds: &v3rbacpb.Principal_Set{
				Ids: principals,
			},
		},
	}
}

func permissionOr(permission []*v3rbacpb.Permission) *v3rbacpb.Permission {
	return &v3rbacpb.Permission{
		Rule: &v3rbacpb.Permission_OrRules{
			OrRules: &v3rbacpb.Permission_Set{
				Rules: permission,
			},
		},
	}
}

func permissionAnd(permission []*v3rbacpb.Permission) *v3rbacpb.Permission {
	return &v3rbacpb.Permission{
		Rule: &v3rbacpb.Permission_AndRules{
			AndRules: &v3rbacpb.Permission_Set{
				Rules: permission,
			},
		},
	}
}

func getStringMatcher(value string) *v3matcherpb.StringMatcher {
	switch {
	case value == "*":
		return &v3matcherpb.StringMatcher{
			MatchPattern: &v3matcherpb.StringMatcher_SafeRegex{
				SafeRegex: &v3matcherpb.RegexMatcher{Regex: ".+"}},
		}
	case strings.HasSuffix(value, "*"):
		prefix := strings.TrimSuffix(value, "*")
		return &v3matcherpb.StringMatcher{
			MatchPattern: &v3matcherpb.StringMatcher_Prefix{Prefix: prefix},
		}
	case strings.HasPrefix(value, "*"):
		suffix := strings.TrimPrefix(value, "*")
		return &v3matcherpb.StringMatcher{
			MatchPattern: &v3matcherpb.StringMatcher_Suffix{Suffix: suffix},
		}
	default:
		return &v3matcherpb.StringMatcher{
			MatchPattern: &v3matcherpb.StringMatcher_Exact{Exact: value},
		}
	}
}

func getHeaderMatcher(key, value string) *v3routepb.HeaderMatcher {
	switch {
	case value == "*":
		return &v3routepb.HeaderMatcher{
			Name: key,
			HeaderMatchSpecifier: &v3routepb.HeaderMatcher_SafeRegexMatch{
				SafeRegexMatch: &v3matcherpb.RegexMatcher{Regex: ".+"}},
		}
	case strings.HasSuffix(value, "*"):
		prefix := strings.TrimSuffix(value, "*")
		return &v3routepb.HeaderMatcher{
			Name:                 key,
			HeaderMatchSpecifier: &v3routepb.HeaderMatcher_PrefixMatch{PrefixMatch: prefix},
		}
	case strings.HasPrefix(value, "*"):
		suffix := strings.TrimPrefix(value, "*")
		return &v3routepb.HeaderMatcher{
			Name:                 key,
			HeaderMatchSpecifier: &v3routepb.HeaderMatcher_SuffixMatch{SuffixMatch: suffix},
		}
	default:
		return &v3routepb.HeaderMatcher{
			Name:                 key,
			HeaderMatchSpecifier: &v3routepb.HeaderMatcher_ExactMatch{ExactMatch: value},
		}
	}
}

func parsePrincipalNames(principalNames []string) []*v3rbacpb.Principal {
	ps := make([]*v3rbacpb.Principal, 0, len(principalNames))
	for _, principalName := range principalNames {
		newPrincipalName := &v3rbacpb.Principal{
			Identifier: &v3rbacpb.Principal_Authenticated_{
				Authenticated: &v3rbacpb.Principal_Authenticated{
					PrincipalName: getStringMatcher(principalName),
				},
			}}
		ps = append(ps, newPrincipalName)
	}
	return ps
}

func parsePeer(source peer) *v3rbacpb.Principal {
	if len(source.Principals) == 0 {
		return &v3rbacpb.Principal{
			Identifier: &v3rbacpb.Principal_Any{
				Any: true,
			},
		}
	}
	return principalOr(parsePrincipalNames(source.Principals))
}

func parsePaths(paths []string) []*v3rbacpb.Permission {
	ps := make([]*v3rbacpb.Permission, 0, len(paths))
	for _, path := range paths {
		newPath := &v3rbacpb.Permission{
			Rule: &v3rbacpb.Permission_UrlPath{
				UrlPath: &v3matcherpb.PathMatcher{
					Rule: &v3matcherpb.PathMatcher_Path{Path: getStringMatcher(path)}}}}
		ps = append(ps, newPath)
	}
	return ps
}

func parseHeaderValues(key string, values []string) []*v3rbacpb.Permission {
	vs := make([]*v3rbacpb.Permission, 0, len(values))
	for _, value := range values {
		newHeader := &v3rbacpb.Permission{
			Rule: &v3rbacpb.Permission_Header{
				Header: getHeaderMatcher(key, value)}}
		vs = append(vs, newHeader)
	}
	return vs
}

var unsupportedHeaders = map[string]bool{
	"host":                true,
	"connection":          true,
	"keep-alive":          true,
	"proxy-authenticate":  true,
	"proxy-authorization": true,
	"te":                  true,
	"trailer":             true,
	"transfer-encoding":   true,
	"upgrade":             true,
}

func unsupportedHeader(key string) bool {
	return key[0] == ':' || strings.HasPrefix(key, "grpc-") || unsupportedHeaders[key]
}

func parseHeaders(headers []header) ([]*v3rbacpb.Permission, error) {
	hs := make([]*v3rbacpb.Permission, 0, len(headers))
	for i, header := range headers {
		if header.Key == "" {
			return nil, fmt.Errorf(`"headers" %d: "key" is not present`, i)
		}
		header.Key = strings.ToLower(header.Key)
		if unsupportedHeader(header.Key) {
			return nil, fmt.Errorf(`"headers" %d: unsupported "key" %s`, i, header.Key)
		}
		if len(header.Values) == 0 {
			return nil, fmt.Errorf(`"headers" %d: "values" is not present`, i)
		}
		values := parseHeaderValues(header.Key, header.Values)
		hs = append(hs, permissionOr(values))
	}
	return hs, nil
}

func parseRequest(request request) (*v3rbacpb.Permission, error) {
	var and []*v3rbacpb.Permission
	if len(request.Paths) > 0 {
		and = append(and, permissionOr(parsePaths(request.Paths)))
	}
	if len(request.Headers) > 0 {
		headers, err := parseHeaders(request.Headers)
		if err != nil {
			return nil, err
		}
		and = append(and, permissionAnd(headers))
	}
	if len(and) > 0 {
		return permissionAnd(and), nil
	}
	return &v3rbacpb.Permission{
		Rule: &v3rbacpb.Permission_Any{
			Any: true,
		},
	}, nil
}

func parseRules(rules []rule, prefixName string) (map[string]*v3rbacpb.Policy, error) {
	policies := make(map[string]*v3rbacpb.Policy)
	for i, rule := range rules {
		if rule.Name == "" {
			return policies, fmt.Errorf(`%d: "name" is not present`, i)
		}
		permission, err := parseRequest(rule.Request)
		if err != nil {
			return nil, fmt.Errorf("%d: %v", i, err)
		}
		policyName := prefixName + "_" + rule.Name
		policies[policyName] = &v3rbacpb.Policy{
			Principals:  []*v3rbacpb.Principal{parsePeer(rule.Source)},
			Permissions: []*v3rbacpb.Permission{permission},
		}
	}
	return policies, nil
}

// Parse auditLoggingOptions to the associated RBAC protos. The single
// auditLoggingOptions results in two different parsed protos, one for the allow
// policy and one for the deny policy
func (options *auditLoggingOptions) toProtos() (allow *v3rbacpb.RBAC_AuditLoggingOptions, deny *v3rbacpb.RBAC_AuditLoggingOptions, err error) {
	allow = &v3rbacpb.RBAC_AuditLoggingOptions{}
	deny = &v3rbacpb.RBAC_AuditLoggingOptions{}

	if options.AuditCondition != "" {
		rbacCondition, ok := v3rbacpb.RBAC_AuditLoggingOptions_AuditCondition_value[options.AuditCondition]
		if !ok {
			return nil, nil, fmt.Errorf("failed to parse AuditCondition %v. Allowed values {NONE, ON_DENY, ON_ALLOW, ON_DENY_AND_ALLOW}", options.AuditCondition)
		}
		allow.AuditCondition = v3rbacpb.RBAC_AuditLoggingOptions_AuditCondition(rbacCondition)
		deny.AuditCondition = toDenyCondition(v3rbacpb.RBAC_AuditLoggingOptions_AuditCondition(rbacCondition))
	}

	for i, config := range options.AuditLoggers {
		if config.Name == "" {
			return nil, nil, fmt.Errorf("missing required field: name in audit_logging_options.audit_loggers[%v]", i)
		}
		if config.Config == nil {
			config.Config = &structpb.Struct{}
		}
		typedStruct := &v1xdsudpatypepb.TypedStruct{
			TypeUrl: typeURLPrefix + config.Name,
			Value:   config.Config,
		}
		customConfig, err := anypb.New(typedStruct)
		if err != nil {
			return nil, nil, fmt.Errorf("error parsing custom audit logger config: %v", err)
		}

		logger := &v3corepb.TypedExtensionConfig{Name: config.Name, TypedConfig: customConfig}
		rbacConfig := v3rbacpb.RBAC_AuditLoggingOptions_AuditLoggerConfig{
			IsOptional:  config.IsOptional,
			AuditLogger: logger,
		}
		allow.LoggerConfigs = append(allow.LoggerConfigs, &rbacConfig)
		deny.LoggerConfigs = append(deny.LoggerConfigs, &rbacConfig)
	}

	return allow, deny, nil
}

// Maps the AuditCondition coming from AuditLoggingOptions to the proper
// condition for the deny policy RBAC proto
func toDenyCondition(condition v3rbacpb.RBAC_AuditLoggingOptions_AuditCondition) v3rbacpb.RBAC_AuditLoggingOptions_AuditCondition {
	// Mapping the overall policy AuditCondition to what it must be for the Deny and Allow RBAC
	// See gRPC A59 for details - https://github.com/grpc/proposal/pull/346/files
	// |Authorization Policy  |DENY RBAC          |ALLOW RBAC           |
	// |----------------------|-------------------|---------------------|
	// |NONE                  |NONE               |NONE                 |
	// |ON_DENY               |ON_DENY            |ON_DENY              |
	// |ON_ALLOW              |NONE               |ON_ALLOW             |
	// |ON_DENY_AND_ALLOW     |ON_DENY            |ON_DENY_AND_ALLOW    |
	switch condition {
	case v3rbacpb.RBAC_AuditLoggingOptions_NONE:
		return v3rbacpb.RBAC_AuditLoggingOptions_NONE
	case v3rbacpb.RBAC_AuditLoggingOptions_ON_DENY:
		return v3rbacpb.RBAC_AuditLoggingOptions_ON_DENY
	case v3rbacpb.RBAC_AuditLoggingOptions_ON_ALLOW:
		return v3rbacpb.RBAC_AuditLoggingOptions_NONE
	case v3rbacpb.RBAC_AuditLoggingOptions_ON_DENY_AND_ALLOW:
		return v3rbacpb.RBAC_AuditLoggingOptions_ON_DENY
	default:
		return v3rbacpb.RBAC_AuditLoggingOptions_NONE
	}
}

// translatePolicy translates SDK authorization policy in JSON format to two
// Envoy RBAC polices (deny followed by allow policy) or only one Envoy RBAC
// allow policy. Also returns the overall policy name. If the input policy
// cannot be parsed or is invalid, an error will be returned.
func translatePolicy(policyStr string) ([]*v3rbacpb.RBAC, string, error) {
	policy := &authorizationPolicy{}
	d := json.NewDecoder(bytes.NewReader([]byte(policyStr)))
	d.DisallowUnknownFields()
	if err := d.Decode(policy); err != nil {
		return nil, "", fmt.Errorf("failed to unmarshal policy: %v", err)
	}
	if policy.Name == "" {
		return nil, "", fmt.Errorf(`"name" is not present`)
	}
	if len(policy.AllowRules) == 0 {
		return nil, "", fmt.Errorf(`"allow_rules" is not present`)
	}
	allowLogger, denyLogger, err := policy.AuditLoggingOptions.toProtos()
	if err != nil {
		return nil, "", err
	}
	rbacs := make([]*v3rbacpb.RBAC, 0, 2)
	if len(policy.DenyRules) > 0 {
		denyPolicies, err := parseRules(policy.DenyRules, policy.Name)
		if err != nil {
			return nil, "", fmt.Errorf(`"deny_rules" %v`, err)
		}
		denyRBAC := &v3rbacpb.RBAC{
			Action:              v3rbacpb.RBAC_DENY,
			Policies:            denyPolicies,
			AuditLoggingOptions: denyLogger,
		}
		rbacs = append(rbacs, denyRBAC)
	}
	allowPolicies, err := parseRules(policy.AllowRules, policy.Name)
	if err != nil {
		return nil, "", fmt.Errorf(`"allow_rules" %v`, err)
	}
	allowRBAC := &v3rbacpb.RBAC{Action: v3rbacpb.RBAC_ALLOW, Policies: allowPolicies, AuditLoggingOptions: allowLogger}
	return append(rbacs, allowRBAC), policy.Name, nil
}
