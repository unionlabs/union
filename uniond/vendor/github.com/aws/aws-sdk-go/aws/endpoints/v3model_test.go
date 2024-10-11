//go:build go1.9
// +build go1.9

package endpoints

import (
	"bytes"
	"encoding/json"
	"io/ioutil"
	"log"
	"net/url"
	"os"
	"path/filepath"
	"reflect"
	"regexp"
	"strconv"
	"strings"
	"testing"
)

// ***************************************************************************
// All endpoint metadata is sourced from the testdata/endpoints.json file at
// test startup. Not the live endpoints model file. Update the testdata file
// for the tests to use the latest live model.
// ***************************************************************************

func TestUnmarshalRegionRegex(t *testing.T) {
	var input = []byte(`
{
    "regionRegex": "^(us|eu|ap|sa|ca)\\-\\w+\\-\\d+$"
}`)

	p := partition{}
	err := json.Unmarshal(input, &p)
	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}

	expectRegexp, err := regexp.Compile(`^(us|eu|ap|sa|ca)\-\w+\-\d+$`)
	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}

	if e, a := expectRegexp.String(), p.RegionRegex.Regexp.String(); e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
}

func TestUnmarshalRegion(t *testing.T) {
	var input = []byte(`
{
	"aws-global": {
	  "description": "AWS partition-global endpoint"
	},
	"us-east-1": {
	  "description": "US East (N. Virginia)"
	}
}`)

	rs := regions{}
	err := json.Unmarshal(input, &rs)
	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}

	if e, a := 2, len(rs); e != a {
		t.Errorf("expect %v len, got %v", e, a)
	}
	r, ok := rs["aws-global"]
	if !ok {
		t.Errorf("expect found, was not")
	}
	if e, a := "AWS partition-global endpoint", r.Description; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}

	r, ok = rs["us-east-1"]
	if !ok {
		t.Errorf("expect found, was not")
	}
	if e, a := "US East (N. Virginia)", r.Description; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
}

func TestUnmarshalServices(t *testing.T) {
	var input = []byte(`
{
	"acm": {
	  "endpoints": {
		"us-east-1": {}
	  }
	},
	"apigateway": {
      "isRegionalized": true,
	  "endpoints": {
		"us-east-1": {},
        "us-west-2": {}
	  }
	},
	"notRegionalized": {
      "isRegionalized": false,
	  "endpoints": {
		"us-east-1": {},
        "us-west-2": {}
	  }
	}
}`)

	ss := services{}
	err := json.Unmarshal(input, &ss)
	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}

	if e, a := 3, len(ss); e != a {
		t.Errorf("expect %v len, got %v", e, a)
	}
	s, ok := ss["acm"]
	if !ok {
		t.Errorf("expect found, was not")
	}
	if e, a := 1, len(s.Endpoints); e != a {
		t.Errorf("expect %v len, got %v", e, a)
	}
	if e, a := boxedBoolUnset, s.IsRegionalized; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}

	s, ok = ss["apigateway"]
	if !ok {
		t.Errorf("expect found, was not")
	}
	if e, a := 2, len(s.Endpoints); e != a {
		t.Errorf("expect %v len, got %v", e, a)
	}
	if e, a := boxedTrue, s.IsRegionalized; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}

	s, ok = ss["notRegionalized"]
	if !ok {
		t.Errorf("expect found, was not")
	}
	if e, a := 2, len(s.Endpoints); e != a {
		t.Errorf("expect %v len, got %v", e, a)
	}
	if e, a := boxedFalse, s.IsRegionalized; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
}

