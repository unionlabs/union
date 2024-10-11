package model

type User struct {
	Name          string `json:"name"`
	Moderator     bool   `json:"moderator"`
	Banned        bool   `json:"banned"`
	NumMessages   int64  `json:"numMessages"`
	Version       uint64 `json:"version"`
	SchemaVersion int    `json:"schemaVersion"`
}
