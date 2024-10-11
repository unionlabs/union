//go:build go1.9
// +build go1.9

package endpoints

import "regexp"

type LoggerFunc func(...interface{})

func (l LoggerFunc) Log(i ...interface{}) {
	l(i...)
}

var testPartitions = partitions{
	partition{
		ID:        "part-id",
		Name:      "partitionName",
		DNSSuffix: "amazonaws.com",
		RegionRegex: regionRegex{
			Regexp: func() *regexp.Regexp {
				reg, _ := regexp.Compile("^(us|eu|ap|sa|ca)\\-\\w+\\-\\d+$")
				return reg
			}(),
		},
		Defaults: endpointDefaults{
			{}: {
				Hostname:          "{service}.{region}.{dnsSuffix}",
				Protocols:         []string{"https"},
				SignatureVersions: []string{"v4"},
			},
		},
		Regions: regions{
			"us-east-1": region{
				Description: "region description",
			},
			"us-west-2": region{},
		},
		Services: services{
			"s3": service{},
			"service1": service{
				Defaults: endpointDefaults{
					{}: {
						CredentialScope: credentialScope{
							Service: "service1",
						},
					},
				},
				Endpoints: serviceEndpoints{
					{Region: "us-east-1"}: {},
					{Region: "us-west-2"}: {},
					{
						Region:  "us-west-2",
						Variant: dualStackVariant,
					}: {
						Hostname: "{service}.dualstack.{region}.{dnsSuffix}",
					},
				},
			},
			"service2": service{
				Defaults: endpointDefaults{
					{}: {
						CredentialScope: credentialScope{
							Service: "service2",
						},
					},
				},
			},
			"httpService": service{
				Defaults: endpointDefaults{
					{}: {
						Protocols: []string{"http"},
					},
				},
			},
			"globalService": service{
				IsRegionalized:    boxedFalse,
				PartitionEndpoint: "aws-global",
				Endpoints: serviceEndpoints{
					{
						Region: "aws-global",
					}: {
						CredentialScope: credentialScope{
							Region: "us-east-1",
						},
						Hostname: "globalService.amazonaws.com",
					},
					{
						Region: "fips-aws-global",
					}: {
						CredentialScope: credentialScope{
							Region: "us-east-1",
						},
						Hostname: "globalService-fips.amazonaws.com",
					},
				},
			},
		},
	},
}
