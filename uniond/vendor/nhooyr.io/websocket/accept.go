// +build !js

package websocket

import (
	"bytes"
	"crypto/sha1"
	"encoding/base64"
	"errors"
	"fmt"
	"io"
	"log"
	"net/http"
	"net/textproto"
	"net/url"
	"path/filepath"
	"strings"

	"nhooyr.io/websocket/internal/errd"
)

// AcceptOptions represents Accept's options.
type AcceptOptions struct {
	// Subprotocols lists the WebSocket subprotocols that Accept will negotiate with the client.
	// The empty subprotocol will always be negotiated as per RFC 6455. If you would like to
	// reject it, close the connection when c.Subprotocol() == "".
	Subprotocols []string

	// InsecureSkipVerify is used to disable Accept's origin verification behaviour.
	//
	// You probably want to use OriginPatterns instead.
	InsecureSkipVerify bool

	// OriginPatterns lists the host patterns for authorized origins.
	// The request host is always authorized.
	// Use this to enable cross origin WebSockets.
	//
	// i.e javascript running on example.com wants to access a WebSocket server at chat.example.com.
	// In such a case, example.com is the origin and chat.example.com is the request host.
	// One would set this field to []string{"example.com"} to authorize example.com to connect.
	//
	// Each pattern is matched case insensitively against the request origin host
	// with filepath.Match.
	// See https://golang.org/pkg/path/filepath/#Match
	//
	// Please ensure you understand the ramifications of enabling this.
	// If used incorrectly your WebSocket server will be open to CSRF attacks.
	//
	// Do not use * as a pattern to allow any origin, prefer to use InsecureSkipVerify instead
	// to bring attention to the danger of such a setting.
	OriginPatterns []string

	// CompressionMode controls the compression mode.
	// Defaults to CompressionNoContextTakeover.
	//
	// See docs on CompressionMode for details.
	CompressionMode CompressionMode

	// CompressionThreshold controls the minimum size of a message before compression is applied.
	//
	// Defaults to 512 bytes for CompressionNoContextTakeover and 128 bytes
	// for CompressionContextTakeover.
	CompressionThreshold int
}

// Accept accepts a WebSocket handshake from a client and upgrades the
// the connection to a WebSocket.
//
// Accept will not allow cross origin requests by default.
// See the InsecureSkipVerify and OriginPatterns options to allow cross origin requests.
//
// Accept will write a response to w on all errors.
func Accept(w http.ResponseWriter, r *http.Request, opts *AcceptOptions) (*Conn, error) {
	return accept(w, r, opts)
}

func accept(w http.ResponseWriter, r *http.Request, opts *AcceptOptions) (_ *Conn, err error) {
	defer errd.Wrap(&err, "failed to accept WebSocket connection")

	if opts == nil {
		opts = &AcceptOptions{}
	}
	opts = &*opts

	errCode, err := verifyClientRequest(w, r)
	if err != nil {
		http.Error(w, err.Error(), errCode)
		return nil, err
	}

	if !opts.InsecureSkipVerify {
		err = authenticateOrigin(r, opts.OriginPatterns)
		if err != nil {
			if errors.Is(err, filepath.ErrBadPattern) {
				log.Printf("websocket: %v", err)
				err = errors.New(http.StatusText(http.StatusForbidden))
			}
			http.Error(w, err.Error(), http.StatusForbidden)
			return nil, err
		}
	}

	hj, ok := w.(http.Hijacker)
	if !ok {
		err = errors.New("http.ResponseWriter does not implement http.Hijacker")
		http.Error(w, http.StatusText(http.StatusNotImplemented), http.StatusNotImplemented)
		return nil, err
	}

	w.Header().Set("Upgrade", "websocket")
	w.Header().Set("Connection", "Upgrade")

	key := r.Header.Get("Sec-WebSocket-Key")
	w.Header().Set("Sec-WebSocket-Accept", secWebSocketAccept(key))

	subproto := selectSubprotocol(r, opts.Subprotocols)
	if subproto != "" {
		w.Header().Set("Sec-WebSocket-Protocol", subproto)
	}

	copts, err := acceptCompression(r, w, opts.CompressionMode)
	if err != nil {
		return nil, err
	}

	w.WriteHeader(http.StatusSwitchingProtocols)
	// See https://github.com/nhooyr/websocket/issues/166
	if ginWriter, ok := w.(interface {
		WriteHeaderNow()
	}); ok {
		ginWriter.WriteHeaderNow()
	}

	netConn, brw, err := hj.Hijack()
	if err != nil {
		err = fmt.Errorf("failed to hijack connection: %w", err)
		http.Error(w, http.StatusText(http.StatusInternalServerError), http.StatusInternalServerError)
		return nil, err
	}

	// https://github.com/golang/go/issues/32314
	b, _ := brw.Reader.Peek(brw.Reader.Buffered())
	brw.Reader.Reset(io.MultiReader(bytes.NewReader(b), netConn))

	return newConn(connConfig{
		subprotocol:    w.Header().Get("Sec-WebSocket-Protocol"),
		rwc:            netConn,
		client:         false,
		copts:          copts,
		flateThreshold: opts.CompressionThreshold,

		br: brw.Reader,
		bw: brw.Writer,
	}), nil
}

