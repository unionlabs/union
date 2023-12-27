package sentry

import "time"

type CheckInStatus string

const (
	CheckInStatusInProgress CheckInStatus = "in_progress"
	CheckInStatusOK         CheckInStatus = "ok"
	CheckInStatusError      CheckInStatus = "error"
)

type checkInScheduleType string

const (
	checkInScheduleTypeCrontab  checkInScheduleType = "crontab"
	checkInScheduleTypeInterval checkInScheduleType = "interval"
)

type MonitorSchedule interface {
	// scheduleType is a private method that must be implemented for monitor schedule
	// implementation. It should never be called. This method is made for having
	// specific private implementation of MonitorSchedule interface.
	scheduleType() checkInScheduleType
}

type crontabSchedule struct {
	Type  string `json:"type"`
	Value string `json:"value"`
}

func (c crontabSchedule) scheduleType() checkInScheduleType {
	return checkInScheduleTypeCrontab
}

// CrontabSchedule defines the MonitorSchedule with a cron format.
// Example: "8 * * * *".
func CrontabSchedule(scheduleString string) MonitorSchedule {
	return crontabSchedule{
		Type:  string(checkInScheduleTypeCrontab),
		Value: scheduleString,
	}
}

type intervalSchedule struct {
	Type  string `json:"type"`
	Value int64  `json:"value"`
	Unit  string `json:"unit"`
}

func (i intervalSchedule) scheduleType() checkInScheduleType {
	return checkInScheduleTypeInterval
}

type MonitorScheduleUnit string

const (
	MonitorScheduleUnitMinute MonitorScheduleUnit = "minute"
	MonitorScheduleUnitHour   MonitorScheduleUnit = "hour"
	MonitorScheduleUnitDay    MonitorScheduleUnit = "day"
	MonitorScheduleUnitWeek   MonitorScheduleUnit = "week"
	MonitorScheduleUnitMonth  MonitorScheduleUnit = "month"
	MonitorScheduleUnitYear   MonitorScheduleUnit = "year"
)

// IntervalSchedule defines the MonitorSchedule with an interval format.
//
// Example:
//
//	IntervalSchedule(1, sentry.MonitorScheduleUnitDay)
func IntervalSchedule(value int64, unit MonitorScheduleUnit) MonitorSchedule {
	return intervalSchedule{
		Type:  string(checkInScheduleTypeInterval),
		Value: value,
		Unit:  string(unit),
	}
}

type MonitorConfig struct { //nolint: maligned // prefer readability over optimal memory layout
	Schedule MonitorSchedule `json:"schedule,omitempty"`
	// The allowed margin of minutes after the expected check-in time that
	// the monitor will not be considered missed for.
	CheckInMargin int64 `json:"checkin_margin,omitempty"`
	// The allowed duration in minutes that the monitor may be `in_progress`
	// for before being considered failed due to timeout.
	MaxRuntime int64 `json:"max_runtime,omitempty"`
	// A tz database string representing the timezone which the monitor's execution schedule is in.
	// See: https://en.wikipedia.org/wiki/List_of_tz_database_time_zones
	Timezone string `json:"timezone,omitempty"`
}

type CheckIn struct { //nolint: maligned // prefer readability over optimal memory layout
	// Check-In ID (unique and client generated)
	ID EventID `json:"check_in_id"`
	// The distinct slug of the monitor.
	MonitorSlug string `json:"monitor_slug"`
	// The status of the check-in.
	Status CheckInStatus `json:"status"`
	// The duration of the check-in. Will only take effect if the status is ok or error.
	Duration time.Duration `json:"duration,omitempty"`
}

// serializedCheckIn is used by checkInMarshalJSON method on Event struct.
// See https://develop.sentry.dev/sdk/check-ins/
type serializedCheckIn struct { //nolint: maligned
	// Check-In ID (unique and client generated).
	CheckInID string `json:"check_in_id"`
	// The distinct slug of the monitor.
	MonitorSlug string `json:"monitor_slug"`
	// The status of the check-in.
	Status CheckInStatus `json:"status"`
	// The duration of the check-in in seconds. Will only take effect if the status is ok or error.
	Duration      float64        `json:"duration,omitempty"`
	Release       string         `json:"release,omitempty"`
	Environment   string         `json:"environment,omitempty"`
	MonitorConfig *MonitorConfig `json:"monitor_config,omitempty"`
}
