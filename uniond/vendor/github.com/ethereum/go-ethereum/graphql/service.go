// Copyright 2019 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package graphql

import (
	"context"
	"encoding/json"
	"net/http"
	"strconv"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/eth/filters"
	"github.com/ethereum/go-ethereum/internal/ethapi"
	"github.com/ethereum/go-ethereum/node"
	"github.com/ethereum/go-ethereum/rpc"
	"github.com/graph-gophers/graphql-go"
	gqlErrors "github.com/graph-gophers/graphql-go/errors"
)

type handler struct {
	Schema *graphql.Schema
}

func (h handler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	var params struct {
		Query         string                 `json:"query"`
		OperationName string                 `json:"operationName"`
		Variables     map[string]interface{} `json:"variables"`
	}
	if err := json.NewDecoder(r.Body).Decode(&params); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	var (
		ctx       = r.Context()
		responded sync.Once
		timer     *time.Timer
		cancel    context.CancelFunc
	)
	ctx, cancel = context.WithCancel(ctx)
	defer cancel()

	if timeout, ok := rpc.ContextRequestTimeout(ctx); ok {
		timer = time.AfterFunc(timeout, func() {
			responded.Do(func() {
				// Cancel request handling.
				cancel()

				// Create the timeout response.
				response := &graphql.Response{
					Errors: []*gqlErrors.QueryError{{Message: "request timed out"}},
				}
				responseJSON, err := json.Marshal(response)
				if err != nil {
					http.Error(w, err.Error(), http.StatusInternalServerError)
					return
				}

				// Setting this disables gzip compression in package node.
				w.Header().Set("Transfer-Encoding", "identity")

				// Flush the response. Since we are writing close to the response timeout,
				// chunked transfer encoding must be disabled by setting content-length.
				w.Header().Set("Content-Type", "application/json")
				w.Header().Set("Content-Length", strconv.Itoa(len(responseJSON)))
				w.Write(responseJSON)
				if flush, ok := w.(http.Flusher); ok {
					flush.Flush()
				}
			})
		})
	}

	response := h.Schema.Exec(ctx, params.Query, params.OperationName, params.Variables)
	if timer != nil {
		timer.Stop()
	}
	responded.Do(func() {
		responseJSON, err := json.Marshal(response)
		if err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		if len(response.Errors) > 0 {
			w.WriteHeader(http.StatusBadRequest)
		}
		w.Write(responseJSON)
	})
}

// New constructs a new GraphQL service instance.
func New(stack *node.Node, backend ethapi.Backend, filterSystem *filters.FilterSystem, cors, vhosts []string) error {
	_, err := newHandler(stack, backend, filterSystem, cors, vhosts)
	return err
}

// newHandler returns a new `http.Handler` that will answer GraphQL queries.
// It additionally exports an interactive query browser on the / endpoint.
func newHandler(stack *node.Node, backend ethapi.Backend, filterSystem *filters.FilterSystem, cors, vhosts []string) (*handler, error) {
	q := Resolver{backend, filterSystem}

	s, err := graphql.ParseSchema(schema, &q)
	if err != nil {
		return nil, err
	}
	h := handler{Schema: s}
	handler := node.NewHTTPHandlerStack(h, cors, vhosts, nil)

	stack.RegisterHandler("GraphQL UI", "/graphql/ui", GraphiQL{})
	stack.RegisterHandler("GraphQL UI", "/graphql/ui/", GraphiQL{})
	stack.RegisterHandler("GraphQL", "/graphql", handler)
	stack.RegisterHandler("GraphQL", "/graphql/", handler)

	return &h, nil
}
