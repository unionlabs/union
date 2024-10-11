//go:build codegen
// +build codegen

package api

import (
	"bytes"
	"fmt"
	"text/template"
)

// S3ManagerUploadInputGoCode returns the Go code for the S3 Upload Manager's
// input structure.
func S3ManagerUploadInputGoCode(a *API) string {
	if v := a.PackageName(); v != "s3" {
		panic(fmt.Sprintf("unexpected API model %s", v))
	}

	s, ok := a.Shapes["PutObjectInput"]
	if !ok {
		panic(fmt.Sprintf("unable to find PutObjectInput shape in S3 model"))
	}

	a.resetImports()
	a.AddImport("io")
	a.AddImport("time")

	var w bytes.Buffer
	if err := s3managerUploadInputTmpl.Execute(&w, s); err != nil {
		panic(fmt.Sprintf("failed to execute %s template, %v",
			s3managerUploadInputTmpl.Name(), err))
	}

	return a.importsGoCode() + w.String()
}

var s3managerUploadInputTmpl = template.Must(
	template.New("s3managerUploadInputTmpl").
		Funcs(template.FuncMap{
			"GetDeprecatedMsg": getDeprecatedMessage,
			"GetDocstring": func(parent *Shape, memberName string, ref *ShapeRef) string {
				doc := ref.Docstring()
				if ref.Deprecated {
					doc = AppendDocstring(doc, fmt.Sprintf(`
					Deprecated: %s
					`, getDeprecatedMessage(ref.DeprecatedMsg, memberName)))
				}
				if parent.WillRefBeBase64Encoded(memberName) {
					doc = AppendDocstring(doc, fmt.Sprintf(`
					%s is automatically base64 encoded/decoded by the SDK.
					`, memberName))
				}
				if parent.IsRequired(memberName) {
					doc = AppendDocstring(doc, fmt.Sprintf(`
					%s is a required field
					`, memberName))
				}
				if memberName == "ContentMD5" {
					doc = AppendDocstring(doc, fmt.Sprintf(`
					If the ContentMD5 is provided for a multipart upload, it
					will be ignored. Objects that will be uploaded in a single
					part, the ContentMD5 will be used.
					`))
				}

				return doc
			},
		}).
		Parse(s3managerUploadInputTmplDef),
)

const s3managerUploadInputTmplDef = `
// UploadInput provides the input parameters for uploading a stream or buffer
// to an object in an Amazon S3 bucket. This type is similar to the s3
// package's PutObjectInput with the exception that the Body member is an
// io.Reader instead of an io.ReadSeeker.
//
// The ContentMD5 member for pre-computed MD5 checksums will be ignored for
// multipart uploads. Objects that will be uploaded in a single part, the
// ContentMD5 will be used.
//
// The Checksum members for pre-computed checksums will be ignored for
// multipart uploads. Objects that will be uploaded in a single part, will
// include the checksum member in the request.
type UploadInput struct {
	_ struct{} {{ .GoTags true false }}

	{{ range $name, $ref := $.MemberRefs -}}
		{{ if eq $name "Body" }}
			// The readable body payload to send to S3.
			Body io.Reader
		{{ else if eq $name "ContentLength" }}
			{{/* S3 Upload Manager does not use modeled content length */}}
		{{ else }}
			{{ $isRequired := $.IsRequired $name -}}
			{{ $doc := GetDocstring $ $name $ref -}}

			{{ if $doc -}}
				{{ $doc }}
			{{- end }}
			{{ $name }} {{ $.GoStructType $name $ref }} {{ $ref.GoTags false $isRequired }}
		{{ end }}
	{{ end }}
}
`
