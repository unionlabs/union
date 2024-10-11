//go:build codegen
// +build codegen

package api

import (
	"bufio"
	"encoding/json"
	"encoding/xml"
	"fmt"
	"html"
	"io"
	"log"
	"os"
	"regexp"
	"strings"
)

type apiDocumentation struct {
	Operations map[string]string
	Service    string
	Shapes     map[string]shapeDocumentation
}

type shapeDocumentation struct {
	Base string
	Refs map[string]string
}

// AttachDocs attaches documentation from a JSON filename.
func (a *API) AttachDocs(filename string) error {
	var d apiDocumentation

	f, err := os.Open(filename)
	defer f.Close()
	if err != nil {
		return err
	}
	err = json.NewDecoder(f).Decode(&d)
	if err != nil {
		return fmt.Errorf("failed to decode %s, err: %v", filename, err)
	}

	return d.setup(a)
}

func (d *apiDocumentation) setup(a *API) error {
	a.Documentation = docstring(d.Service)

	for opName, doc := range d.Operations {
		if _, ok := a.Operations[opName]; !ok {
			continue
		}
		a.Operations[opName].Documentation = docstring(doc)
	}

	for shapeName, docShape := range d.Shapes {
		if s, ok := a.Shapes[shapeName]; ok {
			s.Documentation = docstring(docShape.Base)
		}

		for ref, doc := range docShape.Refs {
			if doc == "" {
				continue
			}

			parts := strings.Split(ref, "$")
			if len(parts) != 2 {
				fmt.Fprintf(os.Stderr,
					"Shape Doc %s has unexpected reference format, %q\n",
					shapeName, ref)
				continue
			}

			if s, ok := a.Shapes[parts[0]]; ok && len(s.MemberRefs) != 0 {
				if m, ok := s.MemberRefs[parts[1]]; ok && m.ShapeName == shapeName {
					m.Documentation = docstring(doc)
				}
			}
		}
	}

	return nil
}

var reNewline = regexp.MustCompile(`\r?\n`)
var reMultiSpace = regexp.MustCompile(`\s+`)
var reComments = regexp.MustCompile(`<!--.*?-->`)
var reFullnameBlock = regexp.MustCompile(`<fullname>(.+?)<\/fullname>`)
var reFullname = regexp.MustCompile(`<fullname>(.*?)</fullname>`)
var reExamples = regexp.MustCompile(`<examples?>.+?<\/examples?>`)
var reEndNL = regexp.MustCompile(`\n+$`)

// docstring rewrites a string to insert godocs formatting.
func docstring(doc string) string {
	doc = strings.TrimSpace(doc)
	if doc == "" {
		return ""
	}

	doc = reNewline.ReplaceAllString(doc, "")
	doc = reMultiSpace.ReplaceAllString(doc, " ")
	doc = reComments.ReplaceAllString(doc, "")

	var fullname string
	parts := reFullnameBlock.FindStringSubmatch(doc)
	if len(parts) > 1 {
		fullname = parts[1]
	}
	// Remove full name block from doc string
	doc = reFullname.ReplaceAllString(doc, "")

	doc = reExamples.ReplaceAllString(doc, "")
	doc = generateDoc(doc)
	doc = reEndNL.ReplaceAllString(doc, "")
	doc = html.UnescapeString(doc)

	// Replace doc with full name if doc is empty.
	if len(doc) == 0 {
		doc = fullname
	}

	return commentify(doc)
}

const (
	indent = "   "
)

// commentify converts a string to a Go comment
func commentify(doc string) string {
	if len(doc) == 0 {
		return ""
	}

	lines := strings.Split(doc, "\n")
	out := make([]string, 0, len(lines))
	for i := 0; i < len(lines); i++ {
		line := lines[i]

		if i > 0 && line == "" && lines[i-1] == "" {
			continue
		}
		out = append(out, line)
	}

	if len(out) > 0 {
		out[0] = "// " + out[0]
		return strings.Join(out, "\n// ")
	}
	return ""
}

func wrap(text string, length int) string {
	var b strings.Builder

	s := bufio.NewScanner(strings.NewReader(text))
	for s.Scan() {
		line := s.Text()

		// cleanup the line's spaces
		var i int
		for i = 0; i < len(line); i++ {
			c := line[i]
			// Ignore leading spaces, e.g indents.
			if !(c == ' ' || c == '\t') {
				break
			}
		}
		line = line[:i] + strings.Join(strings.Fields(line[i:]), " ")
		splitLine(&b, line, length)
	}

	return strings.TrimRight(b.String(), "\n")
}

func splitLine(w stringWriter, line string, length int) {
	leading := getLeadingWhitespace(line)

	line = line[len(leading):]
	length -= len(leading)

	const splitOn = " "
	for len(line) > length {
		// Find the next whitespace to the length
		idx := strings.Index(line[length:], splitOn)
		if idx == -1 {
			break
		}
		offset := length + idx

		if v := line[offset+len(splitOn):]; len(v) == 1 && strings.ContainsAny(v, `,.!?'"`) {
			// Workaround for long lines with space before the punctuation mark.
			break
		}

		w.WriteString(leading)
		w.WriteString(line[:offset])
		w.WriteByte('\n')
		line = strings.TrimLeft(line[offset+len(splitOn):], " \t")
	}

	if len(line) > 0 {
		w.WriteString(leading)
		w.WriteString(line)
	}
	// Add the newline back in that was stripped out by scanner.
	w.WriteByte('\n')
}

