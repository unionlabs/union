// The MIT License (MIT)
//
// Copyright (c) 2016 Muhammed Thanish
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

package graphql

import (
	"encoding/json"
	"net/http"
	"path/filepath"

	"github.com/ethereum/go-ethereum/graphql/internal/graphiql"
	"github.com/ethereum/go-ethereum/log"
)

// GraphiQL is an in-browser IDE for exploring GraphiQL APIs.
// This handler returns GraphiQL when requested.
//
// For more information, see https://github.com/graphql/graphiql.
type GraphiQL struct{}

func respOk(w http.ResponseWriter, body []byte, ctype string) {
	w.Header().Set("Content-Type", ctype)
	w.Header().Set("X-Content-Type-Options", "nosniff")
	w.Write(body)
}

func respErr(w http.ResponseWriter, msg string, code int) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(code)
	errMsg, _ := json.Marshal(struct {
		Error string
	}{Error: msg})
	w.Write(errMsg)
}

func (h GraphiQL) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		respErr(w, "only GET allowed", http.StatusMethodNotAllowed)
		return
	}
	switch r.URL.Path {
	case "/graphql/ui/graphiql.min.css":
		data, err := graphiql.Assets.ReadFile(filepath.Base(r.URL.Path))
		if err != nil {
			log.Warn("Error loading graphiql asset", "err", err)
			respErr(w, "internal error", http.StatusInternalServerError)
			return
		}
		respOk(w, data, "text/css")
	case "/graphql/ui/graphiql.min.js",
		"/graphql/ui/react.production.min.js",
		"/graphql/ui/react-dom.production.min.js":
		data, err := graphiql.Assets.ReadFile(filepath.Base(r.URL.Path))
		if err != nil {
			log.Warn("Error loading graphiql asset", "err", err)
			respErr(w, "internal error", http.StatusInternalServerError)
			return
		}
		respOk(w, data, "application/javascript; charset=utf-8")
	default:
		data, err := graphiql.Assets.ReadFile("index.html")
		if err != nil {
			log.Warn("Error loading graphiql asset", "err", err)
			respErr(w, "internal error", http.StatusInternalServerError)
			return
		}
		respOk(w, data, "text/html")
	}
}
