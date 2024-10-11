// metricsdiff is a tool for generating a diff between two different files containing
// prometheus metrics. metricsdiff outputs which metrics have been added, removed,
// or have different sets of labels between the two files.
package main

import (
	"flag"
	"fmt"
	"io"
	"log"
	"os"
	"path/filepath"
	"sort"
	"strings"

	dto "github.com/prometheus/client_model/go"
	"github.com/prometheus/common/expfmt"
)

func init() {
	flag.Usage = func() {
		fmt.Fprintf(os.Stderr, `Usage: %[1]s <path1> <path2>

Generate the diff between the two files of Prometheus metrics.
The input should have the format output by a Prometheus HTTP endpoint.
The tool indicates which metrics have been added, removed, or use different
label sets from path1 to path2.

`, filepath.Base(os.Args[0]))
		flag.PrintDefaults()
	}
}

// Diff contains the set of metrics that were modified between two files
// containing prometheus metrics output.
type Diff struct {
	Adds    []string
	Removes []string

	Changes []LabelDiff
}

// LabelDiff describes the label changes between two versions of the same metric.
type LabelDiff struct {
	Metric  string
	Adds    []string
	Removes []string
}

type parsedMetric struct {
	name   string
	labels []string
}

type metricsList []parsedMetric

func main() {
	flag.Parse()
	if flag.NArg() != 2 {
		log.Fatalf("Usage is '%s <path1> <path2>', got %d arguments",
			filepath.Base(os.Args[0]), flag.NArg())
	}
	fa, err := os.Open(flag.Arg(0))
	if err != nil {
		log.Fatalf("Open: %v", err)
	}
	fb, err := os.Open(flag.Arg(1))
	if err != nil {
		log.Fatalf("Open: %v", err)
	}
	md, err := DiffFromReaders(fa, fb)
	if err != nil {
		log.Fatalf("Generating diff: %v", err)
	}
	fa.Close()
	fb.Close()
	fmt.Print(md)
}

// DiffFromReaders parses the metrics present in the readers a and b and
// determines which metrics were added and removed in b.
func DiffFromReaders(a, b io.Reader) (Diff, error) {
	var parser expfmt.TextParser
	amf, err := parser.TextToMetricFamilies(a)
	if err != nil {
		return Diff{}, err
	}
	bmf, err := parser.TextToMetricFamilies(b)
	if err != nil {
		return Diff{}, err
	}

	md := Diff{}
	aList := toList(amf)
	bList := toList(bmf)

	i, j := 0, 0
	for i < len(aList) || j < len(bList) {
		for j < len(bList) && (i >= len(aList) || bList[j].name < aList[i].name) {
			md.Adds = append(md.Adds, bList[j].name)
			j++
		}
		for i < len(aList) && j < len(bList) && aList[i].name == bList[j].name {
			adds, removes := listDiff(aList[i].labels, bList[j].labels)
			if len(adds) > 0 || len(removes) > 0 {
				md.Changes = append(md.Changes, LabelDiff{
					Metric:  aList[i].name,
					Adds:    adds,
					Removes: removes,
				})
			}
			i++
			j++
		}
		for i < len(aList) && (j >= len(bList) || aList[i].name < bList[j].name) {
			md.Removes = append(md.Removes, aList[i].name)
			i++
		}
	}
	return md, nil
}

func toList(l map[string]*dto.MetricFamily) metricsList {
	r := make([]parsedMetric, len(l))
	var idx int
	for name, family := range l {
		r[idx] = parsedMetric{
			name:   name,
			labels: labelsToStringList(family.GetMetric()[0].GetLabel()),
		}
		idx++
	}
	sort.Sort(metricsList(r))
	return r
}

func labelsToStringList(ls []*dto.LabelPair) []string {
	r := make([]string, len(ls))
	for i, l := range ls {
		r[i] = l.GetName()
	}
	return sort.StringSlice(r)
}

func listDiff(a, b []string) ([]string, []string) {
	adds, removes := []string{}, []string{}
	i, j := 0, 0
	for i < len(a) || j < len(b) {
		for j < len(b) && (i >= len(a) || b[j] < a[i]) {
			adds = append(adds, b[j])
			j++
		}
		for i < len(a) && j < len(b) && a[i] == b[j] {
			i++
			j++
		}
		for i < len(a) && (j >= len(b) || a[i] < b[j]) {
			removes = append(removes, a[i])
			i++
		}
	}
	return adds, removes
}

func (m metricsList) Len() int           { return len(m) }
func (m metricsList) Less(i, j int) bool { return m[i].name < m[j].name }
func (m metricsList) Swap(i, j int)      { m[i], m[j] = m[j], m[i] }

func (m Diff) String() string {
	var s strings.Builder
	if len(m.Adds) > 0 || len(m.Removes) > 0 {
		fmt.Fprintln(&s, "Metric changes:")
	}
	if len(m.Adds) > 0 {
		for _, add := range m.Adds {
			fmt.Fprintf(&s, "+++ %s\n", add)
		}
	}
	if len(m.Removes) > 0 {
		for _, rem := range m.Removes {
			fmt.Fprintf(&s, "--- %s\n", rem)
		}
	}
	if len(m.Changes) > 0 {
		fmt.Fprintln(&s, "Label changes:")
		for _, ld := range m.Changes {
			fmt.Fprintf(&s, "Metric: %s\n", ld.Metric)
			for _, add := range ld.Adds {
				fmt.Fprintf(&s, "+++ %s\n", add)
			}
			for _, rem := range ld.Removes {
				fmt.Fprintf(&s, "--- %s\n", rem)
			}
		}
	}
	return s.String()
}