func TestUnmarshalEndpoints(t *testing.T) {
	var inputs = []byte(`
{
	"aws-global": {
	  "hostname": "cloudfront.amazonaws.com",
	  "protocols": [
		"http",
		"https"
	  ],
	  "signatureVersions": [ "v4" ],
	  "credentialScope": {
		"region": "us-east-1",
		"service": "serviceName"
	  },
	  "sslCommonName": "commonName"
	},
	"us-east-1": {}
}`)

	es := serviceEndpoints{}
	err := json.Unmarshal(inputs, &es)
	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}

	if e, a := 2, len(es); e != a {
		t.Errorf("expect %v len, got %v", e, a)
	}
	s, ok := es[endpointKey{Region: "aws-global"}]
	if !ok {
		t.Errorf("expect found, was not")
	}
	if e, a := "cloudfront.amazonaws.com", s.Hostname; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := []string{"http", "https"}, s.Protocols; !reflect.DeepEqual(e, a) {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := []string{"v4"}, s.SignatureVersions; !reflect.DeepEqual(e, a) {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := (credentialScope{"us-east-1", "serviceName"}), s.CredentialScope; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "commonName", s.SSLCommonName; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
}

func TestEndpointResolve(t *testing.T) {
	defs := []endpoint{
		{
			Hostname:          "{service}.{region}.{dnsSuffix}",
			SignatureVersions: []string{"v2"},
			SSLCommonName:     "sslCommonName",
		},
		{
			Hostname:  "other-hostname",
			Protocols: []string{"http"},
			CredentialScope: credentialScope{
				Region:  "signing_region",
				Service: "signing_service",
			},
		},
	}

	e := endpoint{
		Hostname:          "{service}.{region}.{dnsSuffix}",
		Protocols:         []string{"http", "https"},
		SignatureVersions: []string{"v4"},
		SSLCommonName:     "new sslCommonName",
	}

	resolved, err := e.resolve("service", "partitionID", "region", dnsSuffixTemplateKey, "dnsSuffix",
		defs, Options{},
	)
	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}

	if e, a := "https://service.region.dnsSuffix", resolved.URL; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "signing_service", resolved.SigningName; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "signing_region", resolved.SigningRegion; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "v4", resolved.SigningMethod; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}

	// Check Invalid Region Identifier Format
	_, err = e.resolve("service", "partitionID", "notvalid.com", dnsSuffixTemplateKey, "dnsSuffix",
		defs, Options{},
	)
	if err == nil {
		t.Errorf("expected err, got nil")
	}
}

func TestEndpointMergeIn(t *testing.T) {
	expected := endpoint{
		Hostname:          "other hostname",
		Protocols:         []string{"http"},
		SignatureVersions: []string{"v4"},
		SSLCommonName:     "ssl common name",
		CredentialScope: credentialScope{
			Region:  "region",
			Service: "service",
		},
	}

	actual := endpoint{}
	actual.mergeIn(endpoint{
		Hostname:          "other hostname",
		Protocols:         []string{"http"},
		SignatureVersions: []string{"v4"},
		SSLCommonName:     "ssl common name",
		CredentialScope: credentialScope{
			Region:  "region",
			Service: "service",
		},
	})

	if e, a := expected, actual; !reflect.DeepEqual(e, a) {
		t.Errorf("expect %v, got %v", e, a)
	}
}

func TestResolveEndpoint(t *testing.T) {
	resolved, err := testPartitions.EndpointFor("service2", "us-west-2")

	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}
	if e, a := "https://service2.us-west-2.amazonaws.com", resolved.URL; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "us-west-2", resolved.SigningRegion; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "service2", resolved.SigningName; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if resolved.SigningNameDerived {
		t.Errorf("expect the signing name not to be derived, but was")
	}
}

func TestResolveEndpoint_DisableSSL(t *testing.T) {
	resolved, err := testPartitions.EndpointFor("service2", "us-west-2", DisableSSLOption)

	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}
	if e, a := "http://service2.us-west-2.amazonaws.com", resolved.URL; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "us-west-2", resolved.SigningRegion; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "service2", resolved.SigningName; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if resolved.SigningNameDerived {
		t.Errorf("expect the signing name not to be derived, but was")
	}
}

