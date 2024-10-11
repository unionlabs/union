import * as flatbuffers from 'flatbuffers';
export declare class Stat implements flatbuffers.IUnpackableObject<StatT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): Stat;
    static getRootAsStat(bb: flatbuffers.ByteBuffer, obj?: Stat): Stat;
    static getSizePrefixedRootAsStat(bb: flatbuffers.ByteBuffer, obj?: Stat): Stat;
    id(): string | null;
    id(optionalEncoding: flatbuffers.Encoding): string | Uint8Array | null;
    val(): bigint;
    mutate_val(value: bigint): boolean;
    count(): number;
    mutate_count(value: number): boolean;
    static getFullyQualifiedName(): string;
    static startStat(builder: flatbuffers.Builder): void;
    static addId(builder: flatbuffers.Builder, idOffset: flatbuffers.Offset): void;
    static addVal(builder: flatbuffers.Builder, val: bigint): void;
    static addCount(builder: flatbuffers.Builder, count: number): void;
    static endStat(builder: flatbuffers.Builder): flatbuffers.Offset;
    static createStat(builder: flatbuffers.Builder, idOffset: flatbuffers.Offset, val: bigint, count: number): flatbuffers.Offset;
    serialize(): Uint8Array;
    static deserialize(buffer: Uint8Array): Stat;
    unpack(): StatT;
    unpackTo(_o: StatT): void;
}
export declare class StatT implements flatbuffers.IGeneratedObject {
    id: string | Uint8Array | null;
    val: bigint;
    count: number;
    constructor(id?: string | Uint8Array | null, val?: bigint, count?: number);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
