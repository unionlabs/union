package clientstats

import "io"

// A Scraper polls the data source it has been configured with
// and interprets the content to produce a client-stats process
// metric. Scrapers currently exist to produce 'validator' and
// 'beaconnode' metric types.
type Scraper interface {
	Scrape() (io.Reader, error)
}

// An Updater can take the io.Reader created by Scraper and
// send it to a data sink for consumption. An Updater is used
// for instance to send the scraped data for a beacon-node to
// a remote client-stats endpoint.
type Updater interface {
	Update(io.Reader) error
}
