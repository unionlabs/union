import * as flatbuffers from 'flatbuffers';
import { StructOfStructs, StructOfStructsT } from '../../my-game/example/struct-of-structs.js';
export declare class StructOfStructsOfStructs implements flatbuffers.IUnpackableObject<StructOfStructsOfStructsT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): StructOfStructsOfStructs;
    a(obj?: StructOfStructs): StructOfStructs | null;
    static getFullyQualifiedName(): string;
    static sizeOf(): number;
    static createStructOfStructsOfStructs(builder: flatbuffers.Builder, a_a_id: number, a_a_distance: number, a_b_a: number, a_b_b: number, a_c_id: number, a_c_distance: number): flatbuffers.Offset;
    unpack(): StructOfStructsOfStructsT;
    unpackTo(_o: StructOfStructsOfStructsT): void;
}
export declare class StructOfStructsOfStructsT implements flatbuffers.IGeneratedObject {
    a: StructOfStructsT | null;
    constructor(a?: StructOfStructsT | null);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
