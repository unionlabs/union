package graphiql

import (
	"embed"
)

//go:embed *.js *.css *.html
var Assets embed.FS
