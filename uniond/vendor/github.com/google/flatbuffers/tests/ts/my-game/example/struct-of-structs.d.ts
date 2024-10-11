import * as flatbuffers from 'flatbuffers';
import { Ability, AbilityT } from '../../my-game/example/ability.js';
import { Test, TestT } from '../../my-game/example/test.js';
export declare class StructOfStructs implements flatbuffers.IUnpackableObject<StructOfStructsT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): StructOfStructs;
    a(obj?: Ability): Ability | null;
    b(obj?: Test): Test | null;
    c(obj?: Ability): Ability | null;
    static getFullyQualifiedName(): string;
    static sizeOf(): number;
    static createStructOfStructs(builder: flatbuffers.Builder, a_id: number, a_distance: number, b_a: number, b_b: number, c_id: number, c_distance: number): flatbuffers.Offset;
    unpack(): StructOfStructsT;
    unpackTo(_o: StructOfStructsT): void;
}
export declare class StructOfStructsT implements flatbuffers.IGeneratedObject {
    a: AbilityT | null;
    b: TestT | null;
    c: AbilityT | null;
    constructor(a?: AbilityT | null, b?: TestT | null, c?: AbilityT | null);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
