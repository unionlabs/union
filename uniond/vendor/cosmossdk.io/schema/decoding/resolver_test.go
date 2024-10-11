package decoding

import (
	"errors"
	"testing"

	"cosmossdk.io/schema"
)

type modA struct{}

func (m modA) ModuleCodec() (schema.ModuleCodec, error) {
	modSchema, err := schema.CompileModuleSchema(schema.StateObjectType{Name: "A", KeyFields: []schema.Field{{Name: "field1", Kind: schema.StringKind}}})
	if err != nil {
		return schema.ModuleCodec{}, err
	}
	return schema.ModuleCodec{
		Schema: modSchema,
	}, nil
}

type modB struct{}

func (m modB) ModuleCodec() (schema.ModuleCodec, error) {
	modSchema, err := schema.CompileModuleSchema(schema.StateObjectType{Name: "B", KeyFields: []schema.Field{{Name: "field2", Kind: schema.StringKind}}})
	if err != nil {
		return schema.ModuleCodec{}, err
	}
	return schema.ModuleCodec{
		Schema: modSchema,
	}, nil
}

type modC struct{}

var moduleSet = map[string]interface{}{
	"modA": modA{},
	"modB": modB{},
	"modC": modC{},
}

var testResolver = ModuleSetDecoderResolver(moduleSet)

func TestModuleSetDecoderResolver_IterateAll(t *testing.T) {
	objectTypes := map[string]bool{}
	err := testResolver.AllDecoders(func(moduleName string, cdc schema.ModuleCodec) error {
		cdc.Schema.AllTypes(func(t schema.Type) bool {
			objTyp, ok := t.(schema.StateObjectType)
			if ok {
				objectTypes[objTyp.Name] = true
			}
			return true
		})
		return nil
	})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if len(objectTypes) != 2 {
		t.Fatalf("expected 2 object types, got %d", len(objectTypes))
	}

	if !objectTypes["A"] {
		t.Fatalf("expected object type A")
	}

	if !objectTypes["B"] {
		t.Fatalf("expected object type B")
	}
}

func TestModuleSetDecoderResolver_LookupDecoder(t *testing.T) {
	decoder, found, err := testResolver.LookupDecoder("modA")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if !found {
		t.Fatalf("expected to find decoder for modA")
	}

	_, ok := decoder.Schema.LookupType("A")
	if !ok {
		t.Fatalf("expected object type A")
	}

	decoder, found, err = testResolver.LookupDecoder("modB")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if !found {
		t.Fatalf("expected to find decoder for modB")
	}

	_, ok = decoder.Schema.LookupType("B")
	if !ok {
		t.Fatalf("expected object type B")
	}

	decoder, found, err = testResolver.LookupDecoder("modC")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if found {
		t.Fatalf("expected not to find decoder")
	}

	decoder, found, err = testResolver.LookupDecoder("modD")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if found {
		t.Fatalf("expected not to find decoder")
	}
}

type modD struct{}

func (m modD) ModuleCodec() (schema.ModuleCodec, error) {
	return schema.ModuleCodec{}, errors.New("an error")
}

func TestModuleSetDecoderResolver_IterateAll_Error(t *testing.T) {
	resolver := ModuleSetDecoderResolver(map[string]interface{}{
		"modD": modD{},
	})
	err := resolver.AllDecoders(func(moduleName string, cdc schema.ModuleCodec) error {
		if moduleName == "modD" {
			t.Fatalf("expected error")
		}
		return nil
	})
	if err == nil {
		t.Fatalf("expected error")
	}
}
