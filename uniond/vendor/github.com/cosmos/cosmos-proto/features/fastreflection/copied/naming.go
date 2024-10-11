package copied

import (
	"strings"
	"unicode/utf8"

	"google.golang.org/protobuf/compiler/protogen"
)

func fileVarName(f *protogen.File, suffix string) string {
	prefix := f.GoDescriptorIdent.GoName
	_, n := utf8.DecodeRuneInString(prefix)
	prefix = strings.ToLower(prefix[:n]) + prefix[n:]
	return prefix + "_" + suffix
}

func MessageTypesVarName(f *FileInfo) string {
	return fileVarName(f.File, "msgTypes")
}

func InitFunctionName(f *protogen.File) string {
	return fileVarName(f, "init")
}
