package statsd

import (
	"fmt"
	"time"
)

// ServiceCheckStatus support
type ServiceCheckStatus byte

const (
	// Ok is the "ok" ServiceCheck status
	Ok ServiceCheckStatus = 0
	// Warn is the "warning" ServiceCheck status
	Warn ServiceCheckStatus = 1
	// Critical is the "critical" ServiceCheck status
	Critical ServiceCheckStatus = 2
	// Unknown is the "unknown" ServiceCheck status
	Unknown ServiceCheckStatus = 3
)

// A ServiceCheck is an object that contains status of DataDog service check.
type ServiceCheck struct {
	// Name of the service check.  Required.
	Name string
	// Status of service check.  Required.
	Status ServiceCheckStatus
	// Timestamp is a timestamp for the serviceCheck.  If not provided, the dogstatsd
	// server will set this to the current time.
	Timestamp time.Time
	// Hostname for the serviceCheck.
	Hostname string
	// A message describing the current state of the serviceCheck.
	Message string
	// Tags for the serviceCheck.
	Tags []string
}

// NewServiceCheck creates a new serviceCheck with the given name and status. Error checking
// against these values is done at send-time, or upon running sc.Check.
func NewServiceCheck(name string, status ServiceCheckStatus) *ServiceCheck {
	return &ServiceCheck{
		Name:   name,
		Status: status,
	}
}

// Check verifies that a service check is valid.
func (sc ServiceCheck) Check() error {
	if len(sc.Name) == 0 {
		return fmt.Errorf("statsd.ServiceCheck name is required")
	}
	if byte(sc.Status) < 0 || byte(sc.Status) > 3 {
		return fmt.Errorf("statsd.ServiceCheck status has invalid value")
	}
	return nil
}

// Encode returns the dogstatsd wire protocol representation for a service check.
// Tags may be passed which will be added to the encoded output but not to
// the Service Check's list of tags, eg. for default tags.
func (sc ServiceCheck) Encode(tags ...string) (string, error) {
	err := sc.Check()
	if err != nil {
		return "", err
	}
	var buffer []byte
	buffer = appendServiceCheck(buffer, sc, tags)
	return string(buffer), nil
}
