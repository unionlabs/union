//go:build go1.9
// +build go1.9

package endpoints

import (
	"testing"
)

// ***************************************************************************
// All endpoint metadata is sourced from the testdata/endpoints.json file at
// test startup. Not the live endpoints model file. Update the testdata file
// for the tests to use the latest live model.
// ***************************************************************************

func TestEndpointFor_STSRegionalFlag(t *testing.T) {
	// resolver for STS regional endpoints model
	var resolver Resolver = AwsPartition()

	cases := map[string]struct {
		service, region                                     string
		regional                                            bool
		ExpectURL, ExpectSigningMethod, ExpectSigningRegion string
		ExpectSigningNameDerived                            bool
	}{
		// STS Endpoints resolver tests :
		"sts/us-west-2/regional": {
			service:                  "sts",
			region:                   "us-west-2",
			regional:                 true,
			ExpectURL:                "https://sts.us-west-2.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "us-west-2",
		},
		"sts/us-west-2/legacy": {
			service:                  "sts",
			region:                   "us-west-2",
			regional:                 false,
			ExpectURL:                "https://sts.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "us-east-1",
		},
		"sts/ap-east-1/regional": {
			service:                  "sts",
			region:                   "ap-east-1",
			regional:                 true,
			ExpectURL:                "https://sts.ap-east-1.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "ap-east-1",
		},
		"sts/ap-east-1/legacy": {
			service:                  "sts",
			region:                   "ap-east-1",
			regional:                 false,
			ExpectURL:                "https://sts.ap-east-1.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "ap-east-1",
		},
		"sts/us-west-2-fips/regional": {
			service:                  "sts",
			region:                   "us-west-2-fips",
			regional:                 true,
			ExpectURL:                "https://sts-fips.us-west-2.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "us-west-2",
		},
		"sts/us-west-2-fips/legacy": {
			service:                  "sts",
			region:                   "us-west-2-fips",
			regional:                 false,
			ExpectURL:                "https://sts-fips.us-west-2.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "us-west-2",
		},
		"sts/aws-global/regional": {
			service:                  "sts",
			region:                   "aws-global",
			regional:                 true,
			ExpectURL:                "https://sts.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "us-east-1",
		},
		"sts/aws-global/legacy": {
			service:                  "sts",
			region:                   "aws-global",
			regional:                 false,
			ExpectURL:                "https://sts.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "us-east-1",
		},
		"sts/ap-south-1/regional": {
			service:                  "sts",
			region:                   "ap-south-1",
			regional:                 true,
			ExpectURL:                "https://sts.ap-south-1.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "ap-south-1",
		},
		"sts/ap-south-1/legacy": {
			service:                  "sts",
			region:                   "ap-south-1",
			regional:                 false,
			ExpectURL:                "https://sts.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "us-east-1",
		},
		"sts/ap-northeast-1/regional": {
			service:                  "sts",
			region:                   "ap-northeast-1",
			regional:                 true,
			ExpectURL:                "https://sts.ap-northeast-1.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "ap-northeast-1",
		},
		"sts/ap-northeast-1/legacy": {
			service:                  "sts",
			region:                   "ap-northeast-1",
			regional:                 false,
			ExpectURL:                "https://sts.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "us-east-1",
		},
		"sts/ap-southeast-1/regional": {
			service:                  "sts",
			region:                   "ap-southeast-1",
			regional:                 true,
			ExpectURL:                "https://sts.ap-southeast-1.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "ap-southeast-1",
		},
		"sts/ap-southeast-1/legacy": {
			service:                  "sts",
			region:                   "ap-southeast-1",
			regional:                 false,
			ExpectURL:                "https://sts.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "us-east-1",
		},
		"sts/ca-central-1/regional": {
			service:                  "sts",
			region:                   "ca-central-1",
			regional:                 true,
			ExpectURL:                "https://sts.ca-central-1.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "ca-central-1",
		},
		"sts/ca-central-1/legacy": {
			service:                  "sts",
			region:                   "ca-central-1",
			regional:                 false,
			ExpectURL:                "https://sts.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "us-east-1",
		},
		"sts/eu-central-1/regional": {
			service:                  "sts",
			region:                   "eu-central-1",
			regional:                 true,
			ExpectURL:                "https://sts.eu-central-1.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "eu-central-1",
		},
		"sts/eu-central-1/legacy": {
			service:                  "sts",
			region:                   "eu-central-1",
			regional:                 false,
			ExpectURL:                "https://sts.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "us-east-1",
		},
		"sts/eu-north-1/regional": {
			service:                  "sts",
			region:                   "eu-north-1",
			regional:                 true,
			ExpectURL:                "https://sts.eu-north-1.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "eu-north-1",
		},
		"sts/eu-north-1/legacy": {
			service:                  "sts",
			region:                   "eu-north-1",
			regional:                 false,
			ExpectURL:                "https://sts.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "us-east-1",
		},
		"sts/eu-west-1/regional": {
			service:                  "sts",
			region:                   "eu-west-1",
			regional:                 true,
			ExpectURL:                "https://sts.eu-west-1.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "eu-west-1",
		},
		"sts/eu-west-1/legacy": {
			service:                  "sts",
			region:                   "eu-west-1",
			regional:                 false,
			ExpectURL:                "https://sts.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "us-east-1",
		},
		"sts/eu-west-2/regional": {
			service:                  "sts",
			region:                   "eu-west-2",
			regional:                 true,
			ExpectURL:                "https://sts.eu-west-2.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "eu-west-2",
		},
		"sts/eu-west-2/legacy": {
			service:                  "sts",
			region:                   "eu-west-2",
			regional:                 false,
			ExpectURL:                "https://sts.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "us-east-1",
		},
		"sts/eu-west-3/regional": {
			service:                  "sts",
			region:                   "eu-west-3",
			regional:                 true,
			ExpectURL:                "https://sts.eu-west-3.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "eu-west-3",
		},
		"sts/eu-west-3/legacy": {
			service:                  "sts",
			region:                   "eu-west-3",
			regional:                 false,
			ExpectURL:                "https://sts.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "us-east-1",
		},
		"sts/sa-east-1/regional": {
			service:                  "sts",
			region:                   "sa-east-1",
			regional:                 true,
			ExpectURL:                "https://sts.sa-east-1.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "sa-east-1",
		},
		"sts/sa-east-1/legacy": {
			service:                  "sts",
			region:                   "sa-east-1",
			regional:                 false,
			ExpectURL:                "https://sts.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "us-east-1",
		},
		"sts/us-east-1/regional": {
			service:                  "sts",
			region:                   "us-east-1",
			regional:                 true,
			ExpectURL:                "https://sts.us-east-1.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "us-east-1",
		},
		"sts/us-east-1/legacy": {
			service:                  "sts",
			region:                   "us-east-1",
			regional:                 false,
			ExpectURL:                "https://sts.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "us-east-1",
		},
		"sts/us-east-2/regional": {
			service:                  "sts",
			region:                   "us-east-2",
			regional:                 true,
			ExpectURL:                "https://sts.us-east-2.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "us-east-2",
		},
		"sts/us-east-2/legacy": {
			service:                  "sts",
			region:                   "us-east-2",
			regional:                 false,
			ExpectURL:                "https://sts.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "us-east-1",
		},
		"sts/us-west-1/regional": {
			service:                  "sts",
			region:                   "us-west-1",
			regional:                 true,
			ExpectURL:                "https://sts.us-west-1.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "us-west-1",
		},
		"sts/us-west-1/legacy": {
			service:                  "sts",
			region:                   "us-west-1",
			regional:                 false,
			ExpectURL:                "https://sts.amazonaws.com",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "us-east-1",
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

func TestEndpointFor_S3UsEast1RegionalFlag(t *testing.T) {
	// resolver for STS regional endpoints model
	var resolver Resolver = AwsPartition()

	cases := map[string]struct {
		service, region     string
		regional            S3UsEast1RegionalEndpoint
		ExpectURL           string
		ExpectSigningRegion string
	}{
		// S3 Endpoints resolver tests:
		"s3/us-east-1/regional": {
			service:             "s3",
			region:              "us-east-1",
			regional:            RegionalS3UsEast1Endpoint,
			ExpectURL:           "https://s3.us-east-1.amazonaws.com",
			ExpectSigningRegion: "us-east-1",
		},
		"s3/us-east-1/legacy": {
			service:             "s3",
			region:              "us-east-1",
			ExpectURL:           "https://s3.amazonaws.com",
			ExpectSigningRegion: "us-east-1",
		},
		"s3/us-west-1/regional": {
			service:             "s3",
			region:              "us-west-1",
			regional:            RegionalS3UsEast1Endpoint,
			ExpectURL:           "https://s3.us-west-1.amazonaws.com",
			ExpectSigningRegion: "us-west-1",
		},
		"s3/us-west-1/legacy": {
			service:             "s3",
			region:              "us-west-1",
			regional:            RegionalS3UsEast1Endpoint,
			ExpectURL:           "https://s3.us-west-1.amazonaws.com",
			ExpectSigningRegion: "us-west-1",
		},
	}

	for name, c := range cases {
		t.Run(name, func(t *testing.T) {
			var optionSlice []func(o *Options)
			optionSlice = append(optionSlice, func(o *Options) {
				o.S3UsEast1RegionalEndpoint = c.regional
			})

			actual, err := resolver.EndpointFor(c.service, c.region, optionSlice...)
			if err != nil {
				t.Fatalf("failed to resolve endpoint, %v", err)
			}

			if e, a := c.ExpectURL, actual.URL; e != a {
				t.Errorf("expect %v, got %v", e, a)
			}

			if e, a := c.ExpectSigningRegion, actual.SigningRegion; e != a {
				t.Errorf("expect %v, got %v", e, a)
			}

		})
	}
}

func TestSTSRegionalEndpoint_CNPartition(t *testing.T) {
	// resolver for STS regional endpoints model
	var resolver Resolver = AwsCnPartition()

	cases := map[string]struct {
		service, region                                     string
		regional                                            bool
		ExpectURL, ExpectSigningMethod, ExpectSigningRegion string
		ExpectSigningNameDerived                            bool
	}{
		"sts/cn-north-1/regional": {
			service:                  "sts",
			region:                   "cn-north-1",
			regional:                 true,
			ExpectURL:                "https://sts.cn-north-1.amazonaws.com.cn",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "cn-north-1",
		},
		"sts/cn-north-1/legacy": {
			service:                  "sts",
			region:                   "cn-north-1",
			regional:                 false,
			ExpectURL:                "https://sts.cn-north-1.amazonaws.com.cn",
			ExpectSigningMethod:      "v4",
			ExpectSigningNameDerived: true,
			ExpectSigningRegion:      "cn-north-1",
		},
	}

	for name, c := range cases {
		var optionSlice []func(o *Options)
		t.Run(name, func(t *testing.T) {
			if c.regional {
				optionSlice = append(optionSlice, STSRegionalEndpointOption)
			}
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
