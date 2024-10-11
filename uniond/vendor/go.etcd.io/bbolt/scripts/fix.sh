GO_CMD="go"

# TODO(ptabor): Expand to cover different architectures (GOOS GOARCH), or just list go files.

GOFILES=$(${GO_CMD} list  --f "{{with \$d:=.}}{{range .GoFiles}}{{\$d.Dir}}/{{.}}{{\"\n\"}}{{end}}{{end}}" ./...)
TESTGOFILES=$(${GO_CMD} list  --f "{{with \$d:=.}}{{range .TestGoFiles}}{{\$d.Dir}}/{{.}}{{\"\n\"}}{{end}}{{end}}" ./...)
XTESTGOFILES=$(${GO_CMD} list  --f "{{with \$d:=.}}{{range .XTestGoFiles}}{{\$d.Dir}}/{{.}}{{\"\n\"}}{{end}}{{end}}" ./...)


echo "${GOFILES}" "${TESTGOFILES}" "${XTESTGOFILES}"| xargs -n 100 go run golang.org/x/tools/cmd/goimports@latest -w -local go.etcd.io

go fmt ./...
go mod tidy
