package statsd

import (
	"strconv"
	"strings"
)

var (
	gaugeSymbol        = []byte("g")
	countSymbol        = []byte("c")
	histogramSymbol    = []byte("h")
	distributionSymbol = []byte("d")
	setSymbol          = []byte("s")
	timingSymbol       = []byte("ms")
)

func appendHeader(buffer []byte, namespace string, name string) []byte {
	if namespace != "" {
		buffer = append(buffer, namespace...)
	}
	buffer = append(buffer, name...)
	buffer = append(buffer, ':')
	return buffer
}

func appendRate(buffer []byte, rate float64) []byte {
	if rate < 1 {
		buffer = append(buffer, "|@"...)
		buffer = strconv.AppendFloat(buffer, rate, 'f', -1, 64)
	}
	return buffer
}

func appendWithoutNewlines(buffer []byte, s string) []byte {
	// fastpath for strings without newlines
	if strings.IndexByte(s, '\n') == -1 {
		return append(buffer, s...)
	}

	for _, b := range []byte(s) {
		if b != '\n' {
			buffer = append(buffer, b)
		}
	}
	return buffer
}

func appendTags(buffer []byte, globalTags []string, tags []string) []byte {
	if len(globalTags) == 0 && len(tags) == 0 {
		return buffer
	}
	buffer = append(buffer, "|#"...)
	firstTag := true

	for _, tag := range globalTags {
		if !firstTag {
			buffer = append(buffer, ',')
		}
		buffer = appendWithoutNewlines(buffer, tag)
		firstTag = false
	}
	for _, tag := range tags {
		if !firstTag {
			buffer = append(buffer, ',')
		}
		buffer = appendWithoutNewlines(buffer, tag)
		firstTag = false
	}
	return buffer
}

func appendFloatMetric(buffer []byte, typeSymbol []byte, namespace string, globalTags []string, name string, value float64, tags []string, rate float64, precision int) []byte {
	buffer = appendHeader(buffer, namespace, name)
	buffer = strconv.AppendFloat(buffer, value, 'f', precision, 64)
	buffer = append(buffer, '|')
	buffer = append(buffer, typeSymbol...)
	buffer = appendRate(buffer, rate)
	buffer = appendTags(buffer, globalTags, tags)
	return buffer
}

func appendIntegerMetric(buffer []byte, typeSymbol []byte, namespace string, globalTags []string, name string, value int64, tags []string, rate float64) []byte {
	buffer = appendHeader(buffer, namespace, name)
	buffer = strconv.AppendInt(buffer, value, 10)
	buffer = append(buffer, '|')
	buffer = append(buffer, typeSymbol...)
	buffer = appendRate(buffer, rate)
	buffer = appendTags(buffer, globalTags, tags)
	return buffer
}

func appendStringMetric(buffer []byte, typeSymbol []byte, namespace string, globalTags []string, name string, value string, tags []string, rate float64) []byte {
	buffer = appendHeader(buffer, namespace, name)
	buffer = append(buffer, value...)
	buffer = append(buffer, '|')
	buffer = append(buffer, typeSymbol...)
	buffer = appendRate(buffer, rate)
	buffer = appendTags(buffer, globalTags, tags)
	return buffer
}

func appendGauge(buffer []byte, namespace string, globalTags []string, name string, value float64, tags []string, rate float64) []byte {
	return appendFloatMetric(buffer, gaugeSymbol, namespace, globalTags, name, value, tags, rate, -1)
}

func appendCount(buffer []byte, namespace string, globalTags []string, name string, value int64, tags []string, rate float64) []byte {
	return appendIntegerMetric(buffer, countSymbol, namespace, globalTags, name, value, tags, rate)
}

func appendHistogram(buffer []byte, namespace string, globalTags []string, name string, value float64, tags []string, rate float64) []byte {
	return appendFloatMetric(buffer, histogramSymbol, namespace, globalTags, name, value, tags, rate, -1)
}

func appendDistribution(buffer []byte, namespace string, globalTags []string, name string, value float64, tags []string, rate float64) []byte {
	return appendFloatMetric(buffer, distributionSymbol, namespace, globalTags, name, value, tags, rate, -1)
}

