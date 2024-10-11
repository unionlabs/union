import * as flatbuffers from 'flatbuffers';
export declare class A implements flatbuffers.IUnpackableObject<AT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): A;
    static getRootAsA(bb: flatbuffers.ByteBuffer, obj?: A): A;
    static getSizePrefixedRootAsA(bb: flatbuffers.ByteBuffer, obj?: A): A;
    a(): number;
    mutate_a(value: number): boolean;
    static getFullyQualifiedName(): string;
    static startA(builder: flatbuffers.Builder): void;
    static addA(builder: flatbuffers.Builder, a: number): void;
    static endA(builder: flatbuffers.Builder): flatbuffers.Offset;
    static createA(builder: flatbuffers.Builder, a: number): flatbuffers.Offset;
    unpack(): AT;
    unpackTo(_o: AT): void;
}
export declare class AT implements flatbuffers.IGeneratedObject {
    a: number;
    constructor(a?: number);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
