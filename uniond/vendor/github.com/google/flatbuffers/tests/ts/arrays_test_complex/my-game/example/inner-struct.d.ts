import * as flatbuffers from 'flatbuffers';
export declare class InnerStruct implements flatbuffers.IUnpackableObject<InnerStructT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): InnerStruct;
    a(): number;
    b(index: number): number | null;
    c(): number;
    dUnderscore(): bigint;
    static getFullyQualifiedName(): string;
    static sizeOf(): number;
    static createInnerStruct(builder: flatbuffers.Builder, a: number, b: number[] | null, c: number, d_underscore: bigint): flatbuffers.Offset;
    unpack(): InnerStructT;
    unpackTo(_o: InnerStructT): void;
}
export declare class InnerStructT implements flatbuffers.IGeneratedObject {
    a: number;
    b: (number)[];
    c: number;
    dUnderscore: bigint;
    constructor(a?: number, b?: (number)[], c?: number, dUnderscore?: bigint);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