func appendSet(buffer []byte, namespace string, globalTags []string, name string, value string, tags []string, rate float64) []byte {
	return appendStringMetric(buffer, setSymbol, namespace, globalTags, name, value, tags, rate)
}

func appendTiming(buffer []byte, namespace string, globalTags []string, name string, value float64, tags []string, rate float64) []byte {
	return appendFloatMetric(buffer, timingSymbol, namespace, globalTags, name, value, tags, rate, 6)
}

func escapedEventTextLen(text string) int {
	return len(text) + strings.Count(text, "\n")
}

func appendEscapedEventText(buffer []byte, text string) []byte {
	for _, b := range []byte(text) {
		if b != '\n' {
			buffer = append(buffer, b)
		} else {
			buffer = append(buffer, "\\n"...)
		}
	}
	return buffer
}

func appendEvent(buffer []byte, event Event, globalTags []string) []byte {
	escapedTextLen := escapedEventTextLen(event.Text)

	buffer = append(buffer, "_e{"...)
	buffer = strconv.AppendInt(buffer, int64(len(event.Title)), 10)
	buffer = append(buffer, ',')
	buffer = strconv.AppendInt(buffer, int64(escapedTextLen), 10)
	buffer = append(buffer, "}:"...)
	buffer = append(buffer, event.Title...)
	buffer = append(buffer, '|')
	if escapedTextLen != len(event.Text) {
		buffer = appendEscapedEventText(buffer, event.Text)
	} else {
		buffer = append(buffer, event.Text...)
	}

	if !event.Timestamp.IsZero() {
		buffer = append(buffer, "|d:"...)
		buffer = strconv.AppendInt(buffer, int64(event.Timestamp.Unix()), 10)
	}

	if len(event.Hostname) != 0 {
		buffer = append(buffer, "|h:"...)
		buffer = append(buffer, event.Hostname...)
	}

	if len(event.AggregationKey) != 0 {
		buffer = append(buffer, "|k:"...)
		buffer = append(buffer, event.AggregationKey...)
	}

	if len(event.Priority) != 0 {
		buffer = append(buffer, "|p:"...)
		buffer = append(buffer, event.Priority...)
	}

	if len(event.SourceTypeName) != 0 {
		buffer = append(buffer, "|s:"...)
		buffer = append(buffer, event.SourceTypeName...)
	}

	if len(event.AlertType) != 0 {
		buffer = append(buffer, "|t:"...)
		buffer = append(buffer, string(event.AlertType)...)
	}

	buffer = appendTags(buffer, globalTags, event.Tags)
	return buffer
}

func appendEscapedServiceCheckText(buffer []byte, text string) []byte {
	for i := 0; i < len(text); i++ {
		if text[i] == '\n' {
			buffer = append(buffer, "\\n"...)
		} else if text[i] == 'm' && i+1 < len(text) && text[i+1] == ':' {
			buffer = append(buffer, "m\\:"...)
			i++
		} else {
			buffer = append(buffer, text[i])
		}
	}
	return buffer
}

func appendServiceCheck(buffer []byte, serviceCheck ServiceCheck, globalTags []string) []byte {
	buffer = append(buffer, "_sc|"...)
	buffer = append(buffer, serviceCheck.Name...)
	buffer = append(buffer, '|')
	buffer = strconv.AppendInt(buffer, int64(serviceCheck.Status), 10)

	if !serviceCheck.Timestamp.IsZero() {
		buffer = append(buffer, "|d:"...)
		buffer = strconv.AppendInt(buffer, int64(serviceCheck.Timestamp.Unix()), 10)
	}

	if len(serviceCheck.Hostname) != 0 {
		buffer = append(buffer, "|h:"...)
		buffer = append(buffer, serviceCheck.Hostname...)
	}

	buffer = appendTags(buffer, globalTags, serviceCheck.Tags)

	if len(serviceCheck.Message) != 0 {
		buffer = append(buffer, "|m:"...)
		buffer = appendEscapedServiceCheckText(buffer, serviceCheck.Message)
	}
	return buffer
}

func appendSeparator(buffer []byte) []byte {
	return append(buffer, '\n')
}
