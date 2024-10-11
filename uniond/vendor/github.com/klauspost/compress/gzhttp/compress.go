package gzhttp

import (
	"bufio"
	"crypto/rand"
	"crypto/sha256"
	"encoding/binary"
	"errors"
	"fmt"
	"hash/crc32"
	"io"
	"math"
	"math/bits"
	"mime"
	"net"
	"net/http"
	"strconv"
	"strings"
	"sync"

	"github.com/klauspost/compress/gzhttp/writer"
	"github.com/klauspost/compress/gzhttp/writer/gzkp"
	"github.com/klauspost/compress/gzip"
)

const (
	// HeaderNoCompression can be used to disable compression.
	// Any header value will disable compression.
	// The Header is always removed from output.
	HeaderNoCompression = "No-Gzip-Compression"

	vary            = "Vary"
	acceptEncoding  = "Accept-Encoding"
	contentEncoding = "Content-Encoding"
	contentRange    = "Content-Range"
	acceptRanges    = "Accept-Ranges"
	contentType     = "Content-Type"
	contentLength   = "Content-Length"
	eTag            = "ETag"
)

type codings map[string]float64

const (
	// DefaultQValue is the default qvalue to assign to an encoding if no explicit qvalue is set.
	// This is actually kind of ambiguous in RFC 2616, so hopefully it's correct.
	// The examples seem to indicate that it is.
	DefaultQValue = 1.0

	// DefaultMinSize is the default minimum size until we enable gzip compression.
	// 1500 bytes is the MTU size for the internet since that is the largest size allowed at the network layer.
	// If you take a file that is 1300 bytes and compress it to 800 bytes, it’s still transmitted in that same 1500 byte packet regardless, so you’ve gained nothing.
	// That being the case, you should restrict the gzip compression to files with a size (plus header) greater than a single packet,
	// 1024 bytes (1KB) is therefore default.
	DefaultMinSize = 1024
)

// GzipResponseWriter provides an http.ResponseWriter interface, which gzips
// bytes before writing them to the underlying response. This doesn't close the
// writers, so don't forget to do that.
// It can be configured to skip response smaller than minSize.
type GzipResponseWriter struct {
	http.ResponseWriter
	level     int
	gwFactory writer.GzipWriterFactory
	gw        writer.GzipWriter

	code int // Saves the WriteHeader value.

	minSize          int    // Specifies the minimum response size to gzip. If the response length is bigger than this value, it is compressed.
	buf              []byte // Holds the first part of the write before reaching the minSize or the end of the write.
	ignore           bool   // If true, then we immediately passthru writes to the underlying ResponseWriter.
	keepAcceptRanges bool   // Keep "Accept-Ranges" header.
	setContentType   bool   // Add content type, if missing and detected.
	suffixETag       string // Suffix to add to ETag header if response is compressed.
	dropETag         bool   // Drop ETag header if response is compressed (supersedes suffixETag).
	sha256Jitter     bool   // Use sha256 for jitter.
	randomJitter     string // Add random bytes to output as header field.
	jitterBuffer     int    // Maximum buffer to accumulate before doing jitter.

	contentTypeFilter func(ct string) bool // Only compress if the response is one of these content-types. All are accepted if empty.
}

type GzipResponseWriterWithCloseNotify struct {
	*GzipResponseWriter
}

func (w GzipResponseWriterWithCloseNotify) CloseNotify() <-chan bool {
	return w.ResponseWriter.(http.CloseNotifier).CloseNotify()
}

