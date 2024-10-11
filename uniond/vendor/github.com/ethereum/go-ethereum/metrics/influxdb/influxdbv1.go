package influxdb

import (
	"fmt"
	uurl "net/url"
	"time"

	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/metrics"
	client "github.com/influxdata/influxdb1-client/v2"
)

type reporter struct {
	reg      metrics.Registry
	interval time.Duration

	url       uurl.URL
	database  string
	username  string
	password  string
	namespace string
	tags      map[string]string

	client client.Client

	cache map[string]int64
}

// InfluxDB starts a InfluxDB reporter which will post the from the given metrics.Registry at each d interval.
func InfluxDB(r metrics.Registry, d time.Duration, url, database, username, password, namespace string) {
	InfluxDBWithTags(r, d, url, database, username, password, namespace, nil)
}

// InfluxDBWithTags starts a InfluxDB reporter which will post the from the given metrics.Registry at each d interval with the specified tags
func InfluxDBWithTags(r metrics.Registry, d time.Duration, url, database, username, password, namespace string, tags map[string]string) {
	u, err := uurl.Parse(url)
	if err != nil {
		log.Warn("Unable to parse InfluxDB", "url", url, "err", err)
		return
	}

	rep := &reporter{
		reg:       r,
		interval:  d,
		url:       *u,
		database:  database,
		username:  username,
		password:  password,
		namespace: namespace,
		tags:      tags,
		cache:     make(map[string]int64),
	}
	if err := rep.makeClient(); err != nil {
		log.Warn("Unable to make InfluxDB client", "err", err)
		return
	}

	rep.run()
}

// InfluxDBWithTagsOnce runs once an InfluxDB reporter and post the given metrics.Registry with the specified tags
func InfluxDBWithTagsOnce(r metrics.Registry, url, database, username, password, namespace string, tags map[string]string) error {
	u, err := uurl.Parse(url)
	if err != nil {
		return fmt.Errorf("unable to parse InfluxDB. url: %s, err: %v", url, err)
	}

	rep := &reporter{
		reg:       r,
		url:       *u,
		database:  database,
		username:  username,
		password:  password,
		namespace: namespace,
		tags:      tags,
		cache:     make(map[string]int64),
	}
	if err := rep.makeClient(); err != nil {
		return fmt.Errorf("unable to make InfluxDB client. err: %v", err)
	}

	if err := rep.send(0); err != nil {
		return fmt.Errorf("unable to send to InfluxDB. err: %v", err)
	}

	return nil
}

func (r *reporter) makeClient() (err error) {
	r.client, err = client.NewHTTPClient(client.HTTPConfig{
		Addr:     r.url.String(),
		Username: r.username,
		Password: r.password,
		Timeout:  10 * time.Second,
	})

	return
}

func (r *reporter) run() {
	intervalTicker := time.NewTicker(r.interval)
	pingTicker := time.NewTicker(time.Second * 5)

	defer intervalTicker.Stop()
	defer pingTicker.Stop()

	for {
		select {
		case <-intervalTicker.C:
			if err := r.send(0); err != nil {
				log.Warn("Unable to send to InfluxDB", "err", err)
			}
		case <-pingTicker.C:
			_, _, err := r.client.Ping(0)
			if err != nil {
				log.Warn("Got error while sending a ping to InfluxDB, trying to recreate client", "err", err)

				if err = r.makeClient(); err != nil {
					log.Warn("Unable to make InfluxDB client", "err", err)
				}
			}
		}
	}
}

// send sends the measurements. If provided tstamp is >0, it is used. Otherwise,
// a 'fresh' timestamp is used.
func (r *reporter) send(tstamp int64) error {
	bps, err := client.NewBatchPoints(
		client.BatchPointsConfig{
			Database: r.database,
		})
	if err != nil {
		return err
	}
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
		if p, err := client.NewPoint(measurement, r.tags, fields, now); err == nil {
			bps.AddPoint(p)
		}
	})
	return r.client.Write(bps)
}
