import * as flatbuffers from 'flatbuffers';
import { InnerStruct, InnerStructT } from '../../my-game/example/inner-struct.js';
export declare class OuterStruct implements flatbuffers.IUnpackableObject<OuterStructT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): OuterStruct;
    a(): boolean;
    b(): number;
    cUnderscore(obj?: InnerStruct): InnerStruct | null;
    d(index: number, obj?: InnerStruct): InnerStruct | null;
    e(obj?: InnerStruct): InnerStruct | null;
    f(index: number): number | null;
    static getFullyQualifiedName(): string;
    static sizeOf(): number;
    static createOuterStruct(builder: flatbuffers.Builder, a: boolean, b: number, c_underscore_a: number, c_underscore_b: number[] | null, c_underscore_c: number, c_underscore_d_underscore: bigint, d: (any | InnerStructT)[] | null, e_a: number, e_b: number[] | null, e_c: number, e_d_underscore: bigint, f: number[] | null): flatbuffers.Offset;
    unpack(): OuterStructT;
    unpackTo(_o: OuterStructT): void;
}
export declare class OuterStructT implements flatbuffers.IGeneratedObject {
    a: boolean;
    b: number;
    cUnderscore: InnerStructT | null;
    d: (InnerStructT)[];
    e: InnerStructT | null;
    f: (number)[];
    constructor(a?: boolean, b?: number, cUnderscore?: InnerStructT | null, d?: (InnerStructT)[], e?: InnerStructT | null, f?: (number)[]);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