// Write appends data to the gzip writer.
func (w *GzipResponseWriter) Write(b []byte) (int, error) {
	// GZIP responseWriter is initialized. Use the GZIP responseWriter.
	if w.gw != nil {
		return w.gw.Write(b)
	}

	// If we have already decided not to use GZIP, immediately passthrough.
	if w.ignore {
		return w.ResponseWriter.Write(b)
	}

	// Save the write into a buffer for later use in GZIP responseWriter
	// (if content is long enough) or at close with regular responseWriter.
	wantBuf := 512
	if w.minSize > wantBuf {
		wantBuf = w.minSize
	}
	if w.jitterBuffer > 0 && w.jitterBuffer > wantBuf {
		wantBuf = w.jitterBuffer
	}
	toAdd := len(b)
	if len(w.buf)+toAdd > wantBuf {
		toAdd = wantBuf - len(w.buf)
	}
	w.buf = append(w.buf, b[:toAdd]...)
	remain := b[toAdd:]
	hdr := w.Header()

	// Only continue if they didn't already choose an encoding or a known unhandled content length or type.
	if len(hdr[HeaderNoCompression]) == 0 && hdr.Get(contentEncoding) == "" && hdr.Get(contentRange) == "" {
		// Check more expensive parts now.
		cl, _ := atoi(hdr.Get(contentLength))
		ct := hdr.Get(contentType)
		if cl == 0 || cl >= w.minSize && (ct == "" || w.contentTypeFilter(ct)) {
			// If the current buffer is less than minSize and a Content-Length isn't set, then wait until we have more data.
			if len(w.buf) < w.minSize && cl == 0 || (w.jitterBuffer > 0 && len(w.buf) < w.jitterBuffer) {
				return len(b), nil
			}

			// If the Content-Length is larger than minSize or the current buffer is larger than minSize, then continue.
			if cl >= w.minSize || len(w.buf) >= w.minSize {
				// If a Content-Type wasn't specified, infer it from the current buffer.
				if ct == "" {
					ct = http.DetectContentType(w.buf)
				}

				// Handles the intended case of setting a nil Content-Type (as for http/server or http/fs)
				// Set the header only if the key does not exist
				if _, ok := hdr[contentType]; w.setContentType && !ok {
					hdr.Set(contentType, ct)
				}

				// If the Content-Type is acceptable to GZIP, initialize the GZIP writer.
				if w.contentTypeFilter(ct) {
					if err := w.startGzip(remain); err != nil {
						return 0, err
					}
					if len(remain) > 0 {
						if _, err := w.gw.Write(remain); err != nil {
							return 0, err
						}
					}
					return len(b), nil
				}
			}
		}
	}
	// If we got here, we should not GZIP this response.
	if err := w.startPlain(); err != nil {
		return 0, err
	}
	if len(remain) > 0 {
		if _, err := w.ResponseWriter.Write(remain); err != nil {
			return 0, err
		}
	}
	return len(b), nil
}

func (w *GzipResponseWriter) Unwrap() http.ResponseWriter {
	return w.ResponseWriter
}

var castagnoliTable = crc32.MakeTable(crc32.Castagnoli)

