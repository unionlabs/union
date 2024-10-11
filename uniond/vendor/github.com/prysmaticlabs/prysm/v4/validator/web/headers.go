package web

import "net/http"

func addSecurityHeaders(w http.ResponseWriter) {
	// Deny displaying the web UI in any iframe.
	w.Header().Add("X-Frame-Options", "DENY")
	// Prevent xss in case a malicious HTML markup is served in any page.
	w.Header().Add("X-Content-Type-Options", "nosniff")
	// Prevent opening site in pop-up window to exploit cross-site leaks.
	w.Header().Add("Cross-Origin-Opener-Policy", "same-origin-allow-popups")
	// Prevent embedding from another resource.
	w.Header().Add("Cross-Origin-Resource-Policy", "same-origin")
}
