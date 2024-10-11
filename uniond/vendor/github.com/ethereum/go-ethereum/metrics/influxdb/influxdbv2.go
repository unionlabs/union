package influxdb

import (
	"context"
	"time"

	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/metrics"
	influxdb2 "github.com/influxdata/influxdb-client-go/v2"
	"github.com/influxdata/influxdb-client-go/v2/api"
)

type v2Reporter struct {
	reg      metrics.Registry
	interval time.Duration

	endpoint     string
	token        string
	bucket       string
	organization string
	namespace    string
	tags         map[string]string

	client influxdb2.Client
	write  api.WriteAPI
}

// InfluxDBWithTags starts a InfluxDB reporter which will post the from the given metrics.Registry at each d interval with the specified tags
func InfluxDBV2WithTags(r metrics.Registry, d time.Duration, endpoint string, token string, bucket string, organization string, namespace string, tags map[string]string) {
	rep := &v2Reporter{
		reg:          r,
		interval:     d,
		endpoint:     endpoint,
		token:        token,
		bucket:       bucket,
		organization: organization,
		namespace:    namespace,
		tags:         tags,
	}

	rep.client = influxdb2.NewClient(rep.endpoint, rep.token)
	defer rep.client.Close()

	// async write client
	rep.write = rep.client.WriteAPI(rep.organization, rep.bucket)
	errorsCh := rep.write.Errors()

	// have to handle write errors in a separate goroutine like this b/c the channel is unbuffered and will block writes if not read
	go func() {
		for err := range errorsCh {
			log.Warn("write error", "err", err.Error())
		}
	}()
	rep.run()
}

func (r *v2Reporter) run() {
	intervalTicker := time.NewTicker(r.interval)
	pingTicker := time.NewTicker(time.Second * 5)

	defer intervalTicker.Stop()
	defer pingTicker.Stop()

	for {
		select {
		case <-intervalTicker.C:
			r.send(0)
		case <-pingTicker.C:
			_, err := r.client.Health(context.Background())
			if err != nil {
				log.Warn("Got error from influxdb client health check", "err", err.Error())
			}
		}
	}
}

// send sends the measurements. If provided tstamp is >0, it is used. Otherwise,
// a 'fresh' timestamp is used.
func (r *v2Reporter) send(tstamp int64) {
	r.reg.Each(func(name string, i interface{}) {
		var now time.Time
		if tstamp <= 0 {
			now = time.Now()
		} else {
			now = time.Unix(tstamp, 0)
		}
		measurement, fields := readMeter(r.namespace, name, i)
		if fields == nil {
			return
		}
		pt := influxdb2.NewPoint(measurement, r.tags, fields, now)
		r.write.WritePoint(pt)
	})
	// Force all unwritten data to be sent
	r.write.Flush()
}