// startGzip initializes a GZIP writer and writes the buffer.
func (w *GzipResponseWriter) startGzip(remain []byte) error {
	// Set the GZIP header.
	w.Header().Set(contentEncoding, "gzip")

	// if the Content-Length is already set, then calls to Write on gzip
	// will fail to set the Content-Length header since its already set
	// See: https://github.com/golang/go/issues/14975.
	w.Header().Del(contentLength)

	// Delete Accept-Ranges.
	if !w.keepAcceptRanges {
		w.Header().Del(acceptRanges)
	}

	// Suffix ETag.
	if w.suffixETag != "" && !w.dropETag && w.Header().Get(eTag) != "" {
		orig := w.Header().Get(eTag)
		insertPoint := strings.LastIndex(orig, `"`)
		if insertPoint == -1 {
			insertPoint = len(orig)
		}
		w.Header().Set(eTag, orig[:insertPoint]+w.suffixETag+orig[insertPoint:])
	}

	// Delete ETag.
	if w.dropETag {
		w.Header().Del(eTag)
	}

	// Write the header to gzip response.
	if w.code != 0 {
		w.ResponseWriter.WriteHeader(w.code)
		// Ensure that no other WriteHeader's happen
		w.code = 0
	}

	// Initialize and flush the buffer into the gzip response if there are any bytes.
	// If there aren't any, we shouldn't initialize it yet because on Close it will
	// write the gzip header even if nothing was ever written.
	if len(w.buf) > 0 {
		// Initialize the GZIP response.
		w.init()

		// Set random jitter based on CRC or SHA-256 of current buffer.
		// Before first write.
		if len(w.randomJitter) > 0 {
			var jitRNG uint32
			if w.jitterBuffer > 0 {
				if w.sha256Jitter {
					h := sha256.New()
					h.Write(w.buf)
					// Use only up to "w.jitterBuffer", otherwise the output depends on write sizes.
					if len(remain) > 0 && len(w.buf) < w.jitterBuffer {
						remain := remain
						if len(remain)+len(w.buf) > w.jitterBuffer {
							remain = remain[:w.jitterBuffer-len(w.buf)]
						}
						h.Write(remain)
					}
					var tmp [sha256.Size]byte
					jitRNG = binary.LittleEndian.Uint32(h.Sum(tmp[:0]))
				} else {
					h := crc32.Update(0, castagnoliTable, w.buf)
					// Use only up to "w.jitterBuffer", otherwise the output depends on write sizes.
					if len(remain) > 0 && len(w.buf) < w.jitterBuffer {
						remain := remain
						if len(remain)+len(w.buf) > w.jitterBuffer {
							remain = remain[:w.jitterBuffer-len(w.buf)]
						}
						h = crc32.Update(h, castagnoliTable, remain)
					}
					jitRNG = bits.RotateLeft32(h, 19) ^ 0xab0755de
				}
			} else {
				// Get from rand.Reader
				var tmp [4]byte
				_, err := rand.Read(tmp[:])
				if err != nil {
					return fmt.Errorf("gzhttp: %w", err)
				}
				jitRNG = binary.LittleEndian.Uint32(tmp[:])
			}
			jit := w.randomJitter[:1+jitRNG%uint32(len(w.randomJitter)-1)]
			w.gw.(writer.GzipWriterExt).SetHeader(writer.Header{Comment: jit})
		}
		n, err := w.gw.Write(w.buf)

		// This should never happen (per io.Writer docs), but if the write didn't
		// accept the entire buffer but returned no specific error, we have no clue
		// what's going on, so abort just to be safe.
		if err == nil && n < len(w.buf) {
			err = io.ErrShortWrite
		}
		w.buf = w.buf[:0]
		return err
	}
	return nil
}

// startPlain writes to sent bytes and buffer the underlying ResponseWriter without gzip.
func (w *GzipResponseWriter) startPlain() error {
	w.Header().Del(HeaderNoCompression)
	if w.code != 0 {
		w.ResponseWriter.WriteHeader(w.code)
		// Ensure that no other WriteHeader's happen
		w.code = 0
	}

	w.ignore = true
	// If Write was never called then don't call Write on the underlying ResponseWriter.
	if len(w.buf) == 0 {
		return nil
	}
	n, err := w.ResponseWriter.Write(w.buf)
	// This should never happen (per io.Writer docs), but if the write didn't
	// accept the entire buffer but returned no specific error, we have no clue
	// what's going on, so abort just to be safe.
	if err == nil && n < len(w.buf) {
		err = io.ErrShortWrite
	}

	w.buf = w.buf[:0]
	return err
}

// WriteHeader just saves the response code until close or GZIP effective writes.
// In the specific case of 1xx status codes, WriteHeader is directly calling the wrapped ResponseWriter.
func (w *GzipResponseWriter) WriteHeader(code int) {
	// Handle informational headers
	// This is gated to not forward 1xx responses on builds prior to go1.20.
	if code >= 100 && code <= 199 {
		w.ResponseWriter.WriteHeader(code)
		return
	}

	if w.code == 0 {
		w.code = code
	}
}

// init graps a new gzip writer from the gzipWriterPool and writes the correct
// content encoding header.
func (w *GzipResponseWriter) init() {
	// Bytes written during ServeHTTP are redirected to this gzip writer
	// before being written to the underlying response.
	w.gw = w.gwFactory.New(w.ResponseWriter, w.level)
}

// Close will close the gzip.Writer and will put it back in the gzipWriterPool.
func (w *GzipResponseWriter) Close() error {
	if w.ignore {
		return nil
	}
	if w.gw == nil {
		var (
			ct = w.Header().Get(contentType)
			ce = w.Header().Get(contentEncoding)
			cr = w.Header().Get(contentRange)
		)
		if ct == "" {
			ct = http.DetectContentType(w.buf)

			// Handles the intended case of setting a nil Content-Type (as for http/server or http/fs)
			// Set the header only if the key does not exist
			if _, ok := w.Header()[contentType]; w.setContentType && !ok {
				w.Header().Set(contentType, ct)
			}
		}

		if len(w.buf) == 0 || len(w.buf) < w.minSize || len(w.Header()[HeaderNoCompression]) != 0 || ce != "" || cr != "" || !w.contentTypeFilter(ct) {
			// GZIP not triggered, write out regular response.
			return w.startPlain()
		}
		err := w.startGzip(nil)
		if err != nil {
			return err
		}
	}

	err := w.gw.Close()
	w.gw = nil
	return err
}

