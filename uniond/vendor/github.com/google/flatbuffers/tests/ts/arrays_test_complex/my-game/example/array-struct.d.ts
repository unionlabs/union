import * as flatbuffers from 'flatbuffers';
import { NestedStruct, NestedStructT } from '../../my-game/example/nested-struct.js';
import { OuterStruct, OuterStructT } from '../../my-game/example/outer-struct.js';
export declare class ArrayStruct implements flatbuffers.IUnpackableObject<ArrayStructT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): ArrayStruct;
    aUnderscore(): number;
    bUnderscore(index: number): number | null;
    c(): number;
    d(index: number, obj?: NestedStruct): NestedStruct | null;
    e(): number;
    f(index: number, obj?: OuterStruct): OuterStruct | null;
    g(index: number): bigint | null;
    static getFullyQualifiedName(): string;
    static sizeOf(): number;
    static createArrayStruct(builder: flatbuffers.Builder, a_underscore: number, b_underscore: number[] | null, c: number, d: (any | NestedStructT)[] | null, e: number, f: (any | OuterStructT)[] | null, g: bigint[] | null): flatbuffers.Offset;
    unpack(): ArrayStructT;
    unpackTo(_o: ArrayStructT): void;
}
export declare class ArrayStructT implements flatbuffers.IGeneratedObject {
    aUnderscore: number;
    bUnderscore: (number)[];
    c: number;
    d: (NestedStructT)[];
    e: number;
    f: (OuterStructT)[];
    g: (bigint)[];
    constructor(aUnderscore?: number, bUnderscore?: (number)[], c?: number, d?: (NestedStructT)[], e?: number, f?: (OuterStructT)[], g?: (bigint)[]);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
