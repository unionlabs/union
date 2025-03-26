import type { SchemaAST } from "effect"
import { Schema } from "effect"

/**
 * @see https://effect.website/docs/schema/basic-usage/#simplifying-tagged-structs-with-taggedstruct
 */
export const TaggedStruct = <
    Tag extends SchemaAST.LiteralValue,
    Fields extends Schema.Struct.Fields
>(
    tag: Tag,
    fields: Fields
) =>
    Schema.Struct({
        _tag: Schema.Literal(tag).pipe(
            Schema.optional,
            Schema.withDefaults({
                constructor: () => tag, // Apply _tag during instance construction
                decoding: () => tag // Apply _tag during decoding
            })
        ),
        ...fields
    })