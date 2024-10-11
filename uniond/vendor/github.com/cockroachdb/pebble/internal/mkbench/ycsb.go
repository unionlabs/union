// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package main

import (
	"bufio"
	"bytes"
	"compress/bzip2"
	"compress/gzip"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"math"
	"os"
	"sort"
	"strings"

	"github.com/cockroachdb/errors/oserror"
	"github.com/spf13/cobra"
)

const (
	defaultDir        = "data"
	defaultCookedFile = "data.js"
)

func getYCSBCommand() *cobra.Command {
	c := &cobra.Command{
		Use:   "ycsb",
		Short: "parse YCSB benchmark data",
		RunE: func(cmd *cobra.Command, args []string) error {
			dataDir, err := cmd.Flags().GetString("dir")
			if err != nil {
				return err
			}

			inFile, err := cmd.Flags().GetString("in")
			if err != nil {
				return err
			}

			outFile, err := cmd.Flags().GetString("out")
			if err != nil {
				return err
			}

			parseYCSB(dataDir, inFile, outFile)
			return nil
		},
	}

	c.Flags().String("dir", defaultDir, "path to data directory")
	c.Flags().String("in", defaultCookedFile, "path to (possibly non-empty) input cooked data file")
	c.Flags().String("out", defaultCookedFile, "path to output data file")
	c.SilenceUsage = true

	return c
}

type ycsbRun struct {
	opsSec     float64
	readBytes  int64
	writeBytes int64
	readAmp    float64
	writeAmp   float64
}

func (r ycsbRun) formatCSV() string {
	return fmt.Sprintf("%.1f,%d,%d,%.1f,%.1f",
		r.opsSec, r.readBytes, r.writeBytes, r.readAmp, r.writeAmp)
}

type ycsbWorkload struct {
	days map[string][]ycsbRun // data -> runs
}

type ycsbLoader struct {
	cookedDays map[string]bool          // set of already cooked days
	data       map[string]*ycsbWorkload // workload name -> workload data
}

func newYCSBLoader() *ycsbLoader {
	return &ycsbLoader{
		cookedDays: make(map[string]bool),
		data:       make(map[string]*ycsbWorkload),
	}
}

func (l *ycsbLoader) addRun(name, day string, r ycsbRun) {
	w := l.data[name]
	if w == nil {
		w = &ycsbWorkload{days: make(map[string][]ycsbRun)}
		l.data[name] = w
	}
	w.days[day] = append(w.days[day], r)
}

func (l *ycsbLoader) loadCooked(path string) {
	data, err := os.ReadFile(path)
	if oserror.IsNotExist(err) {
		return
	}
	if err != nil {
		log.Fatal(err)
	}

	data = bytes.TrimSpace(data)

	prefix := []byte("data = ")
	if !bytes.HasPrefix(data, prefix) {
		log.Fatalf("missing '%s' prefix", prefix)
	}
	data = bytes.TrimPrefix(data, prefix)

	suffix := []byte(";")
	if !bytes.HasSuffix(data, suffix) {
		log.Fatalf("missing '%s' suffix", suffix)
	}
	data = bytes.TrimSuffix(data, suffix)

	m := make(map[string]string)
	if err := json.Unmarshal(data, &m); err != nil {
		log.Fatal(err)
	}

	for name, data := range m {
		s := bufio.NewScanner(strings.NewReader(data))
		for s.Scan() {
			line := s.Text()
			line = strings.Replace(line, ",", " ", -1)

			var r ycsbRun
			var day string
			n, err := fmt.Sscanf(line, "%s %f %d %d %f %f",
				&day, &r.opsSec, &r.readBytes, &r.writeBytes, &r.readAmp, &r.writeAmp)
			if err != nil || n != 6 {
				log.Fatalf("%s: %+v", line, err)
			}
			l.cookedDays[day] = true
			l.addRun(name, day, r)
		}
	}
}