func verifyClientRequest(w http.ResponseWriter, r *http.Request) (errCode int, _ error) {
	if !r.ProtoAtLeast(1, 1) {
		return http.StatusUpgradeRequired, fmt.Errorf("WebSocket protocol violation: handshake request must be at least HTTP/1.1: %q", r.Proto)
	}

	if !headerContainsToken(r.Header, "Connection", "Upgrade") {
		w.Header().Set("Connection", "Upgrade")
		w.Header().Set("Upgrade", "websocket")
		return http.StatusUpgradeRequired, fmt.Errorf("WebSocket protocol violation: Connection header %q does not contain Upgrade", r.Header.Get("Connection"))
	}

	if !headerContainsToken(r.Header, "Upgrade", "websocket") {
		w.Header().Set("Connection", "Upgrade")
		w.Header().Set("Upgrade", "websocket")
		return http.StatusUpgradeRequired, fmt.Errorf("WebSocket protocol violation: Upgrade header %q does not contain websocket", r.Header.Get("Upgrade"))
	}

	if r.Method != "GET" {
		return http.StatusMethodNotAllowed, fmt.Errorf("WebSocket protocol violation: handshake request method is not GET but %q", r.Method)
	}

	if r.Header.Get("Sec-WebSocket-Version") != "13" {
		w.Header().Set("Sec-WebSocket-Version", "13")
		return http.StatusBadRequest, fmt.Errorf("unsupported WebSocket protocol version (only 13 is supported): %q", r.Header.Get("Sec-WebSocket-Version"))
	}

	if r.Header.Get("Sec-WebSocket-Key") == "" {
		return http.StatusBadRequest, errors.New("WebSocket protocol violation: missing Sec-WebSocket-Key")
	}

	return 0, nil
}

func authenticateOrigin(r *http.Request, originHosts []string) error {
	origin := r.Header.Get("Origin")
	if origin == "" {
		return nil
	}

	u, err := url.Parse(origin)
	if err != nil {
		return fmt.Errorf("failed to parse Origin header %q: %w", origin, err)
	}

	if strings.EqualFold(r.Host, u.Host) {
		return nil
	}

	for _, hostPattern := range originHosts {
		matched, err := match(hostPattern, u.Host)
		if err != nil {
			return fmt.Errorf("failed to parse filepath pattern %q: %w", hostPattern, err)
		}
		if matched {
			return nil
		}
	}
	return fmt.Errorf("request Origin %q is not authorized for Host %q", origin, r.Host)
}

func match(pattern, s string) (bool, error) {
	return filepath.Match(strings.ToLower(pattern), strings.ToLower(s))
}

func selectSubprotocol(r *http.Request, subprotocols []string) string {
	cps := headerTokens(r.Header, "Sec-WebSocket-Protocol")
	for _, sp := range subprotocols {
		for _, cp := range cps {
			if strings.EqualFold(sp, cp) {
				return cp
			}
		}
	}
	return ""
}

