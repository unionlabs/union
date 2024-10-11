/* eslint-disable @typescript-eslint/no-namespace */
import { Builder } from './flexbuffers/builder.js'
import { toReference } from './flexbuffers/reference.js'
export { toReference } from './flexbuffers/reference.js'

export function builder(): Builder {
    return new Builder();
}

export function toObject(buffer: ArrayBuffer): unknown {
    return toReference(buffer).toObject();
}

export function encode(object: unknown, size = 2048, deduplicateStrings = true, deduplicateKeys = true, deduplicateKeyVectors = true): Uint8Array {
    const builder = new Builder(size > 0 ? size : 2048, deduplicateStrings, deduplicateKeys, deduplicateKeyVectors);
    builder.add(object);
    return builder.finish();
}
