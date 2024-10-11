package main_test

import (
	"bytes"
	"fmt"
	"go/parser"
	"go/token"
	"io"
	"os"
	"path"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/require"

	metricsgen "github.com/cometbft/cometbft/scripts/metricsgen" //nolint:revive // this only works with metricsgen in front
)

const testDataDir = "./testdata"

func TestSimpleTemplate(t *testing.T) {
	m := metricsgen.ParsedMetricField{
		TypeName:    "Histogram",
		FieldName:   "MyMetric",
		MetricName:  "request_count",
		Description: "how many requests were made since the start of the process",
		Labels:      "first, second, third",
	}
	td := metricsgen.TemplateData{
		Package:       "mypack",
		ParsedMetrics: []metricsgen.ParsedMetricField{m},
	}
	b := bytes.NewBuffer([]byte{})
	err := metricsgen.GenerateMetricsFile(b, td)
	if err != nil {
		t.Fatalf("unable to parse template %v", err)
	}
}

// TestFromData tests that the metricsgen tool can parse a directory of metrics and generate a file.
func TestFromData(t *testing.T) {
	infos, err := os.ReadDir(testDataDir)
	if err != nil {
		t.Fatalf("unable to open file %v", err)
	}
	for _, dir := range infos {
		t.Run(dir.Name(), func(t *testing.T) {
			if !dir.IsDir() {
				t.Fatalf("expected file %s to be directory", dir.Name())
			}
			dirName := path.Join(testDataDir, dir.Name())
			pt, err := metricsgen.ParseMetricsDir(dirName, "Metrics")
			if err != nil {
				t.Fatalf("unable to parse from dir %q: %v", dir, err)
			}
			outFile := path.Join(dirName, "out.go")
			if err != nil {
				t.Fatalf("unable to open file %s: %v", outFile, err)
			}
			of, err := os.Create(outFile)
			if err != nil {
				t.Fatalf("unable to open file %s: %v", outFile, err)
			}
			defer os.Remove(outFile)
			if err := metricsgen.GenerateMetricsFile(of, pt); err != nil {
				t.Fatalf("unable to generate metrics file %s: %v", outFile, err)
			}
			if _, err := parser.ParseFile(token.NewFileSet(), outFile, nil, parser.AllErrors); err != nil {
				t.Fatalf("unable to parse generated file %s: %v", outFile, err)
			}
			bNew, err := os.ReadFile(outFile)
			if err != nil {
				t.Fatalf("unable to read generated file %s: %v", outFile, err)
			}
			goldenFile := path.Join(dirName, "metrics.gen.go")
			bOld, err := os.ReadFile(goldenFile)
			if err != nil {
				t.Fatalf("unable to read file %s: %v", goldenFile, err)
			}
			if !bytes.Equal(bNew, bOld) {
				t.Fatalf("newly generated code in file %s does not match golden file %s\n"+
					"if the output of the metricsgen tool is expected to change run the following make target: \n"+
					"\tmake metrics", outFile, goldenFile)
			}
		})
	}
}

