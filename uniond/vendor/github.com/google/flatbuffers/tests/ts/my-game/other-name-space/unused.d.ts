import * as flatbuffers from 'flatbuffers';
export declare class Unused implements flatbuffers.IUnpackableObject<UnusedT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): Unused;
    a(): number;
    mutate_a(value: number): boolean;
    static getFullyQualifiedName(): string;
    static sizeOf(): number;
    static createUnused(builder: flatbuffers.Builder, a: number): flatbuffers.Offset;
    unpack(): UnusedT;
    unpackTo(_o: UnusedT): void;
}
export declare class UnusedT implements flatbuffers.IGeneratedObject {
    a: number;
    constructor(a?: number);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