func TestResolveEndpoint_UseDualStack_UseDualStackEndpoint(t *testing.T) {
	cases := map[string]struct {
		Service string
		Region  string

		Options func(*Options)

		ExpectedURL              string
		ExpectedSigningName      string
		ExpectedSigningRegion    string
		ExpectSigningNameDerived bool

		ExpectErr bool
	}{
		"deprecated UseDualStack does not apply to services that are not s3 or s3-control": {
			Service:                  "ec2",
			Region:                   "us-west-2",
			Options:                  UseDualStackOption,
			ExpectedURL:              "https://ec2.us-west-2.amazonaws.com",
			ExpectedSigningName:      "ec2",
			ExpectedSigningRegion:    "us-west-2",
			ExpectSigningNameDerived: true,
		},
		"deprecated UseDualStack allowed for s3": {
			Service:                  "s3",
			Region:                   "us-west-2",
			Options:                  UseDualStackOption,
			ExpectedURL:              "https://s3.dualstack.us-west-2.amazonaws.com",
			ExpectedSigningName:      "s3",
			ExpectedSigningRegion:    "us-west-2",
			ExpectSigningNameDerived: true,
		},
		"deprecated UseDualStack allowed for s3-control": {
			Service:                  "s3-control",
			Region:                   "us-west-2",
			Options:                  UseDualStackOption,
			ExpectedURL:              "https://s3-control.dualstack.us-west-2.amazonaws.com",
			ExpectedSigningName:      "s3-control",
			ExpectedSigningRegion:    "us-west-2",
			ExpectSigningNameDerived: true,
		},
		"UseDualStackEndpoint applies to all services": {
			Service:                  "ec2",
			Region:                   "us-west-2",
			Options:                  UseDualStackEndpointOption,
			ExpectedURL:              "https://api.ec2.us-west-2.aws",
			ExpectedSigningName:      "ec2",
			ExpectedSigningRegion:    "us-west-2",
			ExpectSigningNameDerived: true,
		},
		"UseDualStackEndpoint applies to s3": {
			Service:                  "s3",
			Region:                   "us-west-2",
			Options:                  UseDualStackEndpointOption,
			ExpectedURL:              "https://s3.dualstack.us-west-2.amazonaws.com",
			ExpectedSigningName:      "s3",
			ExpectedSigningRegion:    "us-west-2",
			ExpectSigningNameDerived: true,
		},
		"UseDualStackEndpoint applies to s3-control": {
			Service:                  "s3-control",
			Region:                   "us-west-2",
			Options:                  UseDualStackEndpointOption,
			ExpectedURL:              "https://s3-control.dualstack.us-west-2.amazonaws.com",
			ExpectedSigningName:      "s3-control",
			ExpectedSigningRegion:    "us-west-2",
			ExpectSigningNameDerived: true,
		},
		"UseDualStackEndpoint (disabled) setting has higher precedence then UseDualStack for s3": {
			Service: "s3",
			Region:  "us-west-2",
			Options: func(options *Options) {
				options.UseDualStack = true
				options.UseDualStackEndpoint = DualStackEndpointStateDisabled
			},
			ExpectedURL:              "https://s3.us-west-2.amazonaws.com",
			ExpectedSigningName:      "s3",
			ExpectedSigningRegion:    "us-west-2",
			ExpectSigningNameDerived: true,
		},
		"UseDualStackEndpoint (disabled) setting has higher precedence then UseDualStack for s3-control": {
			Service: "s3-control",
			Region:  "us-west-2",
			Options: func(options *Options) {
				options.UseDualStack = true
				options.UseDualStackEndpoint = DualStackEndpointStateDisabled
			},
			ExpectedURL:              "https://s3-control.us-west-2.amazonaws.com",
			ExpectedSigningName:      "s3-control",
			ExpectedSigningRegion:    "us-west-2",
			ExpectSigningNameDerived: true,
		},
		"UseDualStackEndpoint in partition with no partition or service defaults": {
			Service:   "service1",
			Region:    "cn-north-2",
			Options:   UseDualStackEndpointOption,
			ExpectErr: true,
		},
	}

	for name, tt := range cases {
		t.Run(name, func(t *testing.T) {
			if tt.Options == nil {
				tt.Options = func(options *Options) {}
			}

			resolved, err := AwsPartition().EndpointFor(tt.Service, tt.Region, tt.Options)
			if tt.ExpectErr != (err != nil) {
				t.Fatalf("ExpectErr=%v, got err=%v", tt.ExpectErr, err)
			}

			assertEndpoint(t, resolved, tt.ExpectedURL, tt.ExpectedSigningName, tt.ExpectedSigningRegion)

			if e, a := tt.ExpectSigningNameDerived, resolved.SigningNameDerived; e != a {
				t.Errorf("ExpectSigningNameDerived(%v) != SigningNameDerived(%v)", e, a)
			}
		})
	}
}

func TestResolveEndpoint_HTTPProtocol(t *testing.T) {
	resolved, err := testPartitions.EndpointFor("httpService", "us-west-2")

	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}
	if e, a := "http://httpService.us-west-2.amazonaws.com", resolved.URL; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "us-west-2", resolved.SigningRegion; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "httpService", resolved.SigningName; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if !resolved.SigningNameDerived {
		t.Errorf("expect the signing name to be derived")
	}
}

func TestResolveEndpoint_UnknownService(t *testing.T) {
	_, err := testPartitions.EndpointFor("unknownservice", "us-west-2")

	if err == nil {
		t.Errorf("expect error, got none")
	}

	_, ok := err.(UnknownServiceError)
	if !ok {
		t.Errorf("expect error to be UnknownServiceError")
	}
}

