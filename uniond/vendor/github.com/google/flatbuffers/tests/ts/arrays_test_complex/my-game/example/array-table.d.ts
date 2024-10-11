import * as flatbuffers from 'flatbuffers';
import { ArrayStruct, ArrayStructT } from '../../my-game/example/array-struct.js';
export declare class ArrayTable implements flatbuffers.IUnpackableObject<ArrayTableT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): ArrayTable;
    static getRootAsArrayTable(bb: flatbuffers.ByteBuffer, obj?: ArrayTable): ArrayTable;
    static getSizePrefixedRootAsArrayTable(bb: flatbuffers.ByteBuffer, obj?: ArrayTable): ArrayTable;
    static bufferHasIdentifier(bb: flatbuffers.ByteBuffer): boolean;
    a(): string | null;
    a(optionalEncoding: flatbuffers.Encoding): string | Uint8Array | null;
    cUnderscore(obj?: ArrayStruct): ArrayStruct | null;
    static getFullyQualifiedName(): string;
    static startArrayTable(builder: flatbuffers.Builder): void;
    static addA(builder: flatbuffers.Builder, aOffset: flatbuffers.Offset): void;
    static addCUnderscore(builder: flatbuffers.Builder, cUnderscoreOffset: flatbuffers.Offset): void;
    static endArrayTable(builder: flatbuffers.Builder): flatbuffers.Offset;
    static finishArrayTableBuffer(builder: flatbuffers.Builder, offset: flatbuffers.Offset): void;
    static finishSizePrefixedArrayTableBuffer(builder: flatbuffers.Builder, offset: flatbuffers.Offset): void;
    unpack(): ArrayTableT;
    unpackTo(_o: ArrayTableT): void;
}
export declare class ArrayTableT implements flatbuffers.IGeneratedObject {
    a: string | Uint8Array | null;
    cUnderscore: ArrayStructT | null;
    constructor(a?: string | Uint8Array | null, cUnderscore?: ArrayStructT | null);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
