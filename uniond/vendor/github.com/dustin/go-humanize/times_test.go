package humanize

import (
	"math"
	"testing"
	"time"
)

func TestPast(t *testing.T) {
	now := time.Now()
	testList{
		{"now", Time(now), "now"},
		{"1 second ago", Time(now.Add(-1 * time.Second)), "1 second ago"},
		{"12 seconds ago", Time(now.Add(-12 * time.Second)), "12 seconds ago"},
		{"30 seconds ago", Time(now.Add(-30 * time.Second)), "30 seconds ago"},
		{"45 seconds ago", Time(now.Add(-45 * time.Second)), "45 seconds ago"},
		{"1 minute ago", Time(now.Add(-63 * time.Second)), "1 minute ago"},
		{"15 minutes ago", Time(now.Add(-15 * time.Minute)), "15 minutes ago"},
		{"1 hour ago", Time(now.Add(-63 * time.Minute)), "1 hour ago"},
		{"2 hours ago", Time(now.Add(-2 * time.Hour)), "2 hours ago"},
		{"21 hours ago", Time(now.Add(-21 * time.Hour)), "21 hours ago"},
		{"1 day ago", Time(now.Add(-26 * time.Hour)), "1 day ago"},
		{"2 days ago", Time(now.Add(-49 * time.Hour)), "2 days ago"},
		{"3 days ago", Time(now.Add(-3 * Day)), "3 days ago"},
		{"1 week ago (1)", Time(now.Add(-7 * Day)), "1 week ago"},
		{"1 week ago (2)", Time(now.Add(-12 * Day)), "1 week ago"},
		{"2 weeks ago", Time(now.Add(-15 * Day)), "2 weeks ago"},
		{"1 month ago", Time(now.Add(-39 * Day)), "1 month ago"},
		{"3 months ago", Time(now.Add(-99 * Day)), "3 months ago"},
		{"1 year ago (1)", Time(now.Add(-365 * Day)), "1 year ago"},
		{"1 year ago (1)", Time(now.Add(-400 * Day)), "1 year ago"},
		{"2 years ago (1)", Time(now.Add(-548 * Day)), "2 years ago"},
		{"2 years ago (2)", Time(now.Add(-725 * Day)), "2 years ago"},
		{"2 years ago (3)", Time(now.Add(-800 * Day)), "2 years ago"},
		{"3 years ago", Time(now.Add(-3 * Year)), "3 years ago"},
		{"long ago", Time(now.Add(-LongTime)), "a long while ago"},
	}.validate(t)
}

func TestReltimeOffbyone(t *testing.T) {
	testList{
		{"1w-1", RelTime(time.Unix(0, 0), time.Unix(7*24*60*60, -1), "ago", ""), "6 days ago"},
		{"1w±0", RelTime(time.Unix(0, 0), time.Unix(7*24*60*60, 0), "ago", ""), "1 week ago"},
		{"1w+1", RelTime(time.Unix(0, 0), time.Unix(7*24*60*60, 1), "ago", ""), "1 week ago"},
		{"2w-1", RelTime(time.Unix(0, 0), time.Unix(14*24*60*60, -1), "ago", ""), "1 week ago"},
		{"2w±0", RelTime(time.Unix(0, 0), time.Unix(14*24*60*60, 0), "ago", ""), "2 weeks ago"},
		{"2w+1", RelTime(time.Unix(0, 0), time.Unix(14*24*60*60, 1), "ago", ""), "2 weeks ago"},
	}.validate(t)
}