func TestResolveEndpoint_ResolveUnknownService(t *testing.T) {
	resolved, err := testPartitions.EndpointFor("unknown-service", "us-region-1",
		ResolveUnknownServiceOption)

	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}

	if e, a := "https://unknown-service.us-region-1.amazonaws.com", resolved.URL; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "us-region-1", resolved.SigningRegion; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "unknown-service", resolved.SigningName; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if !resolved.SigningNameDerived {
		t.Errorf("expect the signing name to be derived")
	}
}

func TestResolveEndpoint_UnknownMatchedRegion(t *testing.T) {
	resolved, err := testPartitions.EndpointFor("s3", "us-region-1")

	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}
	if e, a := "https://s3.us-region-1.amazonaws.com", resolved.URL; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "us-region-1", resolved.SigningRegion; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "s3", resolved.SigningName; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
}

func TestResolveEndpoint_UnknownRegion(t *testing.T) {
	resolved, err := testPartitions.EndpointFor("s3", "unknownregion")

	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}
	if e, a := "https://s3.unknownregion.amazonaws.com", resolved.URL; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "unknownregion", resolved.SigningRegion; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "s3", resolved.SigningName; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
}

func TestResolveEndpoint_StrictPartitionUnknownEndpoint(t *testing.T) {
	_, err := testPartitions[0].EndpointFor("s3", "unknownregion", StrictMatchingOption)

	if err == nil {
		t.Errorf("expect error, got none")
	}

	_, ok := err.(UnknownEndpointError)
	if !ok {
		t.Errorf("expect error to be UnknownEndpointError")
	}
}

func TestResolveEndpoint_StrictPartitionsUnknownEndpoint(t *testing.T) {
	_, err := testPartitions.EndpointFor("s3", "us-region-1", StrictMatchingOption)

	if err == nil {
		t.Errorf("expect error, got none")
	}

	_, ok := err.(UnknownEndpointError)
	if !ok {
		t.Errorf("expect error to be UnknownEndpointError")
	}
}

func TestResolveEndpoint_NotRegionalized(t *testing.T) {
	resolved, err := testPartitions.EndpointFor("globalService", "us-west-2")

	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}
	if e, a := "https://globalService.amazonaws.com", resolved.URL; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "us-east-1", resolved.SigningRegion; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "globalService", resolved.SigningName; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if !resolved.SigningNameDerived {
		t.Errorf("expect the signing name to be derived")
	}
}

func TestResolveEndpoint_AwsGlobal(t *testing.T) {
	resolved, err := testPartitions.EndpointFor("globalService", "aws-global")

	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}
	if e, a := "https://globalService.amazonaws.com", resolved.URL; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "us-east-1", resolved.SigningRegion; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "globalService", resolved.SigningName; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if !resolved.SigningNameDerived {
		t.Errorf("expect the signing name to be derived")
	}
}

func TestEndpointFor_RegionalFlag(t *testing.T) {
	// AwsPartition resolver for STS regional endpoints in AWS Partition
	resolver := AwsPartition()

	cases := map[string]struct {
		service, region                                     string
		regional                                            bool
		ExpectURL, ExpectSigningMethod, ExpectSigningRegion string
		ExpectSigningNameDerived                            bool
	}{
		"acm/ap-northeast-1/regional": {
			service:                  "acm",
			region:                   "ap-northeast-1",
			regional:                 true,
			ExpectURL:                "https://acm.ap-northeast-1.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "ap-northeast-1",
		},
		"acm/ap-northeast-1/legacy": {
			service:                  "acm",
			region:                   "ap-northeast-1",
			regional:                 false,
			ExpectURL:                "https://acm.ap-northeast-1.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "ap-northeast-1",
		},
	}

	for name, c := range cases {
		t.Run(name, func(t *testing.T) {
			var optionSlice []func(o *Options)
			optionSlice = append(optionSlice, func(o *Options) {
				if c.regional {
					o.STSRegionalEndpoint = RegionalSTSEndpoint
				}
			})

			actual, err := resolver.EndpointFor(c.service, c.region, optionSlice...)
			if err != nil {
				t.Fatalf("failed to resolve endpoint, %v", err)
			}

			if e, a := c.ExpectURL, actual.URL; e != a {
				t.Errorf("expect %v, got %v", e, a)
			}

			if e, a := c.ExpectSigningMethod, actual.SigningMethod; e != a {
				t.Errorf("expect %v, got %v", e, a)
			}

			if e, a := c.ExpectSigningNameDerived, actual.SigningNameDerived; e != a {
				t.Errorf("expect %v, got %v", e, a)
			}

			if e, a := c.ExpectSigningRegion, actual.SigningRegion; e != a {
				t.Errorf("expect %v, got %v", e, a)
			}

		})
	}
}

