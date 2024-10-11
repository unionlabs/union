import * as flatbuffers from 'flatbuffers';
export declare class Test implements flatbuffers.IUnpackableObject<TestT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): Test;
    a(): number;
    mutate_a(value: number): boolean;
    b(): number;
    mutate_b(value: number): boolean;
    static getFullyQualifiedName(): string;
    static sizeOf(): number;
    static createTest(builder: flatbuffers.Builder, a: number, b: number): flatbuffers.Offset;
    unpack(): TestT;
    unpackTo(_o: TestT): void;
}
export declare class TestT implements flatbuffers.IGeneratedObject {
    a: number;
    b: number;
    constructor(a?: number, b?: number);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