func getLeadingWhitespace(v string) string {
	var o strings.Builder
	for _, c := range v {
		if c == ' ' || c == '\t' {
			o.WriteRune(c)
		} else {
			break
		}
	}

	return o.String()
}

// generateDoc will generate the proper doc string for html encoded or plain text doc entries.
func generateDoc(htmlSrc string) string {
	tokenizer := xml.NewDecoder(strings.NewReader(htmlSrc))
	tokenizer.Strict = false
	tokenizer.AutoClose = xml.HTMLAutoClose
	tokenizer.Entity = xml.HTMLEntity

	// Some service docstrings are hopelessly malformed. Rather than throwing
	// up our hands, the converter will map over as much as it can. If it
	// returns an error, we stop there and go with whatever it was able to
	// produce.
	var builder strings.Builder
	encodeHTMLToText(&builder, tokenizer)
	return wrap(strings.Trim(builder.String(), "\n"), 72)
}

type stringWriter interface {
	Write([]byte) (int, error)
	WriteByte(byte) error
	WriteRune(rune) (int, error)
	WriteString(string) (int, error)
}

func encodeHTMLToText(w stringWriter, z *xml.Decoder) error {
	encoder := newHTMLTokenEncoder(w)
	defer encoder.Flush()

	for {
		tt, err := z.Token()
		if err == io.EOF {
			return nil
		}
		if err != nil {
			return err
		}

		if err := encoder.Encode(tt); err != nil {
			return err
		}
	}
}

type htmlTokenHandler interface {
	OnStartTagToken(xml.StartElement) htmlTokenHandler
	OnEndTagToken(xml.Token, bool)
	OnSelfClosingTagToken(xml.Token)
	OnTextTagToken(xml.CharData)
}

type htmlTokenEncoder struct {
	w           stringWriter
	depth       int
	handlers    []tokenHandlerItem
	baseHandler tokenHandlerItem
}

type tokenHandlerItem struct {
	handler htmlTokenHandler
	depth   int
}

func newHTMLTokenEncoder(w stringWriter) *htmlTokenEncoder {
	baseHandler := newBlockTokenHandler(w)
	baseHandler.rootBlock = true

	return &htmlTokenEncoder{
		w: w,
		baseHandler: tokenHandlerItem{
			handler: baseHandler,
		},
	}
}

func (e *htmlTokenEncoder) Flush() error {
	e.baseHandler.handler.OnEndTagToken(xml.CharData([]byte{}), true)
	return nil
}

func (e *htmlTokenEncoder) Encode(token xml.Token) error {
	h := e.baseHandler
	if len(e.handlers) != 0 {
		h = e.handlers[len(e.handlers)-1]
	}

	switch v := token.(type) {
	case xml.StartElement:
		e.depth++

		next := h.handler.OnStartTagToken(v)
		if next != nil {
			e.handlers = append(e.handlers, tokenHandlerItem{
				handler: next,
				depth:   e.depth,
			})
		}

	case xml.EndElement:
		handlerBlockClosing := e.depth == h.depth

		h.handler.OnEndTagToken(token, handlerBlockClosing)

		// Remove all but the root handler as the handler is no longer needed.
		if handlerBlockClosing && len(e.handlers) != 0 {
			e.handlers = e.handlers[:len(e.handlers)-1]
		}
		e.depth--
		if e.depth < 0 {
			log.Printf("ignoring unexpected closing tag, %v", token)
			e.depth = 0
		}

	case xml.CharData:
		h.handler.OnTextTagToken(v)
	}

	return nil
}

type baseTokenHandler struct {
	w stringWriter
}

func (e *baseTokenHandler) OnStartTagToken(token xml.StartElement) htmlTokenHandler { return nil }
func (e *baseTokenHandler) OnEndTagToken(token xml.Token, blockClosing bool)        {}
func (e *baseTokenHandler) OnSelfClosingTagToken(token xml.Token)                   {}
func (e *baseTokenHandler) OnTextTagToken(token xml.CharData) {
	e.w.WriteString(string(token))
}

type blockTokenHandler struct {
	baseTokenHandler

	rootBlock  bool
	origWriter stringWriter
	strBuilder *strings.Builder

	started                bool
	newlineBeforeNextBlock bool
}