func TestParseMetricsStruct(t *testing.T) {
	const pkgName = "mypkg"
	metricsTests := []struct {
		name          string
		shouldError   bool
		metricsStruct string
		expected      metricsgen.TemplateData
	}{
		{
			name: "basic",
			metricsStruct: `type Metrics struct {
				myGauge metrics.Gauge
			}`,
			expected: metricsgen.TemplateData{
				Package: pkgName,
				ParsedMetrics: []metricsgen.ParsedMetricField{
					{
						TypeName:   "Gauge",
						FieldName:  "myGauge",
						MetricName: "my_gauge",
					},
				},
			},
		},
		{
			name: "histogram",
			metricsStruct: "type Metrics struct {\n" +
				"myHistogram metrics.Histogram `metrics_buckettype:\"exp\" metrics_bucketsizes:\"1, 100, .8\"`\n" +
				"}",
			expected: metricsgen.TemplateData{
				Package: pkgName,
				ParsedMetrics: []metricsgen.ParsedMetricField{
					{
						TypeName:   "Histogram",
						FieldName:  "myHistogram",
						MetricName: "my_histogram",

						HistogramOptions: metricsgen.HistogramOpts{
							BucketType:  "stdprometheus.ExponentialBuckets",
							BucketSizes: "1, 100, .8",
						},
					},
				},
			},
		},
		{
			name: "labeled name",
			metricsStruct: "type Metrics struct {\n" +
				"myCounter metrics.Counter `metrics_name:\"new_name\"`\n" +
				"}",
			expected: metricsgen.TemplateData{
				Package: pkgName,
				ParsedMetrics: []metricsgen.ParsedMetricField{
					{
						TypeName:   "Counter",
						FieldName:  "myCounter",
						MetricName: "new_name",
					},
				},
			},
		},
		{
			name: "metric labels",
			metricsStruct: "type Metrics struct {\n" +
				"myCounter metrics.Counter `metrics_labels:\"label1,label2\"`\n" +
				"}",
			expected: metricsgen.TemplateData{
				Package: pkgName,
				ParsedMetrics: []metricsgen.ParsedMetricField{
					{
						TypeName:   "Counter",
						FieldName:  "myCounter",
						MetricName: "my_counter",
						Labels:     "\"label1\",\"label2\"",
					},
				},
			},
		},
		{
			name: "ignore non-metric field",
			metricsStruct: `type Metrics struct {
				myCounter metrics.Counter
				nonMetric string
				}`,
			expected: metricsgen.TemplateData{
				Package: pkgName,
				ParsedMetrics: []metricsgen.ParsedMetricField{
					{
						TypeName:   "Counter",
						FieldName:  "myCounter",
						MetricName: "my_counter",
					},
				},
			},
		},
		{
			name: "parse description from comments",
			metricsStruct: `type Metrics struct {
				// myCounter is a counter.
				// It does count.
				myCounter metrics.Counter
				nonMetric string
				}`,
			expected: metricsgen.TemplateData{
				Package: pkgName,
				ParsedMetrics: []metricsgen.ParsedMetricField{
					{
						Description: "myCounter is a counter. It does count.",
						TypeName:    "Counter",
						FieldName:   "myCounter",
						MetricName:  "my_counter",
					},
				},
			},
		},
		{
			name: "parse short description from comments",
			metricsStruct: `type Metrics struct {
				// myCounter is a counter.
				//
				// myCounter needs a super long description,
				// we don't want it on the description.
				// metrics:It does count.
				myCounter metrics.Counter
				nonMetric string
				}`,
			expected: metricsgen.TemplateData{
				Package: pkgName,
				ParsedMetrics: []metricsgen.ParsedMetricField{
					{
						Description: "It does count.",
						TypeName:    "Counter",
						FieldName:   "myCounter",
						MetricName:  "my_counter",
					},
				},
			},
		},
	}
	for _, testCase := range metricsTests {
		t.Run(testCase.name, func(t *testing.T) {
			dir, err := os.MkdirTemp(os.TempDir(), "metricsdir")
			if err != nil {
				t.Fatalf("unable to create directory: %v", err)
			}
			defer os.Remove(dir)
			f, err := os.Create(filepath.Join(dir, "metrics.go"))
			if err != nil {
				t.Fatalf("unable to open file: %v", err)
			}
			pkgLine := fmt.Sprintf("package %s\n", pkgName)
			importClause := `
			import(
				"github.com/go-kit/kit/metrics"
			)
			`

			_, err = io.WriteString(f, pkgLine)
			require.NoError(t, err)
			_, err = io.WriteString(f, importClause)
			require.NoError(t, err)
			_, err = io.WriteString(f, testCase.metricsStruct)
			require.NoError(t, err)

			td, err := metricsgen.ParseMetricsDir(dir, "Metrics")
			if testCase.shouldError {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
				require.Equal(t, testCase.expected, td)
			}
		})
	}
}

func TestParseAliasedMetric(t *testing.T) {
	aliasedData := `
			package mypkg

			import(
				mymetrics "github.com/go-kit/kit/metrics"
			)
			type Metrics struct {
				m mymetrics.Gauge
			}
			`
	dir, err := os.MkdirTemp(os.TempDir(), "metricsdir")
	if err != nil {
		t.Fatalf("unable to create directory: %v", err)
	}
	defer os.Remove(dir)
	f, err := os.Create(filepath.Join(dir, "metrics.go"))
	if err != nil {
		t.Fatalf("unable to open file: %v", err)
	}
	_, err = io.WriteString(f, aliasedData)
	if err != nil {
		t.Fatalf("unable to write to file: %v", err)
	}
	td, err := metricsgen.ParseMetricsDir(dir, "Metrics")
	require.NoError(t, err)

	expected := metricsgen.TemplateData{
		Package: "mypkg",
		ParsedMetrics: []metricsgen.ParsedMetricField{
			{
				TypeName:   "Gauge",
				FieldName:  "m",
				MetricName: "m",
			},
		},
	}
	require.Equal(t, expected, td)
}
