package depinject

import (
	"fmt"
	"os"
	"path/filepath"
	"reflect"

	"cosmossdk.io/depinject/internal/graphviz"
)

// DebugOption is a functional option for running a container that controls
// debug logging and visualization output.
type DebugOption interface {
	applyConfig(*debugConfig) error
}

// StdoutLogger is a debug option which routes logging output to stdout.
func StdoutLogger() DebugOption {
	return Logger(func(s string) {
		_, _ = fmt.Fprintln(os.Stdout, s)
	})
}

// StderrLogger is a debug option which routes logging output to stderr.
func StderrLogger() DebugOption {
	return Logger(func(s string) {
		_, _ = fmt.Fprintln(os.Stderr, s)
	})
}

// FileLogger is a debug option which routes logging output to a file.
func FileLogger(filename string) DebugOption {
	var f *os.File
	return Logger(func(s string) {
		var err error
		if f == nil {
			f, err = os.Create(filename)
			if err != nil {
				panic(err)
			}
		}

		_, err = f.Write([]byte(s))
		if err != nil {
			panic(err)
		}

		_, err = f.Write([]byte("\n"))
		if err != nil {
			panic(err)
		}
	})
}

// Visualizer creates an option which provides a visualizer function which
// will receive a rendering of the container in the Graphiz DOT format
// whenever the container finishes building or fails due to an error. The
// graph is color-coded to aid debugging with black representing success,
// red representing an error, and gray representing unused types or functions.
// Graph rendering should be deterministic for a given version of the container
// module and container options so that graphs can be used in tests.
func Visualizer(visualizer func(dotGraph string)) DebugOption {
	return debugOption(func(c *debugConfig) error {
		c.addFuncVisualizer(visualizer)
		return nil
	})
}

// LogVisualizer is a debug option which dumps a graphviz DOT rendering of
// the container to the log.
func LogVisualizer() DebugOption {
	return debugOption(func(c *debugConfig) error {
		c.enableLogVisualizer()
		return nil
	})
}

// FileVisualizer is a debug option which dumps a graphviz DOT rendering of
// the container to the specified file.
func FileVisualizer(filename string) DebugOption {
	return debugOption(func(c *debugConfig) error {
		c.addFileVisualizer(filename)
		return nil
	})
}

// Logger creates an option which provides a logger function which will
// receive all log messages from the container.
func Logger(logger func(string)) DebugOption {
	return debugOption(func(c *debugConfig) error {
		logger("Initializing logger")
		c.loggers = append(c.loggers, logger)

		// send conditional log messages batched for onError/onSuccess cases
		if c.logBuf != nil {
			for _, s := range *c.logBuf {
				logger(s)
			}
		}

		return nil
	})
}

const (
	debugContainerDot = "debug_container.dot"
	debugContainerLog = "debug_container.log"
)

// Debug is a default debug option which sends log output to stderr, dumps
// the container in the graphviz DOT and SVG formats to debug_container.dot
// and debug_container.svg respectively.
func Debug() DebugOption {
	return DebugOptions(
		FileLogger(debugContainerLog),
		FileVisualizer(debugContainerDot),
	)
}

func (c *debugConfig) initLogBuf() {
	if c.logBuf == nil {
		c.logBuf = &[]string{}
		c.loggers = append(c.loggers, func(s string) {
			*c.logBuf = append(*c.logBuf, s)
		})
	}
}

// OnError is a debug option that allows setting debug options that are
// conditional on an error happening. Any loggers added error will
// receive the full dump of logs since the start of container processing.
func OnError(option DebugOption) DebugOption {
	return debugOption(func(config *debugConfig) error {
		config.initLogBuf()
		config.onError = option
		return nil
	})
}

// OnSuccess is a debug option that allows setting debug options that are
// conditional on successful container resolution. Any loggers added on success
// will receive the full dump of logs since the start of container processing.
func OnSuccess(option DebugOption) DebugOption {
	return debugOption(func(config *debugConfig) error {
		config.initLogBuf()
		config.onSuccess = option
		return nil
	})
}

// DebugCleanup specifies a clean-up function to be called at the end of
// processing to clean up any resources that may be used during debugging.
func DebugCleanup(cleanup func()) DebugOption {
	return debugOption(func(config *debugConfig) error {
		config.cleanup = append(config.cleanup, cleanup)
		return nil
	})
}

// AutoDebug does the same thing as Debug when there is an error and deletes
// the debug_container.dot if it exists when there is no error. This is the
// default debug mode of Run.
func AutoDebug() DebugOption {
	return DebugOptions(
		OnError(Debug()),
		OnSuccess(DebugCleanup(func() {
			deleteIfExists(debugContainerDot)
			deleteIfExists(debugContainerLog)
		})),
	)
}

