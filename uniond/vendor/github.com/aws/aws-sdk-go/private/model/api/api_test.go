//go:build go1.8 && codegen
// +build go1.8,codegen

package api

import (
	"strings"
	"testing"
)

func TestAPI_StructName(t *testing.T) {
	origAliases := serviceAliaseNames
	defer func() { serviceAliaseNames = origAliases }()

	cases := map[string]struct {
		Aliases    map[string]string
		Metadata   Metadata
		StructName string
	}{
		"FullName": {
			Metadata: Metadata{
				ServiceFullName: "Amazon Service Name-100",
			},
			StructName: "ServiceName100",
		},
		"Abbreviation": {
			Metadata: Metadata{
				ServiceFullName:     "Amazon Service Name-100",
				ServiceAbbreviation: "AWS SN100",
			},
			StructName: "SN100",
		},
		"Lowercase Name": {
			Metadata: Metadata{
				EndpointPrefix:      "other",
				ServiceFullName:     "AWS Lowercase service",
				ServiceAbbreviation: "lowercase",
			},
			StructName: "Lowercase",
		},
		"Lowercase Name Mixed": {
			Metadata: Metadata{
				EndpointPrefix:      "other",
				ServiceFullName:     "AWS Lowercase service",
				ServiceAbbreviation: "lowercase name Goes heRe",
			},
			StructName: "LowercaseNameGoesHeRe",
		},
		"Alias": {
			Aliases: map[string]string{
				"elasticloadbalancing": "ELB",
			},
			Metadata: Metadata{
				ServiceFullName: "Elastic Load Balancing",
			},
			StructName: "ELB",
		},
	}

	for k, c := range cases {
		t.Run(k, func(t *testing.T) {
			serviceAliaseNames = c.Aliases

			a := API{
				Metadata: c.Metadata,
			}

			a.Setup()

			if e, o := c.StructName, a.StructName(); e != o {
				t.Errorf("expect %v structName, got %v", e, o)
			}
		})
	}
}

func TestAPI_Setup_documentShapes(t *testing.T) {
	api := API{
		Shapes: map[string]*Shape{
			"Document": {
				Type:     "structure",
				Document: true,
			},
		},
	}

	err := api.Setup()
	if err == nil {
		t.Fatalf("expect error, but got nil")
	}
	expect := "model contains document shapes"
	if !strings.Contains(err.Error(), expect) {
		t.Errorf("expect %s, got %v", expect, err)
	}
}
