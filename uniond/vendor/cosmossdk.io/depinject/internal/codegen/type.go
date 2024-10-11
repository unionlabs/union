package codegen

import (
	"fmt"
	"go/ast"
	"go/token"
	"reflect"
	"regexp"
	"strings"
)

// TypeExpr generates an ast.Expr to be used in the context of the file for the
// provided reflect.Type, adding any needed imports.
func (g *FileGen) TypeExpr(typ reflect.Type) (ast.Expr, error) {
	if name := typ.Name(); name != "" {
		name = g.importGenericTypeParams(name, typ.PkgPath())
		importPrefix := g.AddOrGetImport(typ.PkgPath())
		if importPrefix == "" {
			return ast.NewIdent(name), nil
		}

		return ast.NewIdent(fmt.Sprintf("%s.%s", importPrefix, name)), nil
	}

	switch typ.Kind() {

	case reflect.Array:
		elt, err := g.TypeExpr(typ.Elem())
		if err != nil {
			return nil, err
		}
		return &ast.ArrayType{
			Len: &ast.BasicLit{Kind: token.INT, Value: fmt.Sprintf("%d", typ.Len())},
			Elt: elt,
		}, nil

	case reflect.Slice:
		elt, err := g.TypeExpr(typ.Elem())
		if err != nil {
			return nil, err
		}
		return &ast.ArrayType{Elt: elt}, nil

	case reflect.Chan:
		elt, err := g.TypeExpr(typ.Elem())
		if err != nil {
			return nil, err
		}
		e := &ast.ChanType{Value: elt}
		switch typ.ChanDir() {
		case reflect.SendDir:
			e.Dir = ast.SEND
		case reflect.RecvDir:
			e.Dir = ast.RECV
		default:
			e.Dir = ast.SEND | ast.RECV
		}
		return e, nil

	case reflect.Func:
		e := &ast.FuncType{
			Params:  &ast.FieldList{},
			Results: &ast.FieldList{},
		}

		numIn := typ.NumIn()
		for i := 0; i < numIn; i++ {
			in, err := g.TypeExpr(typ.In(i))
			if err != nil {
				return nil, err
			}
			e.Params.List = append(e.Params.List, &ast.Field{Type: in})
		}

		if typ.IsVariadic() {
			in, err := g.TypeExpr(typ.In(numIn - 1).Elem())
			if err != nil {
				return nil, err
			}

			e.Params.List[numIn-1] = &ast.Field{Type: &ast.Ellipsis{Elt: in}}
		}

		for i := 0; i < typ.NumOut(); i++ {
			out, err := g.TypeExpr(typ.Out(i))
			if err != nil {
				return nil, err
			}
			e.Results.List = append(e.Results.List, &ast.Field{Type: out})
		}

		return e, nil

	case reflect.Map:
		k, err := g.TypeExpr(typ.Key())
		if err != nil {
			return nil, err
		}

		v, err := g.TypeExpr(typ.Elem())
		if err != nil {
			return nil, err
		}

		return &ast.MapType{Key: k, Value: v}, nil

	case reflect.Pointer:
		elem, err := g.TypeExpr(typ.Elem())
		if err != nil {
			return nil, err
		}

		return &ast.StarExpr{X: elem}, nil

	default:
		return nil, fmt.Errorf("unexpected type %v", typ)
	}
}

var genericTypeNameRegex = regexp.MustCompile(`(\w+)\[(.*)]`)

func (g *FileGen) importGenericTypeParams(typeName, pkgPath string) (newTypeName string) {
	// a generic type parameter from the same package the generic type is defined won't have the
	// full package name so we need to compare it with the final package part (the default import prefix)
	// ex: for a/b.C in package a/b, we'll just see the type param b.C.
	pkgParts := strings.Split(pkgPath, "/")
	pkgDefaultPrefix := pkgParts[len(pkgParts)-1]

	matches := genericTypeNameRegex.FindStringSubmatch(typeName)
	if len(matches) == 3 {
		typeParamExpr := matches[2]
		typeParams := strings.Split(typeParamExpr, ",")
		var importedTypeParams []string
		for _, param := range typeParams {
			param = strings.TrimSpace(param)
			i := strings.LastIndex(param, ".")
			if i > 0 {
				pkg := param[:i]
				name := param[i+1:]
				var prefix string
				if pkg == pkgDefaultPrefix {
					prefix = pkg
				} else {
					prefix = g.AddOrGetImport(pkg)
				}
				param = fmt.Sprintf("%s.%s", prefix, name)
			}
			importedTypeParams = append(importedTypeParams, param)
		}
		return fmt.Sprintf("%s[%s]", matches[1], strings.Join(importedTypeParams, ", "))
	}

	return typeName
}
