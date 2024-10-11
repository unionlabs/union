package statsd

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func TestFormatAppendTags(t *testing.T) {
	var buffer []byte
	buffer = appendTags(buffer, []string{"global:tag"}, []string{"tag:tag", "tag2:tag2"})
	assert.Equal(t, `|#global:tag,tag:tag,tag2:tag2`, string(buffer))

	var buffer2 []byte
	buffer2 = appendTags(buffer2, []string{"global:tag"}, nil)
	assert.Equal(t, `|#global:tag`, string(buffer2))

	var buffer3 []byte
	buffer3 = appendTags(buffer3, nil, []string{"tag:tag", "tag2:tag2"})
	assert.Equal(t, `|#tag:tag,tag2:tag2`, string(buffer3))

	var buffer4 []byte
	buffer4 = appendTags(buffer4, nil, nil)
	assert.Equal(t, "", string(buffer4))
}

func TestFormatAppendTagsAggregated(t *testing.T) {
	var buffer []byte
	buffer = appendTagsAggregated(buffer, []string{"global:tag"}, "tag:tag,tag2:tag2")
	assert.Equal(t, `|#global:tag,tag:tag,tag2:tag2`, string(buffer))

	var buffer2 []byte
	buffer2 = appendTagsAggregated(buffer2, []string{"global:tag"}, "")
	assert.Equal(t, `|#global:tag`, string(buffer2))

	var buffer3 []byte
	buffer3 = appendTagsAggregated(buffer3, nil, "tag:tag,tag2:tag2")
	assert.Equal(t, `|#tag:tag,tag2:tag2`, string(buffer3))

	var buffer4 []byte
	buffer4 = appendTagsAggregated(buffer4, nil, "")
	assert.Equal(t, "", string(buffer4))
}

func TestFormatAppendGauge(t *testing.T) {
	var buffer []byte
	buffer = appendGauge(buffer, "namespace.", []string{"global:tag"}, "gauge", 1., []string{"tag:tag"}, 1)
	assert.Equal(t, `namespace.gauge:1|g|#global:tag,tag:tag`, string(buffer))
}

func TestFormatAppendCount(t *testing.T) {
	var buffer []byte
	buffer = appendCount(buffer, "namespace.", []string{"global:tag"}, "count", 2, []string{"tag:tag"}, 1)
	assert.Equal(t, `namespace.count:2|c|#global:tag,tag:tag`, string(buffer))
}

func TestFormatAppendHistogram(t *testing.T) {
	var buffer []byte
	buffer = appendHistogram(buffer, "namespace.", []string{"global:tag"}, "histogram", 3., []string{"tag:tag"}, 1)
	assert.Equal(t, `namespace.histogram:3|h|#global:tag,tag:tag`, string(buffer))
}

func TestFormatAppendDistribution(t *testing.T) {
	var buffer []byte
	buffer = appendDistribution(buffer, "namespace.", []string{"global:tag"}, "distribution", 4., []string{"tag:tag"}, 1)
	assert.Equal(t, `namespace.distribution:4|d|#global:tag,tag:tag`, string(buffer))
}

func TestFormatAppendSet(t *testing.T) {
	var buffer []byte
	buffer = appendSet(buffer, "namespace.", []string{"global:tag"}, "set", "five", []string{"tag:tag"}, 1)
	assert.Equal(t, `namespace.set:five|s|#global:tag,tag:tag`, string(buffer))
}

func TestFormatAppendTiming(t *testing.T) {
	var buffer []byte
	buffer = appendTiming(buffer, "namespace.", []string{"global:tag"}, "timing", 6., []string{"tag:tag"}, 1)
	assert.Equal(t, `namespace.timing:6.000000|ms|#global:tag,tag:tag`, string(buffer))
}

func TestFormatNoTag(t *testing.T) {
	var buffer []byte
	buffer = appendGauge(buffer, "", []string{}, "gauge", 1., []string{}, 1)
	assert.Equal(t, `gauge:1|g`, string(buffer))
}

func TestFormatOneTag(t *testing.T) {
	var buffer []byte
	buffer = appendGauge(buffer, "", []string{}, "gauge", 1., []string{"tag1:tag1"}, 1)
	assert.Equal(t, `gauge:1|g|#tag1:tag1`, string(buffer))
}

func TestFormatTwoTag(t *testing.T) {
	var buffer []byte
	buffer = appendGauge(buffer, "", []string{}, "metric", 1., []string{"tag1:tag1", "tag2:tag2"}, 1)
	assert.Equal(t, `metric:1|g|#tag1:tag1,tag2:tag2`, string(buffer))
}

