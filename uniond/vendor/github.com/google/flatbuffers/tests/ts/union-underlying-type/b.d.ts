import * as flatbuffers from 'flatbuffers';
export declare class B implements flatbuffers.IUnpackableObject<BT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): B;
    static getRootAsB(bb: flatbuffers.ByteBuffer, obj?: B): B;
    static getSizePrefixedRootAsB(bb: flatbuffers.ByteBuffer, obj?: B): B;
    b(): string | null;
    b(optionalEncoding: flatbuffers.Encoding): string | Uint8Array | null;
    static getFullyQualifiedName(): string;
    static startB(builder: flatbuffers.Builder): void;
    static addB(builder: flatbuffers.Builder, bOffset: flatbuffers.Offset): void;
    static endB(builder: flatbuffers.Builder): flatbuffers.Offset;
    static createB(builder: flatbuffers.Builder, bOffset: flatbuffers.Offset): flatbuffers.Offset;
    unpack(): BT;
    unpackTo(_o: BT): void;
}
export declare class BT implements flatbuffers.IGeneratedObject {
    b: string | Uint8Array | null;
    constructor(b?: string | Uint8Array | null);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