func deleteIfExists(filename string) {
	if _, err := os.Stat(filename); err == nil {
		_ = os.Remove(filename)
	}
}

// DebugOptions creates a debug option which bundles together other debug options.
func DebugOptions(options ...DebugOption) DebugOption {
	return debugOption(func(c *debugConfig) error {
		for _, opt := range options {
			if err := opt.applyConfig(c); err != nil {
				return err
			}
		}
		return nil
	})
}

type debugConfig struct {
	// logging
	loggers   []func(string)
	indentStr string
	logBuf    *[]string // a log buffer for onError/onSuccess processing

	// graphing
	graph         *graphviz.Graph
	visualizers   []func(string)
	logVisualizer bool

	// extra processing
	onError   DebugOption
	onSuccess DebugOption
	cleanup   []func()
}

type debugOption func(*debugConfig) error

func (c debugOption) applyConfig(ctr *debugConfig) error {
	return c(ctr)
}

var _ DebugOption = (*debugOption)(nil)

func newDebugConfig() (*debugConfig, error) {
	return &debugConfig{
		graph: graphviz.NewGraph(),
	}, nil
}

func (c *debugConfig) indentLogger() {
	c.indentStr += " "
}

func (c *debugConfig) dedentLogger() {
	if len(c.indentStr) > 0 {
		c.indentStr = c.indentStr[1:]
	}
}

func (c debugConfig) logf(format string, args ...interface{}) {
	s := fmt.Sprintf(c.indentStr+format, args...)
	for _, logger := range c.loggers {
		logger(s)
	}
}

func (c *debugConfig) generateGraph() {
	dotStr := c.graph.String()
	if c.logVisualizer {
		c.logf("DOT Graph: %s", dotStr)
	}

	for _, v := range c.visualizers {
		v(dotStr)
	}
}

func (c *debugConfig) addFuncVisualizer(f func(string)) {
	c.visualizers = append(c.visualizers, func(dot string) {
		f(dot)
	})
}

func (c *debugConfig) enableLogVisualizer() {
	c.logVisualizer = true
}

func (c *debugConfig) addFileVisualizer(filename string) {
	c.visualizers = append(c.visualizers, func(_ string) {
		dotStr := c.graph.String()
		err := os.WriteFile(filename, []byte(dotStr), 0o600)
		if err != nil {
			c.logf("Error saving graphviz file %s: %+v", filename, err)
		} else {
			path, err := filepath.Abs(filename)
			if err == nil {
				c.logf("Saved graph of container to %s", path)
			}
		}
	})
}

func (c *debugConfig) locationGraphNode(location Location, key *moduleKey) *graphviz.Node {
	graph := c.moduleSubGraph(key)
	name := location.Name()
	node, found := graph.FindOrCreateNode(name)
	if found {
		return node
	}

	node.SetShape("box")
	setUnusedStyle(node.Attributes)
	return node
}

func (c *debugConfig) typeGraphNode(typ reflect.Type) *graphviz.Node {
	name := moreUsefulTypeString(typ)
	node, found := c.graph.FindOrCreateNode(name)
	if found {
		return node
	}

	setUnusedStyle(node.Attributes)
	return node
}

func setUnusedStyle(attr *graphviz.Attributes) {
	attr.SetColor("lightgrey")
	attr.SetPenWidth("0.5")
	attr.SetFontColor("dimgrey")
}

// moreUsefulTypeString is more useful than reflect.Type.String()
func moreUsefulTypeString(ty reflect.Type) string {
	switch ty.Kind() {
	case reflect.Struct, reflect.Interface:
		return fmt.Sprintf("%s.%s", ty.PkgPath(), ty.Name())
	case reflect.Pointer:
		return fmt.Sprintf("*%s", moreUsefulTypeString(ty.Elem()))
	case reflect.Map:
		return fmt.Sprintf("map[%s]%s", moreUsefulTypeString(ty.Key()), moreUsefulTypeString(ty.Elem()))
	case reflect.Slice:
		return fmt.Sprintf("[]%s", moreUsefulTypeString(ty.Elem()))
	default:
		return ty.String()
	}
}

func (c *debugConfig) moduleSubGraph(key *moduleKey) *graphviz.Graph {
	if key == nil {
		// return the root graph
		return c.graph
	}

	gname := fmt.Sprintf("cluster_%s", key.name)
	graph, found := c.graph.FindOrCreateSubGraph(gname)
	if !found {
		graph.SetLabel(fmt.Sprintf("Module: %s", key.name))
		graph.SetPenWidth("0.5")
		graph.SetFontSize("12.0")
		graph.SetStyle("rounded")
	}
	return graph
}

func (c *debugConfig) addGraphEdge(from, to *graphviz.Node) {
	_ = c.graph.CreateEdge(from, to)
}