func TestFuture(t *testing.T) {
	// Add a little time so that these things properly line up in
	// the future.
	now := time.Now().Add(time.Millisecond * 250)
	testList{
		{"now", Time(now), "now"},
		{"1 second from now", Time(now.Add(+1 * time.Second)), "1 second from now"},
		{"12 seconds from now", Time(now.Add(+12 * time.Second)), "12 seconds from now"},
		{"30 seconds from now", Time(now.Add(+30 * time.Second)), "30 seconds from now"},
		{"45 seconds from now", Time(now.Add(+45 * time.Second)), "45 seconds from now"},
		{"15 minutes from now", Time(now.Add(+15 * time.Minute)), "15 minutes from now"},
		{"2 hours from now", Time(now.Add(+2 * time.Hour)), "2 hours from now"},
		{"21 hours from now", Time(now.Add(+21 * time.Hour)), "21 hours from now"},
		{"1 day from now", Time(now.Add(+26 * time.Hour)), "1 day from now"},
		{"2 days from now", Time(now.Add(+49 * time.Hour)), "2 days from now"},
		{"3 days from now", Time(now.Add(+3 * Day)), "3 days from now"},
		{"1 week from now (1)", Time(now.Add(+7 * Day)), "1 week from now"},
		{"1 week from now (2)", Time(now.Add(+12 * Day)), "1 week from now"},
		{"2 weeks from now", Time(now.Add(+15 * Day)), "2 weeks from now"},
		{"1 month from now", Time(now.Add(+30 * Day)), "1 month from now"},
		{"1 year from now", Time(now.Add(+365 * Day)), "1 year from now"},
		{"2 years from now", Time(now.Add(+2 * Year)), "2 years from now"},
		{"a while from now", Time(now.Add(+LongTime)), "a long while from now"},
	}.validate(t)
}

func TestRange(t *testing.T) {
	start := time.Time{}
	end := time.Unix(math.MaxInt64, math.MaxInt64)
	x := RelTime(start, end, "ago", "from now")
	if x != "a long while from now" {
		t.Errorf("Expected a long while from now, got %q", x)
	}
}

func TestCustomRelTime(t *testing.T) {
	now := time.Now().Add(time.Millisecond * 250)
	magnitudes := []RelTimeMagnitude{
		{time.Second, "now", time.Second},
		{2 * time.Second, "1 second %s", 1},
		{time.Minute, "%d seconds %s", time.Second},
		{Day - time.Second, "%d minutes %s", time.Minute},
		{Day, "%d hours %s", time.Hour},
		{2 * Day, "1 day %s", 1},
		{Week, "%d days %s", Day},
		{2 * Week, "1 week %s", 1},
		{6 * Month, "%d weeks %s", Week},
		{Year, "%d months %s", Month},
	}
	customRelTime := func(then time.Time) string {
		return CustomRelTime(then, time.Now(), "ago", "from now", magnitudes)
	}
	testList{
		{"now", customRelTime(now), "now"},
		{"1 second from now", customRelTime(now.Add(+1 * time.Second)), "1 second from now"},
		{"12 seconds from now", customRelTime(now.Add(+12 * time.Second)), "12 seconds from now"},
		{"30 seconds from now", customRelTime(now.Add(+30 * time.Second)), "30 seconds from now"},
		{"45 seconds from now", customRelTime(now.Add(+45 * time.Second)), "45 seconds from now"},
		{"15 minutes from now", customRelTime(now.Add(+15 * time.Minute)), "15 minutes from now"},
		{"2 hours from now", customRelTime(now.Add(+2 * time.Hour)), "120 minutes from now"},
		{"21 hours from now", customRelTime(now.Add(+21 * time.Hour)), "1260 minutes from now"},
		{"1 day from now", customRelTime(now.Add(+26 * time.Hour)), "1 day from now"},
		{"2 days from now", customRelTime(now.Add(+49 * time.Hour)), "2 days from now"},
		{"3 days from now", customRelTime(now.Add(+3 * Day)), "3 days from now"},
		{"1 week from now (1)", customRelTime(now.Add(+7 * Day)), "1 week from now"},
		{"1 week from now (2)", customRelTime(now.Add(+12 * Day)), "1 week from now"},
		{"2 weeks from now", customRelTime(now.Add(+15 * Day)), "2 weeks from now"},
		{"1 month from now", customRelTime(now.Add(+30 * Day)), "4 weeks from now"},
		{"6 months from now", customRelTime(now.Add(+6*Month - time.Second)), "25 weeks from now"},
		{"1 year from now", customRelTime(now.Add(+365 * Day)), "12 months from now"},
		{"2 years from now", customRelTime(now.Add(+2 * Year)), "24 months from now"},
		{"a while from now", customRelTime(now.Add(+LongTime)), "444 months from now"},
	}.validate(t)
}