func newBlockTokenHandler(w stringWriter) *blockTokenHandler {
	strBuilder := &strings.Builder{}
	return &blockTokenHandler{
		origWriter: w,
		strBuilder: strBuilder,
		baseTokenHandler: baseTokenHandler{
			w: strBuilder,
		},
	}
}
func (e *blockTokenHandler) OnStartTagToken(token xml.StartElement) htmlTokenHandler {
	e.started = true
	if e.newlineBeforeNextBlock {
		e.w.WriteString("\n")
		e.newlineBeforeNextBlock = false
	}

	switch token.Name.Local {
	case "a":
		return newLinkTokenHandler(e.w, token)
	case "ul":
		e.w.WriteString("\n")
		e.newlineBeforeNextBlock = true
		return newListTokenHandler(e.w)

	case "div", "dt", "p", "h1", "h2", "h3", "h4", "h5", "h6":
		e.w.WriteString("\n")
		e.newlineBeforeNextBlock = true
		return newBlockTokenHandler(e.w)

	case "pre", "code":
		if e.rootBlock {
			e.w.WriteString("\n")
			e.w.WriteString(indent)
			e.newlineBeforeNextBlock = true
		}
		return newBlockTokenHandler(e.w)
	}

	return nil
}
func (e *blockTokenHandler) OnEndTagToken(token xml.Token, blockClosing bool) {
	if !blockClosing {
		return
	}

	e.origWriter.WriteString(e.strBuilder.String())
	if e.newlineBeforeNextBlock {
		e.origWriter.WriteString("\n")
		e.newlineBeforeNextBlock = false
	}

	e.strBuilder.Reset()
}

func (e *blockTokenHandler) OnTextTagToken(token xml.CharData) {
	if e.newlineBeforeNextBlock {
		e.w.WriteString("\n")
		e.newlineBeforeNextBlock = false
	}
	if !e.started {
		token = xml.CharData(strings.TrimLeft(string(token), " \t\n"))
	}
	if len(token) != 0 {
		e.started = true
	}
	e.baseTokenHandler.OnTextTagToken(token)
}

type linkTokenHandler struct {
	baseTokenHandler
	linkToken xml.StartElement
}

func newLinkTokenHandler(w stringWriter, token xml.StartElement) *linkTokenHandler {
	return &linkTokenHandler{
		baseTokenHandler: baseTokenHandler{
			w: w,
		},
		linkToken: token,
	}
}
func (e *linkTokenHandler) OnEndTagToken(token xml.Token, blockClosing bool) {
	if !blockClosing {
		return
	}

	if href, ok := getHTMLTokenAttr(e.linkToken.Attr, "href"); ok && len(href) != 0 {
		fmt.Fprintf(e.w, " (%s)", strings.TrimSpace(href))
	}
}

type listTokenHandler struct {
	baseTokenHandler

	items int
}

func newListTokenHandler(w stringWriter) *listTokenHandler {
	return &listTokenHandler{
		baseTokenHandler: baseTokenHandler{
			w: w,
		},
	}
}
func (e *listTokenHandler) OnStartTagToken(token xml.StartElement) htmlTokenHandler {
	switch token.Name.Local {
	case "li":
		if e.items >= 1 {
			e.w.WriteString("\n\n")
		}
		e.items++
		return newListItemTokenHandler(e.w)
	}
	return nil
}

func (e *listTokenHandler) OnTextTagToken(token xml.CharData) {
	// Squash whitespace between list and items
}

type listItemTokenHandler struct {
	baseTokenHandler

	origWriter stringWriter
	strBuilder *strings.Builder
}

func newListItemTokenHandler(w stringWriter) *listItemTokenHandler {
	strBuilder := &strings.Builder{}
	return &listItemTokenHandler{
		origWriter: w,
		strBuilder: strBuilder,
		baseTokenHandler: baseTokenHandler{
			w: strBuilder,
		},
	}
}
func (e *listItemTokenHandler) OnStartTagToken(token xml.StartElement) htmlTokenHandler {
	switch token.Name.Local {
	case "p":
		return newBlockTokenHandler(e.w)
	}
	return nil
}
func (e *listItemTokenHandler) OnEndTagToken(token xml.Token, blockClosing bool) {
	if !blockClosing {
		return
	}

	e.origWriter.WriteString(indent + "* ")
	e.origWriter.WriteString(strings.TrimSpace(e.strBuilder.String()))
}

type trimSpaceTokenHandler struct {
	baseTokenHandler

	origWriter stringWriter
	strBuilder *strings.Builder
}

func newTrimSpaceTokenHandler(w stringWriter) *trimSpaceTokenHandler {
	strBuilder := &strings.Builder{}
	return &trimSpaceTokenHandler{
		origWriter: w,
		strBuilder: strBuilder,
		baseTokenHandler: baseTokenHandler{
			w: strBuilder,
		},
	}
}
func (e *trimSpaceTokenHandler) OnEndTagToken(token xml.Token, blockClosing bool) {
	if !blockClosing {
		return
	}

	e.origWriter.WriteString(strings.TrimSpace(e.strBuilder.String()))
}

func getHTMLTokenAttr(attr []xml.Attr, name string) (string, bool) {
	for _, a := range attr {
		if strings.EqualFold(a.Name.Local, name) {
			return a.Value, true
		}
	}
	return "", false
}

func AppendDocstring(base, toAdd string) string {
	if base != "" {
		base += "\n//\n"
	}

	return base + docstring(toAdd)
}
