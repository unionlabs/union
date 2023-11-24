package handlers

import (
	"log"
	"net/http"
	"runtime/debug"
)

// RecoveryHandlerLogger is an interface used by the recovering handler to print logs.
type RecoveryHandlerLogger interface {
	Println(...interface{})
}

type recoveryHandler struct {
	handler    http.Handler
	logger     RecoveryHandlerLogger
	printStack bool
}

// RecoveryOption provides a functional approach to define
// configuration for a handler; such as setting the logging
// whether or not to print stack traces on panic.
type RecoveryOption func(http.Handler)

func parseRecoveryOptions(h http.Handler, opts ...RecoveryOption) http.Handler {
	for _, option := range opts {
		option(h)
	}

	return h
}

// RecoveryHandler is HTTP middleware that recovers from a panic,
// logs the panic, writes http.StatusInternalServerError, and
// continues to the next handler.
//
// Example:
//
//	r := mux.NewRouter()
//	r.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
//		panic("Unexpected error!")
//	})
//
//	http.ListenAndServe(":1123", handlers.RecoveryHandler()(r))
func RecoveryHandler(opts ...RecoveryOption) func(h http.Handler) http.Handler {
	return func(h http.Handler) http.Handler {
		r := &recoveryHandler{handler: h}
		return parseRecoveryOptions(r, opts...)
	}
}

// RecoveryLogger is a functional option to override
// the default logger.
func RecoveryLogger(logger RecoveryHandlerLogger) RecoveryOption {
	return func(h http.Handler) {
		r := h.(*recoveryHandler) //nolint:errcheck //TODO:
		// @bharat-rajani should return type-assertion error but would break the API?
		r.logger = logger
	}
}

// PrintRecoveryStack is a functional option to enable
// or disable printing stack traces on panic.
func PrintRecoveryStack(shouldPrint bool) RecoveryOption {
	return func(h http.Handler) {
		r := h.(*recoveryHandler) //nolint:errcheck //TODO:
		// @bharat-rajani should return type-assertion error but would break the API?
		r.printStack = shouldPrint
	}
}

func (h recoveryHandler) ServeHTTP(w http.ResponseWriter, req *http.Request) {
	defer func() {
		if err := recover(); err != nil {
			w.WriteHeader(http.StatusInternalServerError)
			h.log(err)
		}
	}()

	h.handler.ServeHTTP(w, req)
}

func (h recoveryHandler) log(v ...interface{}) {
	if h.logger != nil {
		h.logger.Println(v...)
	} else {
		log.Println(v...)
	}

	if h.printStack {
		stack := string(debug.Stack())
		if h.logger != nil {
			h.logger.Println(stack)
		} else {
			log.Println(stack)
		}
	}
}
