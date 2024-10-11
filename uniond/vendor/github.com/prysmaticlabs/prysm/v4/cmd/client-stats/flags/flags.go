// Package flags contains all configuration runtime flags for
// the client-stats daemon.
package flags

import (
	"time"

	"github.com/urfave/cli/v2"
)

var (
	// ValidatorMetricsURLFlag defines a flag for the URL to the validator /metrics prometheus endpoint to scrape.
	ValidatorMetricsURLFlag = &cli.StringFlag{
		Name:  "validator-metrics-url",
		Usage: "Full URL to the validator /metrics prometheus endpoint to scrape. eg http://localhost:8081/metrics",
	}
	// BeaconnodeMetricsURLFlag defines a flag for the URL to the beacon-node /metrics prometheus endpoint to scraps.
	BeaconnodeMetricsURLFlag = &cli.StringFlag{
		Name:  "beacon-node-metrics-url",
		Usage: "Full URL to the beacon-node /metrics prometheus endpoint to scrape. eg http://localhost:8080/metrics",
	}
	// ClientStatsAPIURLFlag defines a flag for the URL to the client stats endpoint where collected metrics should be sent.
	ClientStatsAPIURLFlag = &cli.StringFlag{
		Name:  "clientstats-api-url",
		Usage: "Full URL to the client stats endpoint where collected metrics should be sent.",
	}
	// ScrapeIntervalFlag defines a flag for the frequency of scraping.
	ScrapeIntervalFlag = &cli.DurationFlag{
		Name:  "scrape-interval",
		Usage: "Frequency of scraping expressed as a duration, eg 2m or 1m5s.",
		Value: 60 * time.Second,
	}
)