func TestEndpointFor_EmptyRegion(t *testing.T) {
	// skip this test for partitions outside `aws` partition
	if DefaultPartitions()[0].id != "aws" {
		t.Skip()
	}

	cases := map[string]struct {
		Service    string
		Region     string
		RealRegion string
		ExpectErr  string
	}{
		// Legacy services that previous accepted empty region
		"budgets":       {Service: "budgets", RealRegion: "aws-global"},
		"ce":            {Service: "ce", RealRegion: "aws-global"},
		"chime":         {Service: "chime", RealRegion: "aws-global"},
		"ec2metadata":   {Service: "ec2metadata", RealRegion: "aws-global"},
		"iam":           {Service: "iam", RealRegion: "aws-global"},
		"importexport":  {Service: "importexport", RealRegion: "aws-global"},
		"organizations": {Service: "organizations", RealRegion: "aws-global"},
		"route53":       {Service: "route53", RealRegion: "aws-global"},
		"sts":           {Service: "sts", RealRegion: "aws-global"},
		"support":       {Service: "support", RealRegion: "aws-global"},
		"waf":           {Service: "waf", RealRegion: "aws-global"},

		// Other services
		"s3":           {Service: "s3", Region: "us-east-1", RealRegion: "us-east-1"},
		"s3 no region": {Service: "s3", ExpectErr: "could not resolve endpoint"},
	}

	for name, c := range cases {
		t.Run(name, func(t *testing.T) {
			actual, err := DefaultResolver().EndpointFor(c.Service, c.Region)
			if len(c.ExpectErr) != 0 {
				if e, a := c.ExpectErr, err.Error(); !strings.Contains(a, e) {
					t.Errorf("expect %q error in %q", e, a)
				}
				return
			}
			if err != nil {
				t.Fatalf("expect no error got, %v", err)
			}

			expect, err := DefaultResolver().EndpointFor(c.Service, c.RealRegion)
			if err != nil {
				t.Fatalf("failed to get endpoint for default resolver")
			}
			if e, a := expect.URL, actual.URL; e != a {
				t.Errorf("expect %v URL, got %v", e, a)
			}
			if e, a := expect.SigningRegion, actual.SigningRegion; e != a {
				t.Errorf("expect %v signing region, got %v", e, a)
			}

		})
	}
}

func TestRegionValidator(t *testing.T) {
	cases := []struct {
		Region string
		Valid  bool
	}{
		0: {
			Region: "us-east-1",
			Valid:  true,
		},
		1: {
			Region: "invalid.com",
			Valid:  false,
		},
		2: {
			Region: "@invalid.com/%23",
			Valid:  false,
		},
		3: {
			Region: "local",
			Valid:  true,
		},
		4: {
			Region: "9-west-1",
			Valid:  true,
		},
	}

	for i, tt := range cases {
		t.Run(strconv.Itoa(i), func(t *testing.T) {
			if e, a := tt.Valid, validateInputRegion(tt.Region); e != a {
				t.Errorf("expected %v, got %v", e, a)
			}
		})
	}
}

func TestResolveEndpoint_FipsAwsGlobal(t *testing.T) {
	resolved, err := AwsPartition().EndpointFor("route53", "fips-aws-global")

	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}
	if e, a := "https://route53-fips.amazonaws.com", resolved.URL; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "us-east-1", resolved.SigningRegion; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if e, a := "route53", resolved.SigningName; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
	if !resolved.SigningNameDerived {
		t.Errorf("expect the signing name to be derived")
	}
}

