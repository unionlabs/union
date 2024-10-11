package codegen

import (
	"fmt"
	"go/ast"
	"go/token"
	"reflect"
	"strconv"
)

// ValueExpr generates an ast.Expr to be used in the context of the file for the
// provided reflect.Value, adding any needed imports. Values with kind Chan,
// Func, Interface, Uintptr, and UnsafePointer cannot be generated and only
// pointers to structs can be generated.
func (g *FileGen) ValueExpr(value reflect.Value) (ast.Expr, error) {
	typ := value.Type()
	switch typ.Kind() {

	case reflect.Bool:
		return &ast.BasicLit{Kind: token.IDENT, Value: fmt.Sprintf("%t", value.Bool())}, nil

	case reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		return &ast.BasicLit{Kind: token.INT, Value: fmt.Sprintf("%d", value.Uint())}, nil

	case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
		return &ast.BasicLit{Kind: token.INT, Value: fmt.Sprintf("%d", value.Int())}, nil

	case reflect.Float32, reflect.Float64:
		return &ast.BasicLit{Kind: token.FLOAT, Value: strconv.FormatFloat(value.Float(), 'e', -1, 64)}, nil

	case reflect.Complex64, reflect.Complex128:
		return &ast.BasicLit{Kind: token.FLOAT, Value: strconv.FormatComplex(value.Complex(), 'e', -1, 128)}, nil

	case reflect.Array:
		return g.arraySliceExpr(value)

	case reflect.Map:
		if value.IsNil() {
			return ast.NewIdent("nil"), nil
		}

		t, err := g.TypeExpr(typ)
		if err != nil {
			return nil, err
		}

		n := value.Len()
		lit := &ast.CompositeLit{
			Type: t,
			Elts: make([]ast.Expr, n),
		}

		for i, key := range value.MapKeys() {
			k, err := g.ValueExpr(key)
			if err != nil {
				return nil, err
			}

			v, err := g.ValueExpr(value.MapIndex(key))
			if err != nil {
				return nil, err
			}

			lit.Elts[i] = &ast.KeyValueExpr{Key: k, Value: v}
		}

		return lit, nil

	case reflect.Slice:
		if value.IsNil() {
			return ast.NewIdent("nil"), nil
		}

		return g.arraySliceExpr(value)

	case reflect.String:
		return &ast.BasicLit{Kind: token.STRING, Value: fmt.Sprintf("%q", value.String())}, nil

	case reflect.Struct:
		t, err := g.TypeExpr(typ)
		if err != nil {
			return nil, err
		}

		n := typ.NumField()
		lit := &ast.CompositeLit{
			Type: t,
		}

		for i := 0; i < n; i++ {
			f := typ.Field(i)
			v := value.FieldByName(f.Name)
			if v.IsZero() {
				continue
			}

			vExpr, err := g.ValueExpr(v)
			if err != nil {
				return nil, err
			}

			lit.Elts = append(lit.Elts, &ast.KeyValueExpr{
				Key:   ast.NewIdent(f.Name),
				Value: vExpr,
			})
		}

		return lit, nil
	case reflect.Pointer:
		if value.IsNil() {
			return ast.NewIdent("nil"), nil
		}

		if typ.Elem().Kind() != reflect.Struct {
			return nil, fmt.Errorf("invalid type %s", typ)
		}

		v, err := g.ValueExpr(value.Elem())
		if err != nil {
			return nil, err
		}

		return &ast.UnaryExpr{Op: token.AND, X: v}, nil
	case reflect.Invalid, reflect.Uintptr, reflect.Chan, reflect.Func, reflect.Interface, reflect.UnsafePointer:
		return nil, fmt.Errorf("invalid type %s", typ)

	default:
		return nil, fmt.Errorf("invalid type %s", typ)
	}
}

func (g *FileGen) arraySliceExpr(value reflect.Value) (ast.Expr, error) {
	astTyp, err := g.TypeExpr(value.Type())
	if err != nil {
		return nil, err
	}

	n := value.Len()
	lit := &ast.CompositeLit{Type: astTyp, Elts: make([]ast.Expr, n)}

	for i := 0; i < n; i++ {
		lit.Elts[i], err = g.ValueExpr(value.Index(i))
		if err != nil {
			return nil, err
		}
	}

	return lit, nil
}
