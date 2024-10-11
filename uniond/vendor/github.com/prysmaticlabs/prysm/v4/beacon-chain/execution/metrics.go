package execution

import (
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
)

var (
	newPayloadLatency = promauto.NewHistogram(
		prometheus.HistogramOpts{
			Name:    "new_payload_v1_latency_milliseconds",
			Help:    "Captures RPC latency for newPayloadV1 in milliseconds",
			Buckets: []float64{25, 50, 100, 200, 500, 1000, 2000, 4000},
		},
	)
	getPayloadLatency = promauto.NewHistogram(
		prometheus.HistogramOpts{
			Name:    "get_payload_v1_latency_milliseconds",
			Help:    "Captures RPC latency for getPayloadV1 in milliseconds",
			Buckets: []float64{25, 50, 100, 200, 500, 1000, 2000, 4000},
		},
	)
	forkchoiceUpdatedLatency = promauto.NewHistogram(
		prometheus.HistogramOpts{
			Name:    "forkchoice_updated_v1_latency_milliseconds",
			Help:    "Captures RPC latency for forkchoiceUpdatedV1 in milliseconds",
			Buckets: []float64{25, 50, 100, 200, 500, 1000, 2000, 4000},
		},
	)
	errParseCount = promauto.NewCounter(prometheus.CounterOpts{
		Name: "execution_parse_error_count",
		Help: "The number of errors that occurred while parsing execution payload",
	})
	errInvalidRequestCount = promauto.NewCounter(prometheus.CounterOpts{
		Name: "execution_invalid_request_count",
		Help: "The number of errors that occurred due to invalid request",
	})
	errMethodNotFoundCount = promauto.NewCounter(prometheus.CounterOpts{
		Name: "execution_method_not_found_count",
		Help: "The number of errors that occurred due to method not found",
	})
	errInvalidParamsCount = promauto.NewCounter(prometheus.CounterOpts{
		Name: "execution_invalid_params_count",
		Help: "The number of errors that occurred due to invalid params",
	})
	errInternalCount = promauto.NewCounter(prometheus.CounterOpts{
		Name: "execution_internal_error_count",
		Help: "The number of errors that occurred due to internal error",
	})
	errUnknownPayloadCount = promauto.NewCounter(prometheus.CounterOpts{
		Name: "execution_unknown_payload_count",
		Help: "The number of errors that occurred due to unknown payload",
	})
	errInvalidForkchoiceStateCount = promauto.NewCounter(prometheus.CounterOpts{
		Name: "execution_invalid_forkchoice_state_count",
		Help: "The number of errors that occurred due to invalid forkchoice state",
	})
	errInvalidPayloadAttributesCount = promauto.NewCounter(prometheus.CounterOpts{
		Name: "execution_invalid_payload_attributes_count",
		Help: "The number of errors that occurred due to invalid payload attributes",
	})
	errServerErrorCount = promauto.NewCounter(prometheus.CounterOpts{
		Name: "execution_server_error_count",
		Help: "The number of errors that occurred due to server error",
	})
	reconstructedExecutionPayloadCount = promauto.NewCounter(prometheus.CounterOpts{
		Name: "reconstructed_execution_payload_count",
		Help: "Count the number of execution payloads that are reconstructed using JSON-RPC from payload headers",
	})
	errRequestTooLargeCount = promauto.NewCounter(prometheus.CounterOpts{
		Name: "execution_payload_bodies_count",
		Help: "The number of requested payload bodies is too large",
	})
)
