//go:build go1.9
// +build go1.9

package endpoints

import (
	"strings"
	"testing"
)

func TestDecodeEndpoints_V3(t *testing.T) {
	const v3Doc = `{
  "partitions": [
    {
      "defaults": {
        "hostname": "{service}.{region}.{dnsSuffix}",
        "protocols": [
          "https"
        ],
        "signatureVersions": [
          "v4"
        ],
        "variants": [
          {
            "hostname": "{service}-fips.{region}.{dnsSuffix}",
            "tags": [
              "fips"
            ]
          },
          {
            "dnsSuffix": "api.aws",
            "hostname": "{service}.{region}.{dnsSuffix}",
            "tags": [
              "dualstack"
            ]
          },
          {
            "dnsSuffix": "api.aws",
            "hostname": "{service}-fips.{region}.{dnsSuffix}",
            "tags": [
              "dualstack",
              "fips"
            ]
          }
        ]
      },
      "dnsSuffix": "amazonaws.com",
      "partition": "aws",
      "regionRegex": "^(us|eu|ap|sa|ca|me|af)\\-\\w+\\-\\d+$",
      "regions": {
        "us-east-1": {
          "description": "US East (N. Virginia)"
        },
        "us-west-2": {
          "description": "US West (Oregon)"
        }
      },
      "services": {
        "dynamodb": {
          "defaults": {
            "protocols": [
              "http",
              "https"
            ]
          },
          "endpoints": {
            "us-west-2": {
              "variants": [
                {
                  "tags": [
                    "fips"
                  ]
                },
                {
                  "hostname": "fips.dynamodb.us-west-2.api.aws",
                  "tags": [
                    "fips",
                    "dualstack"
                  ]
                },
                {
                  "hostname": "dynamodb.us-west-2.api.aws",
                  "tags": [
                    "dualstack"
                  ]
                }
              ]
            },
            "us-west-2-fips": {
              "hostname": "dynamodb-fips.us-west-2.amazonaws.com",
              "credentialScope": {
                "region": "us-west-2"
              },
              "deprecated": true
            }
          }
        },
        "ec2": {
          "defaults": {
            "hostname": "api.ec2.{region}.{dnsSuffix}",
            "protocols": [
              "http",
              "https"
            ],
            "variants": [
              {
                "dnsSuffix": "amazonaws.com",
                "hostname": "api.ec2-fips.{region}.{dnsSuffix}",
                "tags": [
                  "fips"
                ]
              },
              {
                "dnsSuffix": "api.aws",
                "hostname": "api.ec2.{region}.{dnsSuffix}",
                "tags": [
                  "dualstack"
                ]
              }
            ]
          },
          "endpoints": {
            "us-west-2": {
              "credentialScope": {
                "region": "us-west-2"
              },
              "hostname": "ec2.us-west-2.amazonaws.com",
              "variants": [
                {
                  "hostname": "ec2-fips.us-west-2.amazonaws.com",
                  "tags": [
                    "fips"
                  ]
                },
                {
                  "hostname": "ec2.us-west-2.api.aws",
                  "tags": [
                    "dualstack"
                  ]
                }
              ]
            },
            "fips-us-west-2": {
              "hostname": "ec2-fips.us-west-2.amazonaws.com",
              "credentialScope": {
                "region": "us-west-2"
              },
              "deprecated": true
            }
          }
        },
        "route53": {
          "endpoints": {
            "aws-global": {
              "credentialScope": {
                "region": "us-east-1"
              },
              "hostname": "route53.amazonaws.com",
              "variants": [
                {
                  "hostname": "route53-fips.amazonaws.com",
                  "tags": [
                    "fips"
                  ]
                },
                {
                  "hostname": "route53.global.api.aws",
                  "tags": [
                    "dualstack"
                  ]
                }
              ]
            }
          },
          "isRegionalized": false,
          "partitionEndpoint": "aws-global"
        },
        "s3": {
          "defaults": {
            "protocols": [
              "http",
              "https"
            ],
            "signatureVersions": [
              "s3v4"
            ],
            "variants": [
              {
                "dnsSuffix": "amazonaws.com",
                "hostname": "s3-fips.{region}.{dnsSuffix}",
                "tags": [
                  "fips"
                ]
              },
              {
                "dnsSuffix": "amazonaws.com",
                "hostname": "s3.dualstack.{region}.{dnsSuffix}",
                "tags": ["dualstack"]
              }
            ]
          },
          "endpoints": {
            "us-west-2": {
              "hostname": "s3.api.us-west-2.amazonaws.com",
              "signatureVersions": [
                "s3",
                "s3v4"
              ],
              "variants": [
                {
                  "hostname": "s3-fips.api.us-west-2.amazonaws.com",
                  "tags": [
                    "fips"
                  ]
                },
                {
                  "hostname": "s3.api.dualstack.us-west-2.amazonaws.com",
                  "tags": ["dualstack"]
                }
              ]
            }
          }
        }
      }
    },
    {
      "defaults": {
        "hostname": "{service}.{region}.{dnsSuffix}",
        "protocols": [
          "https"
        ],
        "signatureVersions": [
          "v4"
        ]
      },
      "dnsSuffix": "c2s.ic.gov",
      "partition": "aws-iso",
      "regionRegex": "^us\\-iso\\-\\w+\\-\\d+$",
      "regions": {
        "us-iso-east-1": {
          "description": "US ISO East"
        }
      },
      "services": {
        "ec2": {
          "endpoints": {
            "us-iso-east-1": {}
          }
        }
      }
    }
  ],
  "version": 3
}`

	resolver, err := DecodeModel(strings.NewReader(v3Doc))
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}

	endpoint, err := resolver.EndpointFor("ec2", "us-west-2")
	if err != nil {
		t.Fatalf("failed to resolve endpoint, %v", err)
	}

	if a, e := endpoint.URL, "https://ec2.us-west-2.amazonaws.com"; a != e {
		t.Errorf("expected %q URL got %q", e, a)
	}

	p := resolver.(partitions)[0]

	resolved, err := p.EndpointFor("s3", "us-west-2", func(options *Options) {
		options.UseDualStackEndpoint = DualStackEndpointStateEnabled
	})
	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}

	assertEndpoint(t, resolved, "https://s3.api.dualstack.us-west-2.amazonaws.com", "s3", "us-west-2")
}

