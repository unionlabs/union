package server

import (
	"net/http"

	"github.com/gorilla/mux"
	"github.com/rs/cors"
)

// NormalizeQueryValuesHandler normalizes an input query of "key=value1,value2,value3" to "key=value1&key=value2&key=value3"
func NormalizeQueryValuesHandler(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		query := r.URL.Query()
		NormalizeQueryValues(query)
		r.URL.RawQuery = query.Encode()

		next.ServeHTTP(w, r)
	})
}

// CorsHandler sets the cors settings on api endpoints
func CorsHandler(allowOrigins []string) mux.MiddlewareFunc {
	c := cors.New(cors.Options{
		AllowedOrigins:   allowOrigins,
		AllowedMethods:   []string{http.MethodPost, http.MethodGet, http.MethodDelete, http.MethodOptions},
		AllowCredentials: true,
		MaxAge:           600,
		AllowedHeaders:   []string{"*"},
	})

	return c.Handler
}
