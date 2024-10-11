package eureka

import (
	"fmt"
	"net/http"
	"sync"
	"time"

	"github.com/hudl/fargo"

	"github.com/go-kit/kit/sd"
	"github.com/go-kit/log"
)

// Matches official Netflix Java client default.
const defaultRenewalInterval = 30 * time.Second

// The methods of fargo.Connection used in this package.
type fargoConnection interface {
	RegisterInstance(instance *fargo.Instance) error
	DeregisterInstance(instance *fargo.Instance) error
	ReregisterInstance(instance *fargo.Instance) error
	HeartBeatInstance(instance *fargo.Instance) error
	ScheduleAppUpdates(name string, await bool, done <-chan struct{}) <-chan fargo.AppUpdate
	GetApp(name string) (*fargo.Application, error)
}

type fargoUnsuccessfulHTTPResponse struct {
	statusCode    int
	messagePrefix string
}

func (u *fargoUnsuccessfulHTTPResponse) Error() string {
	return fmt.Sprintf("err=%s code=%d", u.messagePrefix, u.statusCode)
}

// Registrar maintains service instance liveness information in Eureka.
type Registrar struct {
	conn     fargoConnection
	instance *fargo.Instance
	logger   log.Logger
	quitc    chan chan struct{}
	sync.Mutex
}

var _ sd.Registrar = (*Registrar)(nil)

// NewRegistrar returns an Eureka Registrar acting on behalf of the provided
// Fargo connection and instance. See the integration test for usage examples.
func NewRegistrar(conn fargoConnection, instance *fargo.Instance, logger log.Logger) *Registrar {
	return &Registrar{
		conn:     conn,
		instance: instance,
		logger:   log.With(logger, "service", instance.App, "address", fmt.Sprintf("%s:%d", instance.IPAddr, instance.Port)),
	}
}

// Register implements sd.Registrar.
func (r *Registrar) Register() {
	r.Lock()
	defer r.Unlock()

	if r.quitc != nil {
		return // Already in the registration loop.
	}

	if err := r.conn.RegisterInstance(r.instance); err != nil {
		r.logger.Log("during", "Register", "err", err)
	}

	r.quitc = make(chan chan struct{})
	go r.loop()
}

// Deregister implements sd.Registrar.
func (r *Registrar) Deregister() {
	r.Lock()
	defer r.Unlock()

	if r.quitc == nil {
		return // Already deregistered.
	}

	q := make(chan struct{})
	r.quitc <- q
	<-q
	r.quitc = nil
}

func (r *Registrar) loop() {
	var renewalInterval time.Duration
	if r.instance.LeaseInfo.RenewalIntervalInSecs > 0 {
		renewalInterval = time.Duration(r.instance.LeaseInfo.RenewalIntervalInSecs) * time.Second
	} else {
		renewalInterval = defaultRenewalInterval
	}
	ticker := time.NewTicker(renewalInterval)
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			if err := r.heartbeat(); err != nil {
				r.logger.Log("during", "heartbeat", "err", err)
			}

		case q := <-r.quitc:
			if err := r.conn.DeregisterInstance(r.instance); err != nil {
				r.logger.Log("during", "Deregister", "err", err)
			}
			close(q)
			return
		}
	}
}

func httpResponseStatusCode(err error) (code int, present bool) {
	if code, ok := fargo.HTTPResponseStatusCode(err); ok {
		return code, true
	}
	// Allow injection of errors for testing.
	if u, ok := err.(*fargoUnsuccessfulHTTPResponse); ok {
		return u.statusCode, true
	}
	return 0, false
}

func isNotFound(err error) bool {
	code, ok := httpResponseStatusCode(err)
	return ok && code == http.StatusNotFound
}

func (r *Registrar) heartbeat() error {
	err := r.conn.HeartBeatInstance(r.instance)
	if err == nil {
		return nil
	}
	if isNotFound(err) {
		// Instance expired (e.g. network partition). Re-register.
		return r.conn.ReregisterInstance(r.instance)
	}
	return err
}
