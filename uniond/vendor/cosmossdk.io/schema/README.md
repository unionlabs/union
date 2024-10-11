# Logical State Schema Framework

The `cosmossdk.io/schema` base module is designed to provide a stable, **zero-dependency** base layer for specifying the **logical representation of module state schemas** and implementing **state indexing**. This is intended to be used primarily for indexing modules in external databases and providing a standard human-readable state representation for genesis import and export.

The schema defined in this library does not aim to be general purpose and cover all types of schemas, such as those used for defining transactions. For instance, this schema does not include many types of composite objects such as nested objects and arrays. Rather, the schema defined here aims to cover _state_ schemas only which are implemented as key-value pairs and usually have direct mappings to relational database tables or objects in a document store.

Also, this schema does not cover physical state layout and byte-level encoding, but simply describes a common logical format.

## `HasModuleCodec` Interface

Any module which supports logical decoding and/or encoding should implement the `HasModuleCodec` interface. This interface provides a way to get the codec for the module, which can be used to decode the module's state and/or apply logical updates.

State frameworks such as `collections` or `orm` should directly provide `ModuleCodec` implementations so that this functionality basically comes for free if a compatible framework is used. Modules that do not use one of these frameworks can choose to manually implement logical decoding and/or encoding.