func TestEC2MetadataService(t *testing.T) {
	unmodelled := partition{
		ID:   "unmodelled",
		Name: "partition with unmodelled ec2metadata",
		Services: map[string]service{
			"foo": {
				Endpoints: serviceEndpoints{
					endpointKey{Region: "us-west-2"}: endpoint{
						Hostname:          "foo.us-west-2.amazonaws.com",
						Protocols:         []string{"http"},
						SignatureVersions: []string{"v4"},
					},
				},
			},
		},
		Regions: map[string]region{
			"us-west-2": {Description: "us-west-2 region"},
		},
	}

	modelled := partition{
		ID:   "modelled",
		Name: "partition with modelled ec2metadata",
		Services: map[string]service{
			"ec2metadata": {
				Endpoints: serviceEndpoints{
					endpointKey{Region: "us-west-2"}: endpoint{
						Hostname:          "custom.localhost/latest",
						Protocols:         []string{"http"},
						SignatureVersions: []string{"v4"},
					},
				},
			},
			"foo": {
				Endpoints: serviceEndpoints{
					endpointKey{Region: "us-west-2"}: endpoint{
						Hostname:          "foo.us-west-2.amazonaws.com",
						Protocols:         []string{"http"},
						SignatureVersions: []string{"v4"},
					},
				},
			},
		},
		Regions: map[string]region{
			"us-west-2": {Description: "us-west-2 region"},
		},
	}

	uServices := unmodelled.Partition().Services()

	if s, ok := uServices[Ec2metadataServiceID]; !ok {
		t.Errorf("expect ec2metadata to be present")
	} else {
		if regions := s.Regions(); len(regions) != 0 {
			t.Errorf("expect no regions for ec2metadata, got %v", len(regions))
		}
		if resolved, err := unmodelled.EndpointFor(Ec2metadataServiceID, "us-west-2"); err != nil {
			t.Errorf("expect no error, got %v", err)
		} else if e, a := ec2MetadataEndpointIPv4, resolved.URL; e != a {
			t.Errorf("expect %v, got %v", e, a)
		}
	}

	if s, ok := uServices["foo"]; !ok {
		t.Errorf("expect foo to be present")
	} else if regions := s.Regions(); len(regions) == 0 {
		t.Errorf("expect region endpoints for foo. got none")
	}

	mServices := modelled.Partition().Services()

	if s, ok := mServices[Ec2metadataServiceID]; !ok {
		t.Errorf("expect ec2metadata to be present")
	} else if regions := s.Regions(); len(regions) == 0 {
		t.Errorf("expect region for ec2metadata, got none")
	} else {
		if resolved, err := modelled.EndpointFor(Ec2metadataServiceID, "us-west-2"); err != nil {
			t.Errorf("expect no error, got %v", err)
		} else if e, a := "http://custom.localhost/latest", resolved.URL; e != a {
			t.Errorf("expect %v, got %v", e, a)
		}
	}

	if s, ok := mServices["foo"]; !ok {
		t.Errorf("expect foo to be present")
	} else if regions := s.Regions(); len(regions) == 0 {
		t.Errorf("expect region endpoints for foo, got none")
	}
}

func TestEndpointVariants(t *testing.T) {
	modelFile, err := os.Open(filepath.Join("testdata", "variants_model.json"))
	if err != nil {
		t.Fatal(err)
	}
	defer modelFile.Close()

	resolver, err := DecodeModel(modelFile)
	if err != nil {
		t.Fatal(err)
	}

	type testCase struct {
		Service   string `json:"service"`
		Region    string `json:"region"`
		FIPS      bool   `json:"FIPS"`
		DualStack bool   `json:"DualStack"`
		Endpoint  string `json:"Endpoint"`
	}

	casesBytes, err := ioutil.ReadFile(filepath.Join("testdata", "variants_cases.json"))
	if err != nil {
		t.Fatal(err)
	}

	var cases []testCase
	if err := json.Unmarshal(casesBytes, &cases); err != nil {
		panic(err)
	}

	for i, tt := range cases {
		t.Run(strconv.Itoa(i), func(t *testing.T) {
			options := Options{}

			if tt.FIPS {
				options.UseFIPSEndpoint = FIPSEndpointStateEnabled
			}
			if tt.DualStack {
				options.UseDualStackEndpoint = DualStackEndpointStateEnabled
			}

			resolvedEndpoint, err := resolver.EndpointFor(tt.Service, tt.Region, func(o *Options) {
				*o = options
			})
			if err != nil {
				t.Errorf("expect no error, got %v", err)
				return
			}

			parsed, err := url.Parse(resolvedEndpoint.URL)
			if err != nil {
				t.Errorf("expect no error, got %v", err)
				return
			}

			if e, a := parsed.Host, tt.Endpoint; e != a {
				t.Errorf("expect %v, got %v", e, a)
			}
		})
	}
}

