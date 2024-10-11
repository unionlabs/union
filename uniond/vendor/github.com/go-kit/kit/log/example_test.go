package log_test

import (
	"math/rand"
	"os"
	"sync"
	"time"

	"github.com/go-kit/kit/log"
)

func Example_basic() {
	logger := log.NewLogfmtLogger(os.Stdout)

	type Task struct {
		ID int
	}

	RunTask := func(task Task, logger log.Logger) {
		logger.Log("taskID", task.ID, "event", "starting task")

		logger.Log("taskID", task.ID, "event", "task complete")
	}

	RunTask(Task{ID: 1}, logger)

	// Output:
	// taskID=1 event="starting task"
	// taskID=1 event="task complete"
}

func Example_contextual() {
	logger := log.NewLogfmtLogger(os.Stdout)

	type Task struct {
		ID  int
		Cmd string
	}

	taskHelper := func(cmd string, logger log.Logger) {
		// execute(cmd)
		logger.Log("cmd", cmd, "dur", 42*time.Millisecond)
	}

	RunTask := func(task Task, logger log.Logger) {
		logger = log.With(logger, "taskID", task.ID)
		logger.Log("event", "starting task")

		taskHelper(task.Cmd, logger)

		logger.Log("event", "task complete")
	}

	RunTask(Task{ID: 1, Cmd: "echo Hello, world!"}, logger)

	// Output:
	// taskID=1 event="starting task"
	// taskID=1 cmd="echo Hello, world!" dur=42ms
	// taskID=1 event="task complete"
}

func Example_valuer() {
	logger := log.NewLogfmtLogger(os.Stdout)

	count := 0
	counter := func() interface{} {
		count++
		return count
	}

	logger = log.With(logger, "count", log.Valuer(counter))

	logger.Log("call", "first")
	logger.Log("call", "second")

	// Output:
	// count=1 call=first
	// count=2 call=second
}

func Example_debugInfo() {
	logger := log.NewLogfmtLogger(os.Stdout)

	// make time predictable for this test
	baseTime := time.Date(2015, time.February, 3, 10, 0, 0, 0, time.UTC)
	mockTime := func() time.Time {
		baseTime = baseTime.Add(time.Second)
		return baseTime
	}

	logger = log.With(logger, "time", log.Timestamp(mockTime), "caller", log.DefaultCaller)

	logger.Log("call", "first")
	logger.Log("call", "second")

	// ...

	logger.Log("call", "third")

	// Output:
	// time=2015-02-03T10:00:01Z caller=example_test.go:93 call=first
	// time=2015-02-03T10:00:02Z caller=example_test.go:94 call=second
	// time=2015-02-03T10:00:03Z caller=example_test.go:98 call=third
}

func Example_syncWriter() {
	w := log.NewSyncWriter(os.Stdout)
	logger := log.NewLogfmtLogger(w)

	type Task struct {
		ID int
	}

	var wg sync.WaitGroup

	RunTask := func(task Task, logger log.Logger) {
		logger.Log("taskID", task.ID, "event", "starting task")

		time.Sleep(time.Duration(rand.Intn(200)) * time.Millisecond)

		logger.Log("taskID", task.ID, "event", "task complete")
		wg.Done()
	}

	wg.Add(2)

	go RunTask(Task{ID: 1}, logger)
	go RunTask(Task{ID: 2}, logger)

	wg.Wait()

	// Unordered output:
	// taskID=1 event="starting task"
	// taskID=2 event="starting task"
	// taskID=1 event="task complete"
	// taskID=2 event="task complete"
}