// Flush flushes the underlying *gzip.Writer and then the underlying
// http.ResponseWriter if it is an http.Flusher. This makes GzipResponseWriter
// an http.Flusher.
// If not enough bytes has been written to determine if we have reached minimum size,
// this will be ignored.
// If nothing has been written yet, nothing will be flushed.
func (w *GzipResponseWriter) Flush() {
	if w.gw == nil && !w.ignore {
		if len(w.buf) == 0 {
			// Nothing written yet.
			return
		}
		var (
			cl, _ = atoi(w.Header().Get(contentLength))
			ct    = w.Header().Get(contentType)
			ce    = w.Header().Get(contentEncoding)
			cr    = w.Header().Get(contentRange)
		)

		if ct == "" {
			ct = http.DetectContentType(w.buf)

			// Handles the intended case of setting a nil Content-Type (as for http/server or http/fs)
			// Set the header only if the key does not exist
			if _, ok := w.Header()[contentType]; w.setContentType && !ok {
				w.Header().Set(contentType, ct)
			}
		}
		if cl == 0 {
			// Assume minSize.
			cl = w.minSize
		}

		// See if we should compress...
		if len(w.Header()[HeaderNoCompression]) == 0 && ce == "" && cr == "" && cl >= w.minSize && w.contentTypeFilter(ct) {
			w.startGzip(nil)
		} else {
			w.startPlain()
		}
	}

	if w.gw != nil {
		w.gw.Flush()
	}

	if fw, ok := w.ResponseWriter.(http.Flusher); ok {
		fw.Flush()
	}
}

// Hijack implements http.Hijacker. If the underlying ResponseWriter is a
// Hijacker, its Hijack method is returned. Otherwise an error is returned.
func (w *GzipResponseWriter) Hijack() (net.Conn, *bufio.ReadWriter, error) {
	if hj, ok := w.ResponseWriter.(http.Hijacker); ok {
		return hj.Hijack()
	}
	return nil, nil, fmt.Errorf("http.Hijacker interface is not supported")
}

// verify Hijacker interface implementation
var _ http.Hijacker = &GzipResponseWriter{}

var onceDefault sync.Once
var defaultWrapper func(http.Handler) http.HandlerFunc

// GzipHandler allows to easily wrap an http handler with default settings.
func GzipHandler(h http.Handler) http.HandlerFunc {
	onceDefault.Do(func() {
		var err error
		defaultWrapper, err = NewWrapper()
		if err != nil {
			panic(err)
		}
	})

	return defaultWrapper(h)
}

var grwPool = sync.Pool{New: func() interface{} { return &GzipResponseWriter{} }}

