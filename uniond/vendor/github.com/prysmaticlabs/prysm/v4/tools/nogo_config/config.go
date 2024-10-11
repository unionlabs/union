package main

// These Config types are copied from bazelbuild/rules_go.
// License: Apache 2.0
// https://github.com/bazelbuild/rules_go/blob/c90a11ad8dc5f3f9d633f0556b22c90af1b01116/go/tools/builders/generate_nogo_main.go#L193
type Configs map[string]Config

type Config struct {
	Description   string
	OnlyFiles     map[string]string `json:"only_files"`
	ExcludeFiles  map[string]string `json:"exclude_files"`
	AnalyzerFlags map[string]string `json:"analyzer_flags"`
}