func assertEndpoint(t *testing.T, endpoint ResolvedEndpoint, expectedURL, expectedSigningName, expectedSigningRegion string) {
	t.Helper()

	if e, a := expectedURL, endpoint.URL; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}

	if e, a := expectedSigningName, endpoint.SigningName; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}

	if e, a := expectedSigningRegion, endpoint.SigningRegion; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
}

func TestDecodeEndpoints_NoPartitions(t *testing.T) {
	const doc = `{ "version": 3 }`

	resolver, err := DecodeModel(strings.NewReader(doc))
	if err == nil {
		t.Fatalf("expected error")
	}

	if resolver != nil {
		t.Errorf("expect resolver to be nil")
	}
}

func TestDecodeEndpoints_UnsupportedVersion(t *testing.T) {
	const doc = `{ "version": 2 }`

	resolver, err := DecodeModel(strings.NewReader(doc))
	if err == nil {
		t.Fatalf("expected error decoding model")
	}

	if resolver != nil {
		t.Errorf("expect resolver to be nil")
	}
}

func TestDecodeModelOptionsSet(t *testing.T) {
	var actual DecodeModelOptions
	actual.Set(func(o *DecodeModelOptions) {
		o.SkipCustomizations = true
	})

	expect := DecodeModelOptions{
		SkipCustomizations: true,
	}

	if actual != expect {
		t.Errorf("expect %v options got %v", expect, actual)
	}
}

func TestCustFixAppAutoscalingChina(t *testing.T) {
	const doc = `
{
  "version": 3,
  "partitions": [{
    "defaults" : {
      "hostname" : "{service}.{region}.{dnsSuffix}",
      "protocols" : [ "https" ],
      "signatureVersions" : [ "v4" ]
    },
    "dnsSuffix" : "amazonaws.com.cn",
    "partition" : "aws-cn",
    "partitionName" : "AWS China",
    "regionRegex" : "^cn\\-\\w+\\-\\d+$",
    "regions" : {
      "cn-north-1" : {
        "description" : "China (Beijing)"
      },
      "cn-northwest-1" : {
        "description" : "China (Ningxia)"
      }
    },
    "services" : {
      "application-autoscaling" : {
        "defaults" : {
          "credentialScope" : {
            "service" : "application-autoscaling"
          },
          "hostname" : "autoscaling.{region}.amazonaws.com",
          "protocols" : [ "http", "https" ]
        },
        "endpoints" : {
          "cn-north-1" : { },
          "cn-northwest-1" : { }
        }
      }
	}
  }]
}`

	resolver, err := DecodeModel(strings.NewReader(doc))
	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}

	endpoint, err := resolver.EndpointFor(
		"application-autoscaling", "cn-northwest-1",
	)
	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}

	if e, a := `https://autoscaling.cn-northwest-1.amazonaws.com.cn`, endpoint.URL; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
}

func TestCustFixAppAutoscalingUsGov(t *testing.T) {
	const doc = `
{
  "version": 3,
  "partitions": [{
    "defaults" : {
      "hostname" : "{service}.{region}.{dnsSuffix}",
      "protocols" : [ "https" ],
      "signatureVersions" : [ "v4" ]
    },
    "dnsSuffix" : "amazonaws.com",
    "partition" : "aws-us-gov",
    "partitionName" : "AWS GovCloud (US)",
    "regionRegex" : "^us\\-gov\\-\\w+\\-\\d+$",
    "regions" : {
      "us-gov-east-1" : {
        "description" : "AWS GovCloud (US-East)"
      },
      "us-gov-west-1" : {
        "description" : "AWS GovCloud (US)"
      }
    },
    "services" : {
      "application-autoscaling" : {
        "endpoints" : {
          "us-gov-east-1" : { },
          "us-gov-west-1" : { }
        }
      }
	}
  }]
}`

	resolver, err := DecodeModel(strings.NewReader(doc))
	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}

	endpoint, err := resolver.EndpointFor(
		"application-autoscaling", "us-gov-west-1",
	)
	if err != nil {
		t.Fatalf("expect no error, got %v", err)
	}

	if e, a := `https://autoscaling.us-gov-west-1.amazonaws.com`, endpoint.URL; e != a {
		t.Errorf("expect %v, got %v", e, a)
	}
}
