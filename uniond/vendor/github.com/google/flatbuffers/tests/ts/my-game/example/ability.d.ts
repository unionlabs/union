import * as flatbuffers from 'flatbuffers';
export declare class Ability implements flatbuffers.IUnpackableObject<AbilityT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): Ability;
    id(): number;
    mutate_id(value: number): boolean;
    distance(): number;
    mutate_distance(value: number): boolean;
    static getFullyQualifiedName(): string;
    static sizeOf(): number;
    static createAbility(builder: flatbuffers.Builder, id: number, distance: number): flatbuffers.Offset;
    unpack(): AbilityT;
    unpackTo(_o: AbilityT): void;
}
export declare class AbilityT implements flatbuffers.IGeneratedObject {
    id: number;
    distance: number;
    constructor(id?: number, distance?: number);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
