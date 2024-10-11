import * as flatbuffers from 'flatbuffers';
export declare class KeyValue implements flatbuffers.IUnpackableObject<KeyValueT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): KeyValue;
    static getRootAsKeyValue(bb: flatbuffers.ByteBuffer, obj?: KeyValue): KeyValue;
    static getSizePrefixedRootAsKeyValue(bb: flatbuffers.ByteBuffer, obj?: KeyValue): KeyValue;
    key(): string | null;
    key(optionalEncoding: flatbuffers.Encoding): string | Uint8Array | null;
    value(): string | null;
    value(optionalEncoding: flatbuffers.Encoding): string | Uint8Array | null;
    static getFullyQualifiedName(): string;
    static startKeyValue(builder: flatbuffers.Builder): void;
    static addKey(builder: flatbuffers.Builder, keyOffset: flatbuffers.Offset): void;
    static addValue(builder: flatbuffers.Builder, valueOffset: flatbuffers.Offset): void;
    static endKeyValue(builder: flatbuffers.Builder): flatbuffers.Offset;
    static createKeyValue(builder: flatbuffers.Builder, keyOffset: flatbuffers.Offset, valueOffset: flatbuffers.Offset): flatbuffers.Offset;
    unpack(): KeyValueT;
    unpackTo(_o: KeyValueT): void;
}
export declare class KeyValueT implements flatbuffers.IGeneratedObject {
    key: string | Uint8Array | null;
    value: string | Uint8Array | null;
    constructor(key?: string | Uint8Array | null, value?: string | Uint8Array | null);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