func TestFormatRate(t *testing.T) {
	var buffer []byte
	buffer = appendGauge(buffer, "", []string{}, "metric", 1., []string{}, 0.1)
	assert.Equal(t, `metric:1|g|@0.1`, string(buffer))
}

func TestFormatRateAndTag(t *testing.T) {
	var buffer []byte
	buffer = appendGauge(buffer, "", []string{}, "metric", 1., []string{"tag1:tag1"}, 0.1)
	assert.Equal(t, `metric:1|g|@0.1|#tag1:tag1`, string(buffer))
}

func TestFormatNil(t *testing.T) {
	var buffer []byte
	buffer = appendGauge(buffer, "", nil, "metric", 1., nil, 1)
	assert.Equal(t, `metric:1|g`, string(buffer))
}

func TestFormatTagRemoveNewLines(t *testing.T) {
	var buffer []byte
	buffer = appendGauge(buffer, "", []string{"tag\n:d\nog\n"}, "metric", 1., []string{"\ntag\n:d\nog2\n"}, 0.1)
	assert.Equal(t, `metric:1|g|@0.1|#tag:dog,tag:dog2`, string(buffer))
}

func TestFormatEvent(t *testing.T) {
	var buffer []byte
	buffer = appendEvent(buffer, Event{
		Title: "EvenTitle",
		Text:  "EventText",
	}, []string{})
	assert.Equal(t, `_e{9,9}:EvenTitle|EventText`, string(buffer))
}

func TestFormatEventEscapeText(t *testing.T) {
	var buffer []byte
	buffer = appendEvent(buffer, Event{
		Title: "EvenTitle",
		Text:  "\nEventText\nLine2\n\nLine4\n",
	}, []string{})
	assert.Equal(t, `_e{9,29}:EvenTitle|\nEventText\nLine2\n\nLine4\n`, string(buffer))
}

func TestFormatEventTimeStamp(t *testing.T) {
	var buffer []byte
	buffer = appendEvent(buffer, Event{
		Title:     "EvenTitle",
		Text:      "EventText",
		Timestamp: time.Date(2016, time.August, 15, 0, 0, 0, 0, time.UTC),
	}, []string{})
	assert.Equal(t, `_e{9,9}:EvenTitle|EventText|d:1471219200`, string(buffer))
}

func TestFormatEventHostname(t *testing.T) {
	var buffer []byte
	buffer = appendEvent(buffer, Event{
		Title:    "EvenTitle",
		Text:     "EventText",
		Hostname: "hostname",
	}, []string{})
	assert.Equal(t, `_e{9,9}:EvenTitle|EventText|h:hostname`, string(buffer))
}

func TestFormatEventAggregationKey(t *testing.T) {
	var buffer []byte
	buffer = appendEvent(buffer, Event{
		Title:          "EvenTitle",
		Text:           "EventText",
		AggregationKey: "aggregationKey",
	}, []string{})
	assert.Equal(t, `_e{9,9}:EvenTitle|EventText|k:aggregationKey`, string(buffer))
}

func TestFormatEventPriority(t *testing.T) {
	var buffer []byte
	buffer = appendEvent(buffer, Event{
		Title:    "EvenTitle",
		Text:     "EventText",
		Priority: "priority",
	}, []string{})
	assert.Equal(t, `_e{9,9}:EvenTitle|EventText|p:priority`, string(buffer))
}

func TestFormatEventSourceTypeName(t *testing.T) {
	var buffer []byte
	buffer = appendEvent(buffer, Event{
		Title:          "EvenTitle",
		Text:           "EventText",
		SourceTypeName: "sourceTypeName",
	}, []string{})
	assert.Equal(t, `_e{9,9}:EvenTitle|EventText|s:sourceTypeName`, string(buffer))
}

func TestFormatEventAlertType(t *testing.T) {
	var buffer []byte
	buffer = appendEvent(buffer, Event{
		Title:     "EvenTitle",
		Text:      "EventText",
		AlertType: "alertType",
	}, []string{})
	assert.Equal(t, `_e{9,9}:EvenTitle|EventText|t:alertType`, string(buffer))
}

func TestFormatEventOneTag(t *testing.T) {
	var buffer []byte
	buffer = appendEvent(buffer, Event{
		Title: "EvenTitle",
		Text:  "EventText",
	}, []string{"tag:test"})
	assert.Equal(t, `_e{9,9}:EvenTitle|EventText|#tag:test`, string(buffer))
}

func TestFormatEventTwoTag(t *testing.T) {
	var buffer []byte
	buffer = appendEvent(buffer, Event{
		Title: "EvenTitle",
		Text:  "EventText",
		Tags:  []string{"tag1:test"},
	}, []string{"tag2:test"})
	assert.Equal(t, `_e{9,9}:EvenTitle|EventText|#tag2:test,tag1:test`, string(buffer))
}

