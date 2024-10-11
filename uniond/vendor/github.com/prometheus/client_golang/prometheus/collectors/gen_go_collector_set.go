// Copyright 2021 The Prometheus Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//go:build ignore
// +build ignore

package main

import (
	"bytes"
	"fmt"
	"go/format"
	"log"
	"os"
	"regexp"
	"runtime"
	"runtime/metrics"
	"sort"
	"strings"
	"text/template"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/internal"

	version "github.com/hashicorp/go-version"
)

type metricGroup struct {
	Name    string
	Regex   *regexp.Regexp
	Metrics []string
}

var metricGroups = []metricGroup{
	{"withAllMetrics", nil, nil},
	{"withGCMetrics", regexp.MustCompile("^go_gc_.*"), nil},
	{"withMemoryMetrics", regexp.MustCompile("^go_memory_classes_.*"), nil},
	{"withSchedulerMetrics", regexp.MustCompile("^go_sched_.*"), nil},
	{"withDebugMetrics", regexp.MustCompile("^go_godebug_non_default_behavior_.*"), nil},
}

func main() {
	var givenVersion string
	toolVersion := runtime.Version()
	if len(os.Args) != 2 {
		log.Printf("requires Go version (e.g. go1.17) as an argument. Since it is not specified, assuming %s.", toolVersion)
		givenVersion = toolVersion
	} else {
		givenVersion = os.Args[1]
	}
	log.Printf("given version for Go: %s", givenVersion)
	log.Printf("tool version for Go: %s", toolVersion)

	tv, err := version.NewVersion(strings.TrimPrefix(givenVersion, "go"))
	if err != nil {
		log.Fatal(err)
	}

	toolVersion = strings.Split(strings.TrimPrefix(toolVersion, "go"), " ")[0]
	gv, err := version.NewVersion(toolVersion)
	if err != nil {
		log.Fatal(err)
	}
	if !gv.Equal(tv) {
		log.Fatalf("using Go version %q but expected Go version %q", tv, gv)
	}

	v := goVersion(gv.Segments()[1])
	log.Printf("generating metrics for Go version %q", v)

	descriptions := computeMetricsList()
	groupedMetrics := groupMetrics(descriptions)

	// Generate code.
	var buf bytes.Buffer
	err = testFile.Execute(&buf, struct {
		GoVersion goVersion
		Groups    []metricGroup
	}{
		GoVersion: v,
		Groups:    groupedMetrics,
	})
	if err != nil {
		log.Fatalf("executing template: %v", err)
	}

	// Format it.
	result, err := format.Source(buf.Bytes())
	if err != nil {
		log.Fatalf("formatting code: %v", err)
	}

	// Write it to a file.
	fname := fmt.Sprintf("go_collector_%s_test.go", v.Abbr())
	if err := os.WriteFile(fname, result, 0o644); err != nil {
		log.Fatalf("writing file: %v", err)
	}
}

func computeMetricsList() []string {
	var metricsList []string
	for _, d := range metrics.All() {
		if trans := rm2prom(d); trans != "" {
			metricsList = append(metricsList, trans)
		}
	}
	return metricsList
}

func rm2prom(d metrics.Description) string {
	ns, ss, n, ok := internal.RuntimeMetricsToProm(&d)
	if !ok {
		return ""
	}
	return prometheus.BuildFQName(ns, ss, n)
}

func groupMetrics(metricsList []string) []metricGroup {
	var groupedMetrics []metricGroup
	for _, group := range metricGroups {
		matchedMetrics := make([]string, 0)
		for _, metric := range metricsList {
			if group.Regex == nil || group.Regex.MatchString(metric) {
				matchedMetrics = append(matchedMetrics, metric)
			}
		}

		sort.Strings(matchedMetrics)
		groupedMetrics = append(groupedMetrics, metricGroup{
			Name:    group.Name,
			Regex:   group.Regex,
			Metrics: matchedMetrics,
		})
	}
	return groupedMetrics
}

type goVersion int

func (g goVersion) String() string {
	return fmt.Sprintf("go1.%d", g)
}

func (g goVersion) Abbr() string {
	return fmt.Sprintf("go1%d", g)
}

var testFile = template.Must(template.New("testFile").Funcs(map[string]interface{}{
	"nextVersion": func(version goVersion) string {
		return (version + goVersion(1)).String()
	},
}).Parse(`// Copyright 2022 The Prometheus Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//go:build {{.GoVersion}} && !{{nextVersion .GoVersion}}
// +build {{.GoVersion}},!{{nextVersion .GoVersion}}

package collectors

{{- range .Groups }}
func {{ .Name }}() []string {
	return withBaseMetrics([]string{
		{{- range $metric := .Metrics }}
			{{ $metric | printf "%q" }},
		{{- end }}
	})
}
{{ end }}
`))
