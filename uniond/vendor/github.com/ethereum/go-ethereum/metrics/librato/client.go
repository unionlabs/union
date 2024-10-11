package librato

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
)

const Operations = "operations"
const OperationsShort = "ops"

type LibratoClient struct {
	Email, Token string
}

// property strings
const (
	// display attributes
	Color             = "color"
	DisplayMax        = "display_max"
	DisplayMin        = "display_min"
	DisplayUnitsLong  = "display_units_long"
	DisplayUnitsShort = "display_units_short"
	DisplayStacked    = "display_stacked"
	DisplayTransform  = "display_transform"
	// special gauge display attributes
	SummarizeFunction = "summarize_function"
	Aggregate         = "aggregate"

	// metric keys
	Name        = "name"
	Period      = "period"
	Description = "description"
	DisplayName = "display_name"
	Attributes  = "attributes"

	// measurement keys
	MeasureTime = "measure_time"
	Source      = "source"
	Value       = "value"

	// special gauge keys
	Count      = "count"
	Sum        = "sum"
	Max        = "max"
	Min        = "min"
	SumSquares = "sum_squares"

	// batch keys
	Counters = "counters"
	Gauges   = "gauges"

	MetricsPostUrl = "https://metrics-api.librato.com/v1/metrics"
)

type Measurement map[string]interface{}
type Metric map[string]interface{}

type Batch struct {
	Gauges      []Measurement `json:"gauges,omitempty"`
	Counters    []Measurement `json:"counters,omitempty"`
	MeasureTime int64         `json:"measure_time"`
	Source      string        `json:"source"`
}

func (c *LibratoClient) PostMetrics(batch Batch) (err error) {
	var (
		js   []byte
		req  *http.Request
		resp *http.Response
	)

	if len(batch.Counters) == 0 && len(batch.Gauges) == 0 {
		return nil
	}

	if js, err = json.Marshal(batch); err != nil {
		return
	}

	if req, err = http.NewRequest(http.MethodPost, MetricsPostUrl, bytes.NewBuffer(js)); err != nil {
		return
	}

	req.Header.Set("Content-Type", "application/json")
	req.SetBasicAuth(c.Email, c.Token)

	resp, err = http.DefaultClient.Do(req)
	if err != nil {
		return
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		var body []byte
		if body, err = io.ReadAll(resp.Body); err != nil {
			body = []byte(fmt.Sprintf("(could not fetch response body for error: %s)", err))
		}
		err = fmt.Errorf("unable to post to Librato: %d %s %s", resp.StatusCode, resp.Status, string(body))
	}
	return
}
