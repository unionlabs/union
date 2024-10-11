#!/bin/env bash

set -e

go get github.com/hashicorp/go-version@v1.6.0
go run prometheus/gen_go_collector_metrics_set.go
mv -f go_collector_metrics_* prometheus
go run prometheus/collectors/gen_go_collector_set.go
mv -f go_collector_* prometheus/collectors
