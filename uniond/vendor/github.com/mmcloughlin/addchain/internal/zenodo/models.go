package zenodo

import "time"

// Deposition represents a work-in-progress deposit.
type Deposition struct {
	ConceptDOI      string              `json:"conceptdoi,omitempty"`
	ConceptRecordID string              `json:"conceptrecid,omitempty"`
	Created         *time.Time          `json:"created,omitempty"`
	DOI             string              `json:"doi,omitempty"`
	DOIURL          string              `json:"doi_url,omitempty"`
	Files           []*DepositionFile   `json:"files,omitempty"`
	ID              int                 `json:"id,omitempty"`
	Links           map[string]string   `json:"links,omitempty"`
	Metadata        *DepositionMetadata `json:"metadata,omitempty"`
	Modified        *time.Time          `json:"modified,omitempty"`
	Owner           int                 `json:"owner,omitempty"`
	RecordID        int                 `json:"record_id,omitempty"`
	State           string              `json:"state,omitempty"`
	Submitted       bool                `json:"submitted,omitempty"`
	Title           string              `json:"title,omitempty"`
}

// DepositionFile represents a file attached to a deposit.
type DepositionFile struct {
	Checksum string            `json:"checksum"`
	Filename string            `json:"filename"`
	Filesize int               `json:"filesize"`
	ID       string            `json:"id"`
	Links    map[string]string `json:"links"`
}

// DepositionMetadata represents metadata for a deposit.
type DepositionMetadata struct {
	AccessRight        string               `json:"access_right,omitempty"`
	Communities        []*Community         `json:"communities,omitempty"`
	Creators           []*Creator           `json:"creators,omitempty"`
	Description        string               `json:"description,omitempty"`
	DOI                string               `json:"doi,omitempty"`
	License            string               `json:"license,omitempty"`
	PrereserveDOI      *PrereserveDOI       `json:"prereserve_doi,omitempty"`
	PublicationDate    string               `json:"publication_date,omitempty"`
	Title              string               `json:"title,omitempty"`
	UploadType         string               `json:"upload_type,omitempty"`
	RelatedIdentifiers []*RelatedIdentifier `json:"related_identifiers,omitempty"`
	References         []string             `json:"references,omitempty"`
	Version            string               `json:"version,omitempty"`
}

// Community associated with a deposit.
type Community struct {
	Identifier string `json:"identifier"`
}

// Creator of a deposit.
type Creator struct {
	Name        string `json:"name"`
	Affiliation string `json:"affiliation"`
	ORCID       string `json:"orcid"`
}

// PrereserveDOI represents a DOI pre-reserved for a deposit.
type PrereserveDOI struct {
	DOI      string `json:"doi"`
	RecordID int    `json:"recid"`
}

// RelatedIdentifier is a persistent identifiers of a related publications or
// dataset.
type RelatedIdentifier struct {
	Relation   string `json:"relation"`
	Identifier string `json:"identifier"`
	Scheme     string `json:"scheme"`
}