// NewWrapper returns a reusable wrapper with the supplied options.
func NewWrapper(opts ...option) (func(http.Handler) http.HandlerFunc, error) {
	c := &config{
		level:   gzip.DefaultCompression,
		minSize: DefaultMinSize,
		writer: writer.GzipWriterFactory{
			Levels: gzkp.Levels,
			New:    gzkp.NewWriter,
		},
		contentTypes:   DefaultContentTypeFilter,
		setContentType: true,
	}

	for _, o := range opts {
		o(c)
	}

	if err := c.validate(); err != nil {
		return nil, err
	}

	return func(h http.Handler) http.HandlerFunc {
		return func(w http.ResponseWriter, r *http.Request) {
			w.Header().Add(vary, acceptEncoding)
			if c.allowCompressedRequests && contentGzip(r) {
				r.Header.Del(contentEncoding)
				r.Body = &gzipReader{body: r.Body}
			}

			if acceptsGzip(r) {
				gw := grwPool.Get().(*GzipResponseWriter)
				*gw = GzipResponseWriter{
					ResponseWriter:    w,
					gwFactory:         c.writer,
					level:             c.level,
					minSize:           c.minSize,
					contentTypeFilter: c.contentTypes,
					keepAcceptRanges:  c.keepAcceptRanges,
					dropETag:          c.dropETag,
					suffixETag:        c.suffixETag,
					buf:               gw.buf,
					setContentType:    c.setContentType,
					randomJitter:      c.randomJitter,
					jitterBuffer:      c.jitterBuffer,
					sha256Jitter:      c.sha256Jitter,
				}
				if len(gw.buf) > 0 {
					gw.buf = gw.buf[:0]
				}
				defer func() {
					gw.Close()
					gw.ResponseWriter = nil
					grwPool.Put(gw)
				}()

				if _, ok := w.(http.CloseNotifier); ok {
					gwcn := GzipResponseWriterWithCloseNotify{gw}
					h.ServeHTTP(gwcn, r)
				} else {
					h.ServeHTTP(gw, r)
				}
				w.Header().Del(HeaderNoCompression)
			} else {
				h.ServeHTTP(newNoGzipResponseWriter(w), r)
				w.Header().Del(HeaderNoCompression)
			}
		}
	}, nil
}

// Parsed representation of one of the inputs to ContentTypes.
// See https://golang.org/pkg/mime/#ParseMediaType
type parsedContentType struct {
	mediaType string
	params    map[string]string
}

// equals returns whether this content type matches another content type.
func (pct parsedContentType) equals(mediaType string, params map[string]string) bool {
	if pct.mediaType != mediaType {
		return false
	}
	// if pct has no params, don't care about other's params
	if len(pct.params) == 0 {
		return true
	}

	// if pct has any params, they must be identical to other's.
	if len(pct.params) != len(params) {
		return false
	}
	for k, v := range pct.params {
		if w, ok := params[k]; !ok || v != w {
			return false
		}
	}
	return true
}

// Used for functional configuration.
type config struct {
	minSize                 int
	level                   int
	writer                  writer.GzipWriterFactory
	contentTypes            func(ct string) bool
	keepAcceptRanges        bool
	setContentType          bool
	suffixETag              string
	dropETag                bool
	jitterBuffer            int
	randomJitter            string
	sha256Jitter            bool
	allowCompressedRequests bool
}

func (c *config) validate() error {
	min, max := c.writer.Levels()
	if c.level < min || c.level > max {
		return fmt.Errorf("invalid compression level requested: %d, valid range %d -> %d", c.level, min, max)
	}

	if c.minSize < 0 {
		return fmt.Errorf("minimum size must be more than zero")
	}
	if len(c.randomJitter) >= math.MaxUint16 {
		return fmt.Errorf("random jitter size exceeded")
	}
	if len(c.randomJitter) > 0 {
		gzw, ok := c.writer.New(io.Discard, c.level).(writer.GzipWriterExt)
		if !ok {
			return errors.New("the custom compressor does not allow setting headers for random jitter")
		}
		gzw.Close()
	}
	return nil
}

type option func(c *config)

func MinSize(size int) option {
	return func(c *config) {
		c.minSize = size
	}
}

// AllowCompressedRequests will enable or disable RFC 7694 compressed requests.
// By default this is Disabled.
// See https://datatracker.ietf.org/doc/html/rfc7694
func AllowCompressedRequests(b bool) option {
	return func(c *config) {
		c.allowCompressedRequests = b
	}
}

// CompressionLevel sets the compression level
func CompressionLevel(level int) option {
	return func(c *config) {
		c.level = level
	}
}

// SetContentType sets the content type before returning
// requests, if unset before returning, and it was detected.
// Default: true.
func SetContentType(b bool) option {
	return func(c *config) {
		c.setContentType = b
	}
}

// Implementation changes the implementation of GzipWriter
//
// The default implementation is backed by github.com/klauspost/compress
// To support RandomJitter, the GzipWriterExt must also be
// supported by the returned writers.
func Implementation(writer writer.GzipWriterFactory) option {
	return func(c *config) {
		c.writer = writer
	}
}

