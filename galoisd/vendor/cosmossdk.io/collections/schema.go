package collections

import (
	"context"
	"fmt"
	"regexp"
	"sort"
	"strings"

	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/store"
)

// SchemaBuilder is used for building schemas. The Build method should always
// be called after all collections have been initialized. Initializing new
// collections with the builder after initialization will result in panics.
type SchemaBuilder struct {
	schema *Schema
	err    error
}

// NewSchemaBuilderFromAccessor creates a new schema builder from the provided store accessor function.
func NewSchemaBuilderFromAccessor(accessorFunc func(ctx context.Context) store.KVStore) *SchemaBuilder {
	return &SchemaBuilder{
		schema: &Schema{
			storeAccessor:       accessorFunc,
			collectionsByName:   map[string]Collection{},
			collectionsByPrefix: map[string]Collection{},
		},
	}
}

// NewSchemaBuilder creates a new schema builder from the provided store key.
// Callers should always call the SchemaBuilder.Build method when they are
// done adding collections to the schema.
func NewSchemaBuilder(service store.KVStoreService) *SchemaBuilder {
	return NewSchemaBuilderFromAccessor(service.OpenKVStore)
}

// Build should be called after all collections that are part of the schema
// have been initialized in order to get a reference to the Schema. It is
// important to check the returned error for any initialization errors.
// The SchemaBuilder CANNOT be used after Build is called - doing so will
// result in panics.
func (s *SchemaBuilder) Build() (Schema, error) {
	if s.err != nil {
		return Schema{}, s.err
	}

	// check for any overlapping prefixes
	for prefix := range s.schema.collectionsByPrefix {
		for prefix2 := range s.schema.collectionsByPrefix {
			// don't compare the prefix to itself
			if prefix == prefix2 {
				continue
			}

			// if one prefix is the prefix of the other we have an overlap and
			// this schema is corrupt
			if strings.HasPrefix(prefix, prefix2) {
				return Schema{}, fmt.Errorf("schema has overlapping prefixes 0x%x and 0x%x", prefix, prefix2)
			}
		}
	}

	// compute ordered collections
	collectionsOrdered := make([]string, 0, len(s.schema.collectionsByName))
	for name := range s.schema.collectionsByName {
		collectionsOrdered = append(collectionsOrdered, name)
	}
	sort.Strings(collectionsOrdered)
	s.schema.collectionsOrdered = collectionsOrdered

	if s.schema == nil {
		// explicit panic to avoid nil pointer dereference
		panic("builder already used to construct a schema")
	}

	schema := *s.schema

	s.schema = nil // this makes the builder unusable

	return schema, nil
}

func (s *SchemaBuilder) addCollection(collection Collection) {
	prefix := collection.GetPrefix()
	name := collection.GetName()

	if _, ok := s.schema.collectionsByPrefix[string(prefix)]; ok {
		s.appendError(fmt.Errorf("prefix %v already taken within schema", prefix))
		return
	}

	if _, ok := s.schema.collectionsByName[name]; ok {
		s.appendError(fmt.Errorf("name %s already taken within schema", name))
		return
	}

	if !nameRegex.MatchString(name) {
		s.appendError(fmt.Errorf("name must match regex %s, got %s", NameRegex, name))
		return
	}

	s.schema.collectionsByPrefix[string(prefix)] = collection
	s.schema.collectionsByName[name] = collection
}

func (s *SchemaBuilder) appendError(err error) {
	if s.err == nil {
		s.err = err
		return
	}
	s.err = fmt.Errorf("%w\n%w", s.err, err)
}

// NameRegex is the regular expression that all valid collection names must match.
const NameRegex = "[A-Za-z][A-Za-z0-9_]*"

var nameRegex = regexp.MustCompile("^" + NameRegex + "$")

// Schema specifies a group of collections stored within the storage specified
// by a single store key. All the collections within the schema must have a
// unique binary prefix and human-readable name. Schema will eventually include
// methods for importing/exporting genesis data and for schema reflection for
// clients.
type Schema struct {
	storeAccessor       func(context.Context) store.KVStore
	collectionsOrdered  []string
	collectionsByPrefix map[string]Collection
	collectionsByName   map[string]Collection
}

// NewSchema creates a new schema for the provided KVStoreService.
func NewSchema(service store.KVStoreService) Schema {
	return NewSchemaFromAccessor(func(ctx context.Context) store.KVStore {
		return service.OpenKVStore(ctx)
	})
}

