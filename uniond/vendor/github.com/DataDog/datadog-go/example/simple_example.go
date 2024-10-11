package main

import (
	"log"

	"github.com/DataDog/datadog-go/statsd"
)

func main() {
	client, err := statsd.New("127.0.0.1:8125",
		statsd.WithTags([]string{"env:prod", "service:myservice"}),
	)
	if err != nil {
		log.Fatal(err)
	}

	client.Gauge("my.metrics", 21, []string{"tag1", "tag2:value"}, 1)
	client.Close()
}
