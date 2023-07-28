// Package profile provides a simple way to generate pprof compatible gnark circuit profile.
//
// Since the gnark frontend compiler is not thread safe and operates in a single go-routine,
// this package is also NOT thread safe and is meant to be called in the same go-routine.
package profile

import (
	"bytes"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"sync"
	"sync/atomic"

	"github.com/consensys/gnark/logger"
	"github.com/consensys/gnark/profile/internal/report"
	"github.com/google/pprof/profile"
)

var (
	sessions       []*Profile // active sessions
	activeSessions uint32
)

// Profile represents an active constraint system profiling session.
type Profile struct {
	// defaults to ./gnark.pprof
	// if blank, profiile is not written to disk
	filePath string

	// actual pprof profile struct
	// details on pprof format: https://github.com/google/pprof/blob/main/proto/README.md
	pprof profile.Profile

	functions map[string]*profile.Function
	locations map[uint64]*profile.Location

	onceSetName sync.Once

	chDone chan struct{}
}

// Option defines configuration Options for Profile.
type Option func(*Profile)

// WithPath controls the profile destination file. If blank, profile is not written.
//
// Defaults to ./gnark.pprof.
func WithPath(path string) Option {
	return func(p *Profile) {
		p.filePath = path
	}
}

// WithNoOutput indicates that the profile is not going to be written to disk.
//
// This is equivalent to WithPath("")
func WithNoOutput() Option {
	return func(p *Profile) {
		p.filePath = ""
	}
}

// Start creates a new active profiling session. When Stop() is called, this session is removed from
// active profiling sessions and may be serialized to disk as a pprof compatible file (see ProfilePath option).
//
// All calls to profile.Start() and Stop() are meant to be executed in the same go routine (frontend.Compile).
//
// It is allowed to create multiple overlapping profiling sessions in one circuit.
func Start(options ...Option) *Profile {

	// start the worker first time a profiling session starts.
	onceInit.Do(func() {
		go worker()
	})

	p := Profile{
		functions: make(map[string]*profile.Function),
		locations: make(map[uint64]*profile.Location),
		filePath:  filepath.Join(".", "gnark.pprof"),
		chDone:    make(chan struct{}),
	}
	p.pprof.SampleType = []*profile.ValueType{{
		Type: "constraints",
		Unit: "count",
	}}

	for _, option := range options {
		option(&p)
	}

	log := logger.Logger()
	if p.filePath == "" {
		log.Warn().Msg("gnark profiling enabled [not writing to disk]")
	} else {
		log.Info().Str("path", p.filePath).Msg("gnark profiling enabled")
	}

	// add the session to active sessions
	chCommands <- command{p: &p}
	atomic.AddUint32(&activeSessions, 1)

	return &p
}

// Stop removes the profile from active session and may write the pprof file to disk. See ProfilePath option.
func (p *Profile) Stop() {
	log := logger.Logger()

	if p.chDone == nil {
		log.Fatal().Msg("gnark profile stopped multiple times")
	}

	// ask worker routine to remove ourselves from the active sessions
	chCommands <- command{p: p, remove: true}

	// wait for worker routine to remove us.
	<-p.chDone
	p.chDone = nil

	// if filePath is set, serialize profile to disk in pprof format
	if p.filePath != "" {
		f, err := os.Create(p.filePath)
		if err != nil {
			log.Fatal().Err(err).Msg("could not create gnark profile")
		}
		if err := p.pprof.Write(f); err != nil {
			log.Error().Err(err).Msg("writing profile")
		}
		f.Close()
		log.Info().Str("path", p.filePath).Msg("gnark profiling disabled")
	} else {
		log.Warn().Msg("gnark profiling disabled [not writing to disk]")
	}

}

// NbConstraints return number of collected samples (constraints) by the profile session
func (p *Profile) NbConstraints() int {
	return len(p.pprof.Sample)
}

// Top return a similar output than pprof top command
func (p *Profile) Top() string {
	r := report.NewDefault(&p.pprof, report.Options{
		OutputFormat:  report.Tree,
		CompactLabels: true,
		NodeFraction:  0.005,
		EdgeFraction:  0.001,
		SampleValue:   func(v []int64) int64 { return v[0] },
		SampleUnit:    "count",
	})
	var buf bytes.Buffer
	report.Generate(&buf, r)
	return buf.String()
}

// RecordConstraint add a sample (with count == 1) to all the active profiling sessions.
func RecordConstraint() {
	if n := atomic.LoadUint32(&activeSessions); n == 0 {
		return // do nothing, no active session.
	}

	// collect the stack and send it async to the worker
	pc := make([]uintptr, 20)
	n := runtime.Callers(3, pc)
	if n == 0 {
		return
	}
	pc = pc[:n]
	chCommands <- command{pc: pc}
}

func (p *Profile) getLocation(frame *runtime.Frame) *profile.Location {
	l, ok := p.locations[uint64(frame.PC)]
	if !ok {
		// first let's see if we have the function.
		f, ok := p.functions[frame.File+frame.Function]
		if !ok {
			fe := strings.Split(frame.Function, "/")
			fName := fe[len(fe)-1]
			f = &profile.Function{
				ID:         uint64(len(p.functions) + 1),
				Name:       fName,
				SystemName: frame.Function,
				Filename:   frame.File,
			}

			p.functions[frame.File+frame.Function] = f
			p.pprof.Function = append(p.pprof.Function, f)
		}

		l = &profile.Location{
			ID:   uint64(len(p.locations) + 1),
			Line: []profile.Line{{Function: f, Line: int64(frame.Line)}},
		}
		p.locations[uint64(frame.PC)] = l
		p.pprof.Location = append(p.pprof.Location, l)
	}

	return l
}