func (l *ycsbLoader) loadRaw(dir string) {
	walkFn := func(path, pathRel string, info os.FileInfo) error {
		// The directory structure is of the form:
		//   $date/pebble/ycsb/$name/$run/$file
		parts := strings.Split(pathRel, string(os.PathSeparator))
		if len(parts) < 6 {
			return nil // stumble forward on invalid paths
		}

		// We're only interested in YCSB benchmark data.
		if parts[2] != "ycsb" {
			return nil
		}

		day := parts[0]
		if l.cookedDays[day] {
			return nil
		}

		f, err := os.Open(path)
		if err != nil {
			fmt.Fprintf(os.Stderr, "%+v\n", err)
			return nil // stumble forward on error
		}
		defer f.Close()

		r := io.Reader(f)
		if strings.HasSuffix(path, ".bz2") {
			r = bzip2.NewReader(f)
		} else if strings.HasSuffix(path, ".gz") {
			var err error
			r, err = gzip.NewReader(f)
			if err != nil {
				fmt.Fprintf(os.Stderr, "%+v\n", err)
				return nil // stumble forward on error
			}
		}

		s := bufio.NewScanner(r)
		for s.Scan() {
			line := s.Text()
			if !strings.HasPrefix(line, "Benchmark") {
				continue
			}

			var r ycsbRun
			var name string
			var ops int64
			n, err := fmt.Sscanf(line,
				"Benchmark%s %d %f ops/sec %d read %d write %f r-amp %f w-amp",
				&name, &ops, &r.opsSec, &r.readBytes, &r.writeBytes, &r.readAmp, &r.writeAmp)
			if err != nil || n != 7 {
				fmt.Fprintf(os.Stderr, "%s: %v\n", s.Text(), err)
				// Stumble forward on error.
				continue
			}

			fmt.Fprintf(os.Stderr, "%s: adding %s\n", day, name)
			l.addRun(name, day, r)
		}
		return nil
	}

	_ = walkDir(dir, walkFn)
}

func (l *ycsbLoader) cook(path string) {
	m := make(map[string]string)
	for name, workload := range l.data {
		m[name] = l.cookWorkload(workload)
	}

	out := []byte("data = ")
	out = append(out, prettyJSON(m)...)
	out = append(out, []byte(";\n")...)
	if err := os.WriteFile(path, out, 0644); err != nil {
		log.Fatal(err)
	}
}

func (l *ycsbLoader) cookWorkload(w *ycsbWorkload) string {
	days := make([]string, 0, len(w.days))
	for day := range w.days {
		days = append(days, day)
	}
	sort.Strings(days)

	var buf bytes.Buffer
	for _, day := range days {
		fmt.Fprintf(&buf, "%s,%s\n", day, l.cookDay(w.days[day]))
	}
	return buf.String()
}

func (l *ycsbLoader) cookDay(runs []ycsbRun) string {
	if len(runs) == 1 {
		return runs[0].formatCSV()
	}

	// The benchmarks show significant run-to-run variance due to
	// instance-to-instance performance variability on AWS. We attempt to smooth
	// out this variance by excluding outliers: any run that is more than one
	// stddev from the average, and then taking the average of the remaining
	// runs. Note that the runs on a given day are all from the same SHA, so this
	// smoothing will not affect exceptional day-to-day performance changes.

	var sum float64
	for i := range runs {
		sum += runs[i].opsSec
	}
	mean := sum / float64(len(runs))

	var sum2 float64
	for i := range runs {
		v := runs[i].opsSec - mean
		sum2 += v * v
	}

	stddev := math.Sqrt(sum2 / float64(len(runs)))
	lo := mean - stddev
	hi := mean + stddev

	var avg ycsbRun
	var count int
	for i := range runs {
		r := &runs[i]
		if r.opsSec < lo || r.opsSec > hi {
			continue
		}
		count++
		avg.opsSec += r.opsSec
		avg.readBytes += r.readBytes
		avg.writeBytes += r.writeBytes
		avg.readAmp += r.readAmp
		avg.writeAmp += r.writeAmp
	}

	avg.opsSec /= float64(count)
	avg.readBytes /= int64(count)
	avg.writeBytes /= int64(count)
	avg.readAmp /= float64(count)
	avg.writeAmp /= float64(count)
	return avg.formatCSV()
}

// parseYCSB coalesces YCSB benchmark data.
func parseYCSB(dataDir, inFile, outFile string) {
	log.SetFlags(log.Lshortfile)

	l := newYCSBLoader()
	l.loadCooked(inFile)
	l.loadRaw(dataDir)
	l.cook(outFile)
}
