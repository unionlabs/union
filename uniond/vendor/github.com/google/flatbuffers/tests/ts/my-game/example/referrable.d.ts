import * as flatbuffers from 'flatbuffers';
export declare class Referrable implements flatbuffers.IUnpackableObject<ReferrableT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): Referrable;
    static getRootAsReferrable(bb: flatbuffers.ByteBuffer, obj?: Referrable): Referrable;
    static getSizePrefixedRootAsReferrable(bb: flatbuffers.ByteBuffer, obj?: Referrable): Referrable;
    id(): bigint;
    mutate_id(value: bigint): boolean;
    static getFullyQualifiedName(): string;
    static startReferrable(builder: flatbuffers.Builder): void;
    static addId(builder: flatbuffers.Builder, id: bigint): void;
    static endReferrable(builder: flatbuffers.Builder): flatbuffers.Offset;
    static createReferrable(builder: flatbuffers.Builder, id: bigint): flatbuffers.Offset;
    serialize(): Uint8Array;
    static deserialize(buffer: Uint8Array): Referrable;
    unpack(): ReferrableT;
    unpackTo(_o: ReferrableT): void;
}
export declare class ReferrableT implements flatbuffers.IGeneratedObject {
    id: bigint;
    constructor(id?: bigint);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