// ContentTypes specifies a list of content types to compare
// the Content-Type header to before compressing. If none
// match, the response will be returned as-is.
//
// Content types are compared in a case-insensitive, whitespace-ignored
// manner.
//
// A MIME type without any other directive will match a content type
// that has the same MIME type, regardless of that content type's other
// directives. I.e., "text/html" will match both "text/html" and
// "text/html; charset=utf-8".
//
// A MIME type with any other directive will only match a content type
// that has the same MIME type and other directives. I.e.,
// "text/html; charset=utf-8" will only match "text/html; charset=utf-8".
//
// By default common compressed audio, video and archive formats, see DefaultContentTypeFilter.
//
// Setting this will override default and any previous Content Type settings.
func ContentTypes(types []string) option {
	return func(c *config) {
		var contentTypes []parsedContentType
		for _, v := range types {
			mediaType, params, err := mime.ParseMediaType(v)
			if err == nil {
				contentTypes = append(contentTypes, parsedContentType{mediaType, params})
			}
		}
		c.contentTypes = func(ct string) bool {
			return handleContentType(contentTypes, ct)
		}
	}
}

// ExceptContentTypes specifies a list of content types to compare
// the Content-Type header to before compressing. If none
// match, the response will be compressed.
//
// Content types are compared in a case-insensitive, whitespace-ignored
// manner.
//
// A MIME type without any other directive will match a content type
// that has the same MIME type, regardless of that content type's other
// directives. I.e., "text/html" will match both "text/html" and
// "text/html; charset=utf-8".
//
// A MIME type with any other directive will only match a content type
// that has the same MIME type and other directives. I.e.,
// "text/html; charset=utf-8" will only match "text/html; charset=utf-8".
//
// By default common compressed audio, video and archive formats, see DefaultContentTypeFilter.
//
// Setting this will override default and any previous Content Type settings.
func ExceptContentTypes(types []string) option {
	return func(c *config) {
		var contentTypes []parsedContentType
		for _, v := range types {
			mediaType, params, err := mime.ParseMediaType(v)
			if err == nil {
				contentTypes = append(contentTypes, parsedContentType{mediaType, params})
			}
		}
		c.contentTypes = func(ct string) bool {
			return !handleContentType(contentTypes, ct)
		}
	}
}

// KeepAcceptRanges will keep Accept-Ranges header on gzipped responses.
// This will likely break ranged requests since that cannot be transparently
// handled by the filter.
func KeepAcceptRanges() option {
	return func(c *config) {
		c.keepAcceptRanges = true
	}
}

// ContentTypeFilter allows adding a custom content type filter.
//
// The supplied function must return true/false to indicate if content
// should be compressed.
//
// When called no parsing of the content type 'ct' has been done.
// It may have been set or auto-detected.
//
// Setting this will override default and any previous Content Type settings.
func ContentTypeFilter(compress func(ct string) bool) option {
	return func(c *config) {
		c.contentTypes = compress
	}
}

// SuffixETag adds the specified suffix to the ETag header (if it exists) of
// responses which are compressed.
//
// Per [RFC 7232 Section 2.3.3](https://www.rfc-editor.org/rfc/rfc7232#section-2.3.3),
// the ETag of a compressed response must differ from it's uncompressed version.
//
// A suffix such as "-gzip" is sometimes used as a workaround for generating a
// unique new ETag (see https://bz.apache.org/bugzilla/show_bug.cgi?id=39727).
func SuffixETag(suffix string) option {
	return func(c *config) {
		c.suffixETag = suffix
	}
}

// DropETag removes the ETag of responses which are compressed. If DropETag is
// specified in conjunction with SuffixETag, this option will take precedence
// and the ETag will be dropped.
//
// Per [RFC 7232 Section 2.3.3](https://www.rfc-editor.org/rfc/rfc7232#section-2.3.3),
// the ETag of a compressed response must differ from it's uncompressed version.
//
// This workaround eliminates ETag conflicts between the compressed and
// uncompressed versions by removing the ETag from the compressed version.
func DropETag() option {
	return func(c *config) {
		c.dropETag = true
	}
}

