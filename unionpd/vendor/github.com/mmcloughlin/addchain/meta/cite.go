package meta

import (
	"bytes"
	"errors"
	"fmt"
	"io"
	"strconv"
	"strings"

	"github.com/mmcloughlin/addchain/internal/print"
)

// CheckCitable checks whether a citation can be generated for this built
// version.
func (p *Properties) CheckCitable() error {
	if !p.IsRelease() {
		return errors.New("cannot cite non-release version")
	}
	return nil
}

// WriteCitation writes BibTeX citation for the most recent release to the given
// writer.
func (p *Properties) WriteCitation(w io.Writer) error {
	// Determine release time.
	date, err := p.ReleaseTime()
	if err != nil {
		return fmt.Errorf("release date: %w", err)
	}

	// Use tabwriter for field alignment.
	tw := print.NewTabWriter(w, 1, 4, 1, ' ', 0)

	field := func(key, value string) { tw.Linef("    %s\t=\t%s,", key, value) }
	str := func(key, value string) { field(key, "{"+value+"}") }

	tw.Linef("@misc{%s,", p.Name)
	str("title", p.Title())
	str("author", "Michael B. McLoughlin")
	field("year", strconv.Itoa(date.Year()))
	field("month", strings.ToLower(date.Month().String()[:3]))
	str("howpublished", "Repository \\url{"+p.RepositoryURL()+"}")
	str("version", p.ReleaseVersion)
	str("license", "BSD 3-Clause License")
	str("doi", p.DOI)
	str("url", p.DOIURL())
	tw.Linef("}")
	tw.Flush()

	return tw.Error()
}

// Citation returns a BibTeX citation for the most recent release.
func (p *Properties) Citation() (string, error) {
	buf := bytes.NewBuffer(nil)
	if err := p.WriteCitation(buf); err != nil {
		return "", err
	}
	return buf.String(), nil
}