// NewMemoryStoreSchema creates a new schema for the provided MemoryStoreService.
func NewMemoryStoreSchema(service store.MemoryStoreService) Schema {
	return NewSchemaFromAccessor(func(ctx context.Context) store.KVStore {
		return service.OpenMemoryStore(ctx)
	})
}

// NewSchemaFromAccessor creates a new schema for the provided store accessor
// function. Modules built against versions of the SDK which do not support
// the cosmossdk.io/core/appmodule APIs should use this method.
// Ex:

//	NewSchemaFromAccessor(func(ctx context.Context) store.KVStore {
//			return sdk.UnwrapSDKContext(ctx).KVStore(kvStoreKey)
//	}
func NewSchemaFromAccessor(accessor func(context.Context) store.KVStore) Schema {
	return Schema{
		storeAccessor:       accessor,
		collectionsByName:   map[string]Collection{},
		collectionsByPrefix: map[string]Collection{},
	}
}

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (s Schema) IsOnePerModuleType() {}

// IsAppModule implements the appmodule.AppModule interface.
func (s Schema) IsAppModule() {}

// DefaultGenesis implements the appmodule.HasGenesis.DefaultGenesis method.
func (s Schema) DefaultGenesis(target appmodule.GenesisTarget) error {
	for _, name := range s.collectionsOrdered {
		err := s.defaultGenesis(target, name)
		if err != nil {
			return fmt.Errorf("failed to instantiate default genesis for %s: %w", name, err)
		}
	}

	return nil
}

func (s Schema) defaultGenesis(target appmodule.GenesisTarget, name string) error {
	wc, err := target(name)
	if err != nil {
		return err
	}
	defer wc.Close()

	coll, err := s.getCollection(name)
	if err != nil {
		return err
	}

	return coll.defaultGenesis(wc)
}

// ValidateGenesis implements the appmodule.HasGenesis.ValidateGenesis method.
func (s Schema) ValidateGenesis(source appmodule.GenesisSource) error {
	for _, name := range s.collectionsOrdered {
		err := s.validateGenesis(source, name)
		if err != nil {
			return fmt.Errorf("failed genesis validation of %s: %w", name, err)
		}
	}
	return nil
}

func (s Schema) validateGenesis(source appmodule.GenesisSource, name string) error {
	rc, err := source(name)
	if err != nil {
		return err
	}
	defer rc.Close()

	coll, err := s.getCollection(name)
	if err != nil {
		return err
	}

	err = coll.validateGenesis(rc)
	if err != nil {
		return err
	}

	return nil
}

// InitGenesis implements the appmodule.HasGenesis.InitGenesis method.
func (s Schema) InitGenesis(ctx context.Context, source appmodule.GenesisSource) error {
	for _, name := range s.collectionsOrdered {
		err := s.initGenesis(ctx, source, name)
		if err != nil {
			return fmt.Errorf("failed genesis initialisation of %s: %w", name, err)
		}
	}

	return nil
}

func (s Schema) initGenesis(ctx context.Context, source appmodule.GenesisSource, name string) error {
	rc, err := source(name)
	if err != nil {
		return err
	}
	defer rc.Close()

	coll, err := s.getCollection(name)
	if err != nil {
		return err
	}

	err = coll.importGenesis(ctx, rc)
	if err != nil {
		return err
	}

	return nil
}

// ExportGenesis implements the appmodule.HasGenesis.ExportGenesis method.
func (s Schema) ExportGenesis(ctx context.Context, target appmodule.GenesisTarget) error {
	for _, name := range s.collectionsOrdered {
		err := s.exportGenesis(ctx, target, name)
		if err != nil {
			return fmt.Errorf("failed to export genesis for %s: %w", name, err)
		}
	}

	return nil
}

func (s Schema) exportGenesis(ctx context.Context, target appmodule.GenesisTarget, name string) error {
	wc, err := target(name)
	if err != nil {
		return err
	}
	defer wc.Close()

	coll, err := s.getCollection(name)
	if err != nil {
		return err
	}

	return coll.exportGenesis(ctx, wc)
}

func (s Schema) getCollection(name string) (Collection, error) {
	coll, ok := s.collectionsByName[name]
	if !ok {
		return nil, fmt.Errorf("unknown collection: %s", name)
	}
	return coll, nil
}

func (s Schema) ListCollections() []Collection {
	colls := make([]Collection, len(s.collectionsOrdered))
	for i, name := range s.collectionsOrdered {
		colls[i] = s.collectionsByName[name]
	}
	return colls
}
