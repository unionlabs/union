import * as flatbuffers from 'flatbuffers';
export declare class InParentNamespace implements flatbuffers.IUnpackableObject<InParentNamespaceT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): InParentNamespace;
    static getRootAsInParentNamespace(bb: flatbuffers.ByteBuffer, obj?: InParentNamespace): InParentNamespace;
    static getSizePrefixedRootAsInParentNamespace(bb: flatbuffers.ByteBuffer, obj?: InParentNamespace): InParentNamespace;
    static getFullyQualifiedName(): string;
    static startInParentNamespace(builder: flatbuffers.Builder): void;
    static endInParentNamespace(builder: flatbuffers.Builder): flatbuffers.Offset;
    static createInParentNamespace(builder: flatbuffers.Builder): flatbuffers.Offset;
    serialize(): Uint8Array;
    static deserialize(buffer: Uint8Array): InParentNamespace;
    unpack(): InParentNamespaceT;
    unpackTo(_o: InParentNamespaceT): void;
}
export declare class InParentNamespaceT implements flatbuffers.IGeneratedObject {
    constructor();
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