func acceptCompression(r *http.Request, w http.ResponseWriter, mode CompressionMode) (*compressionOptions, error) {
	if mode == CompressionDisabled {
		return nil, nil
	}

	for _, ext := range websocketExtensions(r.Header) {
		switch ext.name {
		case "permessage-deflate":
			return acceptDeflate(w, ext, mode)
			// Disabled for now, see https://github.com/nhooyr/websocket/issues/218
			// case "x-webkit-deflate-frame":
			// 	return acceptWebkitDeflate(w, ext, mode)
		}
	}
	return nil, nil
}

func acceptDeflate(w http.ResponseWriter, ext websocketExtension, mode CompressionMode) (*compressionOptions, error) {
	copts := mode.opts()

	for _, p := range ext.params {
		switch p {
		case "client_no_context_takeover":
			copts.clientNoContextTakeover = true
			continue
		case "server_no_context_takeover":
			copts.serverNoContextTakeover = true
			continue
		}

		if strings.HasPrefix(p, "client_max_window_bits") {
			// We cannot adjust the read sliding window so cannot make use of this.
			continue
		}

		err := fmt.Errorf("unsupported permessage-deflate parameter: %q", p)
		http.Error(w, err.Error(), http.StatusBadRequest)
		return nil, err
	}

	copts.setHeader(w.Header())

	return copts, nil
}

func acceptWebkitDeflate(w http.ResponseWriter, ext websocketExtension, mode CompressionMode) (*compressionOptions, error) {
	copts := mode.opts()
	// The peer must explicitly request it.
	copts.serverNoContextTakeover = false

	for _, p := range ext.params {
		if p == "no_context_takeover" {
			copts.serverNoContextTakeover = true
			continue
		}

		// We explicitly fail on x-webkit-deflate-frame's max_window_bits parameter instead
		// of ignoring it as the draft spec is unclear. It says the server can ignore it
		// but the server has no way of signalling to the client it was ignored as the parameters
		// are set one way.
		// Thus us ignoring it would make the client think we understood it which would cause issues.
		// See https://tools.ietf.org/html/draft-tyoshino-hybi-websocket-perframe-deflate-06#section-4.1
		//
		// Either way, we're only implementing this for webkit which never sends the max_window_bits
		// parameter so we don't need to worry about it.
		err := fmt.Errorf("unsupported x-webkit-deflate-frame parameter: %q", p)
		http.Error(w, err.Error(), http.StatusBadRequest)
		return nil, err
	}

	s := "x-webkit-deflate-frame"
	if copts.clientNoContextTakeover {
		s += "; no_context_takeover"
	}
	w.Header().Set("Sec-WebSocket-Extensions", s)

	return copts, nil
}

func headerContainsToken(h http.Header, key, token string) bool {
	token = strings.ToLower(token)

	for _, t := range headerTokens(h, key) {
		if t == token {
			return true
		}
	}
	return false
}

type websocketExtension struct {
	name   string
	params []string
}

func websocketExtensions(h http.Header) []websocketExtension {
	var exts []websocketExtension
	extStrs := headerTokens(h, "Sec-WebSocket-Extensions")
	for _, extStr := range extStrs {
		if extStr == "" {
			continue
		}

		vals := strings.Split(extStr, ";")
		for i := range vals {
			vals[i] = strings.TrimSpace(vals[i])
		}

		e := websocketExtension{
			name:   vals[0],
			params: vals[1:],
		}

		exts = append(exts, e)
	}
	return exts
}

func headerTokens(h http.Header, key string) []string {
	key = textproto.CanonicalMIMEHeaderKey(key)
	var tokens []string
	for _, v := range h[key] {
		v = strings.TrimSpace(v)
		for _, t := range strings.Split(v, ",") {
			t = strings.ToLower(t)
			t = strings.TrimSpace(t)
			tokens = append(tokens, t)
		}
	}
	return tokens
}

var keyGUID = []byte("258EAFA5-E914-47DA-95CA-C5AB0DC85B11")

func secWebSocketAccept(secWebSocketKey string) string {
	h := sha1.New()
	h.Write([]byte(secWebSocketKey))
	h.Write(keyGUID)

	return base64.StdEncoding.EncodeToString(h.Sum(nil))
}
