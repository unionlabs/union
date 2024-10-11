// Package zenodo provides a client for the Zendodo API.
package zenodo

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"mime/multipart"
	"net/http"
	"net/textproto"
	"regexp"
)

// Default API base URLs.
const (
	BaseURL        = "https://zenodo.org"
	SandboxBaseURL = "https://sandbox.zenodo.org"
)

// Client for the Zenodo API.
type Client struct {
	client *http.Client
	base   string
	token  string
}

// NewClient builds a new Zenodo client. Uses the given HTTP client, base URL
// and access token.
func NewClient(c *http.Client, base, token string) *Client {
	return &Client{
		client: c,
		base:   base,
		token:  token,
	}
}

// DepositionRetrieve retrieves a deposit.
func (c *Client) DepositionRetrieve(ctx context.Context, id string) (*Deposition, error) {
	path := fmt.Sprintf("api/deposit/depositions/%s", id)

	// Build request.
	u := c.base + "/" + path
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, u, nil)
	if err != nil {
		return nil, err
	}

	// Execute.
	d := &Deposition{}
	if err := c.request(req, d); err != nil {
		return nil, err
	}

	return d, nil
}

// DepositionCreate creates a new empty deposit.
func (c *Client) DepositionCreate(ctx context.Context) (*Deposition, error) {
	path := "api/deposit/depositions"
	empty := &Deposition{}
	d := &Deposition{}
	if err := c.requestjson(ctx, http.MethodPost, path, &empty, d); err != nil {
		return nil, err
	}
	return d, nil
}

// DepositionUpdate updates deposit metadata.
func (c *Client) DepositionUpdate(ctx context.Context, id string, meta *DepositionMetadata) (*Deposition, error) {
	path := fmt.Sprintf("api/deposit/depositions/%s", id)
	input := &Deposition{Metadata: meta}
	d := &Deposition{}
	if err := c.requestjson(ctx, http.MethodPut, path, input, d); err != nil {
		return nil, err
	}
	return d, nil
}

// DepositionNewVersion creates a new version of an existing deposit. Returns
// the ID of the new version.
func (c *Client) DepositionNewVersion(ctx context.Context, id string) (string, error) {
	// Create new version.
	d, err := c.action(ctx, id, "newversion")
	if err != nil {
		return "", err
	}

	// Parse out the new ID from the latest_draft link.
	field := "latest_draft"
	u, ok := d.Links["latest_draft"]
	if !ok {
		return "", fmt.Errorf("expected %q link", field)
	}

	match := newidrx.FindStringSubmatch(u)
	if match == nil {
		return "", fmt.Errorf("could not parse ID from %q link", field)
	}
	newid := match[1]

	return newid, nil
}

var newidrx = regexp.MustCompile(`/api/deposit/depositions/(\d+)$`)

// DepositionPublish publishes a deposit.
func (c *Client) DepositionPublish(ctx context.Context, id string) (*Deposition, error) {
	return c.action(ctx, id, "publish")
}

func (c *Client) action(ctx context.Context, id, name string) (*Deposition, error) {
	path := fmt.Sprintf("api/deposit/depositions/%s/actions/%s", id, name)

	// Build request.
	u := c.base + "/" + path
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, u, nil)
	if err != nil {
		return nil, err
	}

	// Execute.
	d := &Deposition{}
	if err := c.request(req, d); err != nil {
		return nil, err
	}
	return d, nil
}

// DepositionFilesList lists all files attached to a deposit.
func (c *Client) DepositionFilesList(ctx context.Context, id string) ([]*DepositionFile, error) {
	path := fmt.Sprintf("api/deposit/depositions/%s/files", id)

	// Build request.
	u := c.base + "/" + path
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, u, nil)
	if err != nil {
		return nil, err
	}

	// Execute.
	var fs []*DepositionFile
	if err := c.request(req, &fs); err != nil {
		return nil, err
	}
	return fs, nil
}

// DepositionFilesCreate uploads a new file to a deposit.
func (c *Client) DepositionFilesCreate(ctx context.Context, id, filename, mimetype string, r io.Reader) (*DepositionFile, error) {
	path := fmt.Sprintf("api/deposit/depositions/%s/files", id)

	// Build multipart body.
	body := &bytes.Buffer{}
	w := multipart.NewWriter(body)

	if err := w.WriteField("name", filename); err != nil {
		return nil, err
	}

	// Add file.
	hdr := make(textproto.MIMEHeader)
	hdr.Set("Content-Disposition",
		fmt.Sprintf(`form-data; name="file"; filename=%q`, filename))
	hdr.Set("Content-Type", mimetype)
	part, err := w.CreatePart(hdr)
	if err != nil {
		return nil, err
	}

	if _, err := io.Copy(part, r); err != nil {
		return nil, err
	}

	// Finalize body.
	if err := w.Close(); err != nil {
		return nil, err
	}

	// Build request.
	u := c.base + "/" + path
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, u, body)
	if err != nil {
		return nil, err
	}

	req.Header.Set("Content-Type", w.FormDataContentType())

	// Execute request.
	f := &DepositionFile{}
	if err := c.request(req, f); err != nil {
		return nil, err
	}

	return f, nil
}

// DepositionFilesDelete deletes a file from a deposit.
func (c *Client) DepositionFilesDelete(ctx context.Context, did, fid string) error {
	path := fmt.Sprintf("api/deposit/depositions/%s/files/%s", did, fid)

	// Build request.
	u := c.base + "/" + path
	req, err := http.NewRequestWithContext(ctx, http.MethodDelete, u, nil)
	if err != nil {
		return err
	}

	// Execute.
	return c.request(req, nil)
}

// DepositionFilesDeleteAll deletes all files from a deposit. This requires
// multiple API calls.
func (c *Client) DepositionFilesDeleteAll(ctx context.Context, id string) error {
	fs, err := c.DepositionFilesList(ctx, id)
	if err != nil {
		return fmt.Errorf("list files: %w", err)
	}

	for _, f := range fs {
		if err := c.DepositionFilesDelete(ctx, id, f.ID); err != nil {
			return fmt.Errorf("delete file: %w", err)
		}
	}

	return nil
}

func (c *Client) requestjson(ctx context.Context, method, path string, data, payload interface{}) error {
	// Encode request data.
	body, err := json.Marshal(data)
	if err != nil {
		return err
	}

	// Build request.
	u := c.base + "/" + path
	req, err := http.NewRequestWithContext(ctx, method, u, bytes.NewReader(body))
	if err != nil {
		return err
	}

	req.Header.Set("Content-Type", "application/json")

	return c.request(req, payload)
}

func (c *Client) request(req *http.Request, payload interface{}) (err error) {
	// Add common headers.
	req.Header.Set("Authorization", "Bearer "+c.token)
	req.Header.Set("Accept", "application/json")

	// Execute the request.
	res, err := c.client.Do(req)
	if err != nil {
		return err
	}
	defer func() {
		if errc := res.Body.Close(); errc != nil && err == nil {
			err = errc
		}
	}()

	// Parse response body.
	if payload != nil {
		if err := decodejson(res.Body, payload); err != nil {
			return err
		}
	}

	return nil
}

func decodejson(r io.Reader, v interface{}) error {
	d := json.NewDecoder(r)

	if err := d.Decode(v); err != nil {
		return err
	}

	// Should not have trailing data.
	if d.More() {
		return errors.New("unexpected extra data after JSON")
	}

	return nil
}
