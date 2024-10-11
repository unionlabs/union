import * as flatbuffers from 'flatbuffers';
export declare class Monster implements flatbuffers.IUnpackableObject<MonsterT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): Monster;
    static getRootAsMonster(bb: flatbuffers.ByteBuffer, obj?: Monster): Monster;
    static getSizePrefixedRootAsMonster(bb: flatbuffers.ByteBuffer, obj?: Monster): Monster;
    static getFullyQualifiedName(): string;
    static startMonster(builder: flatbuffers.Builder): void;
    static endMonster(builder: flatbuffers.Builder): flatbuffers.Offset;
    static createMonster(builder: flatbuffers.Builder): flatbuffers.Offset;
    serialize(): Uint8Array;
    static deserialize(buffer: Uint8Array): Monster;
    unpack(): MonsterT;
    unpackTo(_o: MonsterT): void;
}
export declare class MonsterT implements flatbuffers.IGeneratedObject {
    constructor();
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
