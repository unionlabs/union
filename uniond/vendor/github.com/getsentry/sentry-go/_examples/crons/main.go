package main

import (
	"fmt"
	"time"

	"github.com/getsentry/sentry-go"
)

func runTask(monitorSlug string, duration time.Duration, shouldFail bool) {
	checkinId := sentry.CaptureCheckIn(
		&sentry.CheckIn{
			MonitorSlug: monitorSlug,
			Status:      sentry.CheckInStatusInProgress,
		},
		&sentry.MonitorConfig{
			Schedule:      sentry.CrontabSchedule("* * * * *"),
			MaxRuntime:    2,
			CheckInMargin: 1,
		},
	)
	task := fmt.Sprintf("Task[monitor_slug=%s,id=%s]", monitorSlug, *checkinId)
	fmt.Printf("Task started: %s\n", task)

	time.Sleep(duration)

	var status sentry.CheckInStatus
	if shouldFail {
		status = sentry.CheckInStatusError
	} else {
		status = sentry.CheckInStatusOK
	}

	sentry.CaptureCheckIn(
		&sentry.CheckIn{
			ID:          *checkinId,
			MonitorSlug: monitorSlug,
			Status:      status,
		},
		nil,
	)
	fmt.Printf("Task finished: %s; Status: %s\n", task, status)
}

func main() {
	_ = sentry.Init(sentry.ClientOptions{
		Dsn:   "",
		Debug: true,
	})

	// Start a task that runs every minute and always succeeds
	go func() {
		for {
			go runTask("sentry-go-periodic-task-success", time.Second, false)
			time.Sleep(time.Minute)
		}
	}()

	time.Sleep(3 * time.Second)

	// Start a task that runs every minute and fails every second time
	go func() {
		shouldFail := true
		for {
			go runTask("sentry-go-periodic-task-sometimes-fail", 2*time.Second, shouldFail)
			time.Sleep(time.Minute)
			shouldFail = !shouldFail
		}
	}()

	select {}
}
