module github.com/hashicorp/go-metrics

go 1.12

require (
	github.com/DataDog/datadog-go v3.2.0+incompatible
	github.com/circonus-labs/circonus-gometrics v2.3.1+incompatible
	github.com/circonus-labs/circonusllhist v0.1.3 // indirect
	github.com/golang/protobuf v1.3.2
	github.com/hashicorp/go-immutable-radix v1.0.0
	github.com/hashicorp/go-retryablehttp v0.5.3 // indirect
	github.com/pascaldekloe/goe v0.1.0
	github.com/prometheus/client_golang v1.4.0
	github.com/prometheus/client_model v0.2.0
	github.com/prometheus/common v0.9.1
	github.com/tv42/httpunix v0.0.0-20150427012821-b75d8614f926 // indirect
)

// Introduced undocumented breaking change to metrics sink interface
retract v0.3.11