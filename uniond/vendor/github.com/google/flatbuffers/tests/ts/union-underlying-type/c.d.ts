import * as flatbuffers from 'flatbuffers';
export declare class C implements flatbuffers.IUnpackableObject<CT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): C;
    static getRootAsC(bb: flatbuffers.ByteBuffer, obj?: C): C;
    static getSizePrefixedRootAsC(bb: flatbuffers.ByteBuffer, obj?: C): C;
    c(): boolean;
    mutate_c(value: boolean): boolean;
    static getFullyQualifiedName(): string;
    static startC(builder: flatbuffers.Builder): void;
    static addC(builder: flatbuffers.Builder, c: boolean): void;
    static endC(builder: flatbuffers.Builder): flatbuffers.Offset;
    static createC(builder: flatbuffers.Builder, c: boolean): flatbuffers.Offset;
    unpack(): CT;
    unpackTo(_o: CT): void;
}
export declare class CT implements flatbuffers.IGeneratedObject {
    c: boolean;
    constructor(c?: boolean);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