func TestFormatEventAllOptions(t *testing.T) {
	var buffer []byte
	buffer = appendEvent(buffer, Event{
		Title:          "EvenTitle",
		Text:           "EventText",
		Timestamp:      time.Date(2016, time.August, 15, 0, 0, 0, 0, time.UTC),
		Hostname:       "hostname",
		AggregationKey: "aggregationKey",
		Priority:       "priority",
		SourceTypeName: "SourceTypeName",
		AlertType:      "alertType",
		Tags:           []string{"tag:normal"},
	}, []string{"tag:global"})
	assert.Equal(t, `_e{9,9}:EvenTitle|EventText|d:1471219200|h:hostname|k:aggregationKey|p:priority|s:SourceTypeName|t:alertType|#tag:global,tag:normal`, string(buffer))
}

func TestFormatEventNil(t *testing.T) {
	var buffer []byte
	buffer = appendEvent(buffer, Event{}, []string{})
	assert.Equal(t, `_e{0,0}:|`, string(buffer))
}

func TestFormatServiceCheck(t *testing.T) {
	var buffer []byte
	buffer = appendServiceCheck(buffer, ServiceCheck{
		Name:   "service.check",
		Status: Ok,
	}, []string{})
	assert.Equal(t, `_sc|service.check|0`, string(buffer))
}

func TestFormatServiceCheckEscape(t *testing.T) {
	var buffer []byte
	buffer = appendServiceCheck(buffer, ServiceCheck{
		Name:    "service.check",
		Status:  Ok,
		Message: "\n\nmessagem:hello...\n\nm:aa\nm:m",
	}, []string{})
	assert.Equal(t, `_sc|service.check|0|m:\n\nmessagem\:hello...\n\nm\:aa\nm\:m`, string(buffer))
}

func TestFormatServiceCheckTimestamp(t *testing.T) {
	var buffer []byte
	buffer = appendServiceCheck(buffer, ServiceCheck{
		Name:      "service.check",
		Status:    Ok,
		Timestamp: time.Date(2016, time.August, 15, 0, 0, 0, 0, time.UTC),
	}, []string{})
	assert.Equal(t, `_sc|service.check|0|d:1471219200`, string(buffer))
}

func TestFormatServiceCheckHostname(t *testing.T) {
	var buffer []byte
	buffer = appendServiceCheck(buffer, ServiceCheck{
		Name:     "service.check",
		Status:   Ok,
		Hostname: "hostname",
	}, []string{})
	assert.Equal(t, `_sc|service.check|0|h:hostname`, string(buffer))
}

func TestFormatServiceCheckMessage(t *testing.T) {
	var buffer []byte
	buffer = appendServiceCheck(buffer, ServiceCheck{
		Name:    "service.check",
		Status:  Ok,
		Message: "message",
	}, []string{})
	assert.Equal(t, `_sc|service.check|0|m:message`, string(buffer))
}

func TestFormatServiceCheckOneTag(t *testing.T) {
	var buffer []byte
	buffer = appendServiceCheck(buffer, ServiceCheck{
		Name:   "service.check",
		Status: Ok,
		Tags:   []string{"tag:tag"},
	}, []string{})
	assert.Equal(t, `_sc|service.check|0|#tag:tag`, string(buffer))
}

func TestFormatServiceCheckTwoTag(t *testing.T) {
	var buffer []byte
	buffer = appendServiceCheck(buffer, ServiceCheck{
		Name:   "service.check",
		Status: Ok,
		Tags:   []string{"tag1:tag1"},
	}, []string{"tag2:tag2"})
	assert.Equal(t, `_sc|service.check|0|#tag2:tag2,tag1:tag1`, string(buffer))
}

func TestFormatServiceCheckAllOptions(t *testing.T) {
	var buffer []byte
	buffer = appendServiceCheck(buffer, ServiceCheck{
		Name:      "service.check",
		Status:    Ok,
		Timestamp: time.Date(2016, time.August, 15, 0, 0, 0, 0, time.UTC),
		Hostname:  "hostname",
		Message:   "message",
		Tags:      []string{"tag1:tag1"},
	}, []string{"tag2:tag2"})
	assert.Equal(t, `_sc|service.check|0|d:1471219200|h:hostname|#tag2:tag2,tag1:tag1|m:message`, string(buffer))
}

func TestFormatServiceCheckNil(t *testing.T) {
	var buffer []byte
	buffer = appendServiceCheck(buffer, ServiceCheck{}, nil)
	assert.Equal(t, `_sc||0`, string(buffer))
}

func TestFormatSeparator(t *testing.T) {
	var buffer []byte
	buffer = appendSeparator(buffer)
	assert.Equal(t, "\n", string(buffer))
}
