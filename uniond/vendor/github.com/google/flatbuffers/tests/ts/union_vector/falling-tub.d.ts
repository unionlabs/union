import * as flatbuffers from 'flatbuffers';
export declare class FallingTub implements flatbuffers.IUnpackableObject<FallingTubT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): FallingTub;
    weight(): number;
    mutate_weight(value: number): boolean;
    static getFullyQualifiedName(): string;
    static sizeOf(): number;
    static createFallingTub(builder: flatbuffers.Builder, weight: number): flatbuffers.Offset;
    unpack(): FallingTubT;
    unpackTo(_o: FallingTubT): void;
}
export declare class FallingTubT implements flatbuffers.IGeneratedObject {
    weight: number;
    constructor(weight?: number);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