func TestLogDeprecated(t *testing.T) {
	partitions := partitions{
		partition{
			ID: "aws",
			RegionRegex: regionRegex{
				Regexp: regexp.MustCompile("^(us|eu|ap|sa|ca)\\-\\w+\\-\\d+$"),
			},
			Defaults: map[defaultKey]endpoint{
				{}: {
					Hostname:  "foo.{region}.bar.tld",
					Protocols: []string{"https", "http"},
				},
				{
					Variant: fipsVariant,
				}: {
					Hostname: "foo-fips.{region}.bar.tld",
				},
			},
			Services: map[string]service{
				"service": {
					Endpoints: map[endpointKey]endpoint{
						{
							Region: "foo",
						}: {},
						{
							Region: "bar",
						}: {
							Deprecated: boxedTrue,
						},
						{
							Region:  "bar",
							Variant: fipsVariant,
						}: {
							Deprecated: boxedTrue,
						},
					},
				},
			},
		},
	}

	cases := []struct {
		Region      string
		Options     Options
		Expected    ResolvedEndpoint
		SetupLogger func() (Logger, func(*testing.T))
		WantErr     bool
	}{
		{
			Region: "foo",
			Expected: ResolvedEndpoint{
				URL:                "https://foo.foo.bar.tld",
				PartitionID:        "aws",
				SigningName:        "service",
				SigningRegion:      "foo",
				SigningMethod:      "v4",
				SigningNameDerived: true,
			},
		},
		{
			Region: "bar",
			Options: Options{
				LogDeprecated: true,
			},
			Expected: ResolvedEndpoint{
				URL:                "https://foo.bar.bar.tld",
				PartitionID:        "aws",
				SigningName:        "service",
				SigningRegion:      "bar",
				SigningMethod:      "v4",
				SigningNameDerived: true,
			},
		},
		{
			Region: "bar",
			Options: Options{
				LogDeprecated:   true,
				UseFIPSEndpoint: FIPSEndpointStateEnabled,
			},
			Expected: ResolvedEndpoint{
				URL:                "https://foo-fips.bar.bar.tld",
				PartitionID:        "aws",
				SigningName:        "service",
				SigningRegion:      "bar",
				SigningMethod:      "v4",
				SigningNameDerived: true,
			},
		},
		{
			Region: "bar",
			Options: Options{
				LogDeprecated: true,
			},
			SetupLogger: func() (Logger, func(*testing.T)) {
				buffer := bytes.NewBuffer(nil)
				logger := log.New(buffer, "", 0)
				return LoggerFunc(func(i ...interface{}) {
						logger.Println(i...)
					}), func(t *testing.T) {
						if e, a := "endpoint identifier \"bar\", url \"https://foo.bar.bar.tld\" marked as deprecated\n", buffer.String(); e != a {
							t.Errorf("expect %v, got %v", e, a)
						}
					}
			},
			Expected: ResolvedEndpoint{
				URL:                "https://foo.bar.bar.tld",
				PartitionID:        "aws",
				SigningName:        "service",
				SigningRegion:      "bar",
				SigningMethod:      "v4",
				SigningNameDerived: true,
			},
		},
		{
			Region: "bar",
			Options: Options{
				LogDeprecated:   true,
				UseFIPSEndpoint: FIPSEndpointStateEnabled,
			},
			SetupLogger: func() (Logger, func(*testing.T)) {
				buffer := bytes.NewBuffer(nil)
				logger := log.New(buffer, "", 0)
				return LoggerFunc(func(i ...interface{}) {
						logger.Println(i...)
					}), func(t *testing.T) {
						if e, a := "endpoint identifier \"bar\", url \"https://foo-fips.bar.bar.tld\" marked as deprecated\n", buffer.String(); e != a {
							t.Errorf("expect %v, got %v", e, a)
						}
					}
			},
			Expected: ResolvedEndpoint{
				URL:                "https://foo-fips.bar.bar.tld",
				PartitionID:        "aws",
				SigningName:        "service",
				SigningRegion:      "bar",
				SigningMethod:      "v4",
				SigningNameDerived: true,
			},
		},
	}

	for i, tt := range cases {
		t.Run(strconv.Itoa(i), func(t *testing.T) {
			var verifyLog func(*testing.T)
			if tt.SetupLogger != nil {
				tt.Options.Logger, verifyLog = tt.SetupLogger()
			}

			endpoint, err := partitions.EndpointFor("service", tt.Region, func(options *Options) {
				*options = tt.Options
			})
			if (err != nil) != tt.WantErr {
				t.Errorf("WantErr(%v), got error %v", tt.WantErr, err)
			}

			if !reflect.DeepEqual(tt.Expected, endpoint) {
				t.Errorf("expect %v, got %v", tt.Expected, endpoint)
			}

			if verifyLog != nil {
				verifyLog(t)
			}
		})
	}
}

