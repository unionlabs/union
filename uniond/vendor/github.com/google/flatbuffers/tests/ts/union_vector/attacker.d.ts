import * as flatbuffers from 'flatbuffers';
export declare class Attacker implements flatbuffers.IUnpackableObject<AttackerT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): Attacker;
    static getRootAsAttacker(bb: flatbuffers.ByteBuffer, obj?: Attacker): Attacker;
    static getSizePrefixedRootAsAttacker(bb: flatbuffers.ByteBuffer, obj?: Attacker): Attacker;
    swordAttackDamage(): number;
    mutate_sword_attack_damage(value: number): boolean;
    static getFullyQualifiedName(): string;
    static startAttacker(builder: flatbuffers.Builder): void;
    static addSwordAttackDamage(builder: flatbuffers.Builder, swordAttackDamage: number): void;
    static endAttacker(builder: flatbuffers.Builder): flatbuffers.Offset;
    static createAttacker(builder: flatbuffers.Builder, swordAttackDamage: number): flatbuffers.Offset;
    unpack(): AttackerT;
    unpackTo(_o: AttackerT): void;
}
export declare class AttackerT implements flatbuffers.IGeneratedObject {
    swordAttackDamage: number;
    constructor(swordAttackDamage?: number);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
