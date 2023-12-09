package textual

import (
	"context"
	"fmt"
	"regexp"
	"strconv"
	"strings"

	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
	dpb "google.golang.org/protobuf/types/known/durationpb"
)

type durationValueRenderer struct{}

// NewDurationValueRenderer returns a ValueRenderer for protocol buffer Duration messages.
// It renders durations by grouping seconds into units of days (86400s), hours (3600s),
// and minutes(60s), plus the total seconds elapsed. E.g. a duration of 1483530s is
// formatted as "17 days, 4 hours, 5 minutes, 30 seconds".
// Note that the days are always 24 hours regardless of daylight savings changes.
func NewDurationValueRenderer() ValueRenderer {
	return durationValueRenderer{}
}

const (
	minSec  = 60
	hourSec = 60 * minSec
	daySec  = 24 * hourSec
)

type factors struct {
	days, hours, minutes, seconds int64
}

func factorSeconds(x int64) factors {
	var f factors
	f.days = x / daySec
	x -= f.days * daySec
	f.hours = x / hourSec
	x -= f.hours * hourSec
	f.minutes = x / minSec
	x -= f.minutes * minSec
	f.seconds = x
	return f
}

func maybePlural(s string, plural bool) string {
	if plural {
		return s + "s"
	}
	return s
}

func formatSeconds(seconds int64, nanos int32) string {
	var s string
	if nanos == 0 {
		s = fmt.Sprintf("%d", seconds)
	} else {
		frac := fmt.Sprintf("%09d", nanos)
		frac = strings.TrimRight(frac, "0")
		s = fmt.Sprintf("%d.%s", seconds, frac)
	}
	return s
}

// Format implements the ValueRenderer interface.
func (dr durationValueRenderer) Format(_ context.Context, v protoreflect.Value) ([]Screen, error) {
	// Reify the reflected message as a proto Duration
	msg := v.Message().Interface()
	duration := &dpb.Duration{}
	err := coerceToMessage(msg, duration)
	if err != nil {
		return nil, err
	}

	// Bypass use of time.Duration, as the range is more limited than that of dpb.Duration.
	// (Too bad the companies that produced both technologies didn't coordinate better!)

	if err := duration.CheckValid(); err != nil {
		return nil, err
	}

	negative := false
	if duration.Seconds < 0 || duration.Nanos < 0 {
		negative = true
		// copy to avoid side-effecting our input
		duration = proto.Clone(duration).(*dpb.Duration)
		duration.Seconds *= -1
		duration.Nanos *= -1
	}
	factors := factorSeconds(duration.Seconds)
	components := []string{}

	if factors.days > 0 {
		components = append(components, fmt.Sprintf("%d %s", factors.days, maybePlural("day", factors.days != 1)))
	}
	if factors.hours > 0 || (len(components) > 0 && (factors.minutes > 0 || factors.seconds > 0 || duration.Nanos > 0)) {
		components = append(components, fmt.Sprintf("%d %s", factors.hours, maybePlural("hour", factors.hours != 1)))
	}
	if factors.minutes > 0 || (len(components) > 0 && (factors.seconds > 0 || duration.Nanos > 0)) {
		components = append(components, fmt.Sprintf("%d %s", factors.minutes, maybePlural("minute", factors.minutes != 1)))
	}
	if factors.seconds > 0 || duration.Nanos > 0 {
		components = append(components, formatSeconds(factors.seconds, duration.Nanos)+" "+maybePlural("second", factors.seconds != 1 || duration.Nanos > 0))
	}

	s := strings.Join(components, ", ")

	if s == "" {
		s = "0 seconds"
	}

	if negative {
		s = "-" + s
	}

	return []Screen{{Content: s}}, nil
}

var durRegexp = regexp.MustCompile(`^(-)?(?:([0-9]+) days?)?(?:, )?(?:([0-9]+) hours?)?(?:, )?(?:([0-9]+) minutes?)?(?:, )?(?:([0-9]+)(?:\.([0-9]+))? seconds?)?$`)

// Parse implements the ValueRenderer interface.
func (dr durationValueRenderer) Parse(_ context.Context, screens []Screen) (protoreflect.Value, error) {
	if len(screens) != 1 {
		return nilValue, fmt.Errorf("expected single screen: %v", screens)
	}

	parts := durRegexp.FindStringSubmatch(screens[0].Content)
	if parts == nil {
		return nilValue, fmt.Errorf("bad duration format: %s", screens[0].Content)
	}

	isNegative := parts[1] == "-"
	var days, hours, minutes, seconds, nanos int64
	var err error

	if parts[2] != "" {
		days, err = strconv.ParseInt(parts[2], 10, 64)
		if err != nil {
			return nilValue, fmt.Errorf(`bad number "%s": %w`, parts[2], err)
		}
	}
	if parts[3] != "" {
		hours, err = strconv.ParseInt(parts[3], 10, 64)
		if err != nil {
			return nilValue, fmt.Errorf(`bad number "%s": %w`, parts[3], err)
		}
	}
	if parts[4] != "" {
		minutes, err = strconv.ParseInt(parts[4], 10, 64)
		if err != nil {
			return nilValue, fmt.Errorf(`bad number "%s": %w`, parts[4], err)
		}
	}
	if parts[5] != "" {
		seconds, err = strconv.ParseInt(parts[5], 10, 64)
		if err != nil {
			return nilValue, fmt.Errorf(`bad number "%s": %w`, parts[5], err)
		}
		if parts[6] != "" {
			if len(parts[6]) > 9 {
				return nilValue, fmt.Errorf(`too many nanos "%s"`, parts[6])
			}
			addZeros := 9 - len(parts[6])
			text := parts[6] + strings.Repeat("0", addZeros)
			nanos, err = strconv.ParseInt(text, 10, 32)
			if err != nil {
				return nilValue, fmt.Errorf(`bad number "%s": %w`, text, err)
			}
		}
	}

	dur := &dpb.Duration{}
	dur.Seconds = days*daySec + hours*hourSec + minutes*minSec + seconds
	// #nosec G701
	// Since there are 9 digits or fewer, this conversion is safe.
	dur.Nanos = int32(nanos)

	if isNegative {
		dur.Seconds *= -1
		dur.Nanos *= -1
	}

	msg := dur.ProtoReflect()
	return protoreflect.ValueOfMessage(msg), nil
}