func TestPartitionVariantMerging(t *testing.T) {
	partition := partition{
		ID:        "aws-iso",
		Name:      "AWS ISO (US)",
		DNSSuffix: "c2s.ic.gov",
		RegionRegex: regionRegex{
			Regexp: func() *regexp.Regexp {
				reg, _ := regexp.Compile("^us\\-iso\\-\\w+\\-\\d+$")
				return reg
			}(),
		},
		Defaults: endpointDefaults{
			{}: {
				Hostname:          "{service}.{region}.{dnsSuffix}",
				Protocols:         []string{"https"},
				SignatureVersions: []string{"v4"},
			},
			{Variant: dualStackVariant}: {
				DNSSuffix:         "dualstack.foo.bar",
				Hostname:          "{service}.{region}.{dnsSuffix}",
				Protocols:         []string{"https"},
				SignatureVersions: []string{"v4"},
			},
		},
		Regions: regions{
			"us-iso-east-1": region{
				Description: "US ISO East",
			},
			"us-iso-west-1": region{
				Description: "US ISO WEST",
			},
		},
		Services: services{
			"service1": {},
			"service2": {
				Defaults: map[defaultKey]endpoint{
					{}: {
						CredentialScope: credentialScope{
							Service: "service-two",
						},
					},
					{Variant: fipsVariant}: {
						Hostname:  "{service}-fips.{region}.{dnsSuffix}",
						DNSSuffix: "foo.bar",
						CredentialScope: credentialScope{
							Service: "service-two",
						},
					},
				},
			},
		},
	}

	cases := []struct {
		Service          string
		Region           string
		Options          Options
		WantErr          bool
		ExpectedEndpoint ResolvedEndpoint
	}{
		{
			Service: "service1",
			Region:  "us-iso-east-1",
			Options: Options{
				UseFIPSEndpoint: FIPSEndpointStateEnabled,
			},
			WantErr: true,
		},
		{
			Service: "service1",
			Region:  "us-iso-east-1",
			Options: Options{
				UseDualStackEndpoint: DualStackEndpointStateEnabled,
			},
			ExpectedEndpoint: ResolvedEndpoint{
				URL:                "https://service1.us-iso-east-1.dualstack.foo.bar",
				PartitionID:        "aws-iso",
				SigningRegion:      "us-iso-east-1",
				SigningName:        "service1",
				SigningNameDerived: true,
				SigningMethod:      "v4",
			},
		},
		{
			Service: "service1",
			Region:  "us-iso-east-1",
			ExpectedEndpoint: ResolvedEndpoint{
				URL:                "https://service1.us-iso-east-1.c2s.ic.gov",
				PartitionID:        "aws-iso",
				SigningRegion:      "us-iso-east-1",
				SigningName:        "service1",
				SigningNameDerived: true,
				SigningMethod:      "v4",
			},
		},
		{
			Service: "service2",
			Region:  "us-iso-east-1",
			Options: Options{
				UseFIPSEndpoint: FIPSEndpointStateEnabled,
			},
			ExpectedEndpoint: ResolvedEndpoint{
				URL:           "https://service2-fips.us-iso-east-1.foo.bar",
				PartitionID:   "aws-iso",
				SigningRegion: "us-iso-east-1",
				SigningName:   "service-two",
				SigningMethod: "v4",
			},
		},
		{
			Service: "service2",
			Region:  "us-iso-east-1",
			Options: Options{
				UseDualStackEndpoint: DualStackEndpointStateEnabled,
			},
			ExpectedEndpoint: ResolvedEndpoint{
				URL:           "https://service2.us-iso-east-1.dualstack.foo.bar",
				PartitionID:   "aws-iso",
				SigningRegion: "us-iso-east-1",
				SigningName:   "service-two",
				SigningMethod: "v4",
			},
		},
	}

	for i, tt := range cases {
		t.Run(strconv.Itoa(i), func(t *testing.T) {
			resolved, err := partition.EndpointFor(tt.Service, tt.Region, func(options *Options) {
				*options = tt.Options
			})
			if (err != nil) != tt.WantErr {
				t.Errorf("WantErr(%v) got Err(%v)", tt.WantErr, err)
				return
			}
			if tt.WantErr {
				return
			}
			if e, a := tt.ExpectedEndpoint, resolved; !reflect.DeepEqual(e, a) {
				t.Errorf("expect %v, got %v", e, a)
			}
		})
	}

}
