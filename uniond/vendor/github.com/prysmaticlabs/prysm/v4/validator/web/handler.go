package web

import (
	"mime"
	"net/http"
	"net/url"
	"path"
)

const prefix = "prysm-web-ui"

// Handler serves web requests from the bundled site data.
// DEPRECATED: Prysm Web UI and associated endpoints will be fully removed in a future hard fork.
var Handler = func(res http.ResponseWriter, req *http.Request) {
	addSecurityHeaders(res)
	u, err := url.ParseRequestURI(req.RequestURI)
	if err != nil {
		log.WithError(err).Error("Cannot parse request URI")
		return
	}
	p := u.Path
	if p == "/" {
		p = "/index.html"
	}
	p = path.Join(prefix, p)

	if d, ok := _bindata[p]; ok {
		m := mime.TypeByExtension(path.Ext(p))
		res.Header().Add("Content-Type", m)
		res.WriteHeader(http.StatusOK)
		asset, err := d()
		if err != nil {
			log.WithError(err).Error("Failed to unwrap asset data for http response")
		}
		if _, err := res.Write(asset.bytes); err != nil {
			log.WithError(err).Error("Failed to write http response")
		}
	} else if d, ok := _bindata[path.Join(prefix, "index.html")]; ok {
		// Angular routing expects that routes are rewritten to serve index.html. For example, if
		// requesting /login, this should serve the single page app index.html.
		m := mime.TypeByExtension(".html")
		res.Header().Add("Content-Type", m)
		res.WriteHeader(http.StatusOK)
		asset, err := d()
		if err != nil {
			log.WithError(err).Error("Failed to unwrap asset data for http response")
		}
		if _, err := res.Write(asset.bytes); err != nil {
			log.WithError(err).Error("Failed to write http response")
		}
	} else { // If index.html is not present, serve 404. This should never happen.
		log.WithField("URI", req.RequestURI).Error("Path not found")
		res.WriteHeader(http.StatusNotFound)
		if _, err := res.Write([]byte("Not found")); err != nil {
			log.WithError(err).Error("Failed to write http response")
		}
	}
}