// RandomJitter adds 1->n random bytes to output based on checksum of payload.
// Specify the amount of input to buffer before applying jitter.
// This should cover the sensitive part of your response.
// This can be used to obfuscate the exact compressed size.
// Specifying 0 will use a buffer size of 64KB.
// 'paranoid' will use a slower hashing function, that MAY provide more safety.
// See README.md for more information.
// If a negative buffer is given, the amount of jitter will not be content dependent.
// This provides *less* security than applying content based jitter.
func RandomJitter(n, buffer int, paranoid bool) option {
	return func(c *config) {
		if n > 0 {
			c.sha256Jitter = paranoid
			c.randomJitter = strings.Repeat("Padding-", 1+(n/8))[:n+1]
			c.jitterBuffer = buffer
			if c.jitterBuffer == 0 {
				c.jitterBuffer = 64 << 10
			}
		} else {
			c.randomJitter = ""
			c.jitterBuffer = 0
		}
	}
}

// contentGzip returns true if the given HTTP request indicates that it gzipped.
func contentGzip(r *http.Request) bool {
	// See more detail in `acceptsGzip`
	return r.Method != http.MethodHead && r.Body != nil && parseEncodingGzip(r.Header.Get(contentEncoding)) > 0
}

// acceptsGzip returns true if the given HTTP request indicates that it will
// accept a gzipped response.
func acceptsGzip(r *http.Request) bool {
	// Note that we don't request this for HEAD requests,
	// due to a bug in nginx:
	//   https://trac.nginx.org/nginx/ticket/358
	//   https://golang.org/issue/5522
	return r.Method != http.MethodHead && parseEncodingGzip(r.Header.Get(acceptEncoding)) > 0
}

// returns true if we've been configured to compress the specific content type.
func handleContentType(contentTypes []parsedContentType, ct string) bool {
	// If contentTypes is empty we handle all content types.
	if len(contentTypes) == 0 {
		return true
	}

	mediaType, params, err := mime.ParseMediaType(ct)
	if err != nil {
		return false
	}

	for _, c := range contentTypes {
		if c.equals(mediaType, params) {
			return true
		}
	}

	return false
}

// parseEncodingGzip returns the qvalue of gzip compression.
func parseEncodingGzip(s string) float64 {
	s = strings.TrimSpace(s)

	for len(s) > 0 {
		stop := strings.IndexByte(s, ',')
		if stop < 0 {
			stop = len(s)
		}
		coding, qvalue, _ := parseCoding(s[:stop])

		if coding == "gzip" {
			return qvalue
		}
		if stop == len(s) {
			break
		}
		s = s[stop+1:]
	}
	return 0
}

func parseEncodings(s string) (codings, error) {
	split := strings.Split(s, ",")
	c := make(codings, len(split))
	var e []string

	for _, ss := range split {
		coding, qvalue, err := parseCoding(ss)

		if err != nil {
			e = append(e, err.Error())
		} else {
			c[coding] = qvalue
		}
	}

	// TODO (adammck): Use a proper multi-error struct, so the individual errors
	//                 can be extracted if anyone cares.
	if len(e) > 0 {
		return c, fmt.Errorf("errors while parsing encodings: %s", strings.Join(e, ", "))
	}

	return c, nil
}

var errEmptyEncoding = errors.New("empty content-coding")

// parseCoding parses a single coding (content-coding with an optional qvalue),
// as might appear in an Accept-Encoding header. It attempts to forgive minor
// formatting errors.
func parseCoding(s string) (coding string, qvalue float64, err error) {
	// Avoid splitting if we can...
	if len(s) == 0 {
		return "", 0, errEmptyEncoding
	}
	if !strings.ContainsRune(s, ';') {
		coding = strings.ToLower(strings.TrimSpace(s))
		if coding == "" {
			err = errEmptyEncoding
		}
		return coding, DefaultQValue, err
	}
	for n, part := range strings.Split(s, ";") {
		part = strings.TrimSpace(part)
		qvalue = DefaultQValue

		if n == 0 {
			coding = strings.ToLower(part)
		} else if strings.HasPrefix(part, "q=") {
			qvalue, err = strconv.ParseFloat(strings.TrimPrefix(part, "q="), 64)

			if qvalue < 0.0 {
				qvalue = 0.0
			} else if qvalue > 1.0 {
				qvalue = 1.0
			}
		}
	}

	if coding == "" {
		err = errEmptyEncoding
	}

	return
}

