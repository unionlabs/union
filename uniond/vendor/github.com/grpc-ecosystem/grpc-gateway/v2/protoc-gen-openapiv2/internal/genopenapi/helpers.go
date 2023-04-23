//go:build go1.12
// +build go1.12

package genopenapi

import (
	"strings"

	"golang.org/x/text/cases"
	"golang.org/x/text/language"
)

func fieldName(k string) string {
	return strings.ReplaceAll(cases.Title(language.AmericanEnglish).String(k), "-", "_")
}

// this method will filter the same fields and return the unique one
func getUniqueFields(schemaFieldsRequired []string, fieldsRequired []string) []string {
	var unique []string
	var index *int

	for j, schemaFieldRequired := range schemaFieldsRequired {
		index = nil
		for i, fieldRequired := range fieldsRequired {
			if schemaFieldRequired == fieldRequired {
				index = &i
				break
			}
		}
		if index == nil {
			unique = append(unique, schemaFieldsRequired[j])
		}
	}
	return unique
}