// Don't compress any audio/video types.
var excludePrefixDefault = []string{"video/", "audio/", "image/jp"}

// Skip a bunch of compressed types that contains this string.
// Curated by supposedly still active formats on https://en.wikipedia.org/wiki/List_of_archive_formats
var excludeContainsDefault = []string{"compress", "zip", "snappy", "lzma", "xz", "zstd", "brotli", "stuffit"}

// DefaultContentTypeFilter excludes common compressed audio, video and archive formats.
func DefaultContentTypeFilter(ct string) bool {
	ct = strings.TrimSpace(strings.ToLower(ct))
	if ct == "" {
		return true
	}
	for _, s := range excludeContainsDefault {
		if strings.Contains(ct, s) {
			return false
		}
	}

	for _, prefix := range excludePrefixDefault {
		if strings.HasPrefix(ct, prefix) {
			return false
		}
	}
	return true
}

// CompressAllContentTypeFilter will compress all mime types.
func CompressAllContentTypeFilter(ct string) bool {
	return true
}

const intSize = 32 << (^uint(0) >> 63)

// atoi is equivalent to ParseInt(s, 10, 0), converted to type int.
func atoi(s string) (int, bool) {
	if len(s) == 0 {
		return 0, false
	}
	sLen := len(s)
	if intSize == 32 && (0 < sLen && sLen < 10) ||
		intSize == 64 && (0 < sLen && sLen < 19) {
		// Fast path for small integers that fit int type.
		s0 := s
		if s[0] == '-' || s[0] == '+' {
			s = s[1:]
			if len(s) < 1 {
				return 0, false
			}
		}

		n := 0
		for _, ch := range []byte(s) {
			ch -= '0'
			if ch > 9 {
				return 0, false
			}
			n = n*10 + int(ch)
		}
		if s0[0] == '-' {
			n = -n
		}
		return n, true
	}

	// Slow path for invalid, big, or underscored integers.
	i64, err := strconv.ParseInt(s, 10, 0)
	return int(i64), err == nil
}

type unwrapper interface {
	Unwrap() http.ResponseWriter
}

// newNoGzipResponseWriter will return a response writer that
// cleans up compression artifacts.
// Depending on whether http.Hijacker is supported the returned will as well.
func newNoGzipResponseWriter(w http.ResponseWriter) http.ResponseWriter {
	n := &NoGzipResponseWriter{ResponseWriter: w}
	if hj, ok := w.(http.Hijacker); ok {
		x := struct {
			http.ResponseWriter
			http.Hijacker
			http.Flusher
			unwrapper
		}{
			ResponseWriter: n,
			Hijacker:       hj,
			Flusher:        n,
			unwrapper:      n,
		}
		return x
	}

	return n
}

// NoGzipResponseWriter filters out HeaderNoCompression.
type NoGzipResponseWriter struct {
	http.ResponseWriter
	hdrCleaned bool
}

func (n *NoGzipResponseWriter) CloseNotify() <-chan bool {
	if cn, ok := n.ResponseWriter.(http.CloseNotifier); ok {
		return cn.CloseNotify()
	}
	return nil
}

func (n *NoGzipResponseWriter) Flush() {
	if !n.hdrCleaned {
		n.ResponseWriter.Header().Del(HeaderNoCompression)
		n.hdrCleaned = true
	}
	if f, ok := n.ResponseWriter.(http.Flusher); ok {
		f.Flush()
	}
}

func (n *NoGzipResponseWriter) Header() http.Header {
	return n.ResponseWriter.Header()
}

func (n *NoGzipResponseWriter) Write(bytes []byte) (int, error) {
	if !n.hdrCleaned {
		n.ResponseWriter.Header().Del(HeaderNoCompression)
		n.hdrCleaned = true
	}
	return n.ResponseWriter.Write(bytes)
}

func (n *NoGzipResponseWriter) WriteHeader(statusCode int) {
	if !n.hdrCleaned {
		n.ResponseWriter.Header().Del(HeaderNoCompression)
		n.hdrCleaned = true
	}
	n.ResponseWriter.WriteHeader(statusCode)
}

func (n *NoGzipResponseWriter) Unwrap() http.ResponseWriter {
	return n.ResponseWriter
}
