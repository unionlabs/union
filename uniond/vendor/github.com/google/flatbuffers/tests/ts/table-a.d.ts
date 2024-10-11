import * as flatbuffers from 'flatbuffers';
import { TableB, TableBT } from './my-game/other-name-space/table-b.js';
export declare class TableA implements flatbuffers.IUnpackableObject<TableAT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): TableA;
    static getRootAsTableA(bb: flatbuffers.ByteBuffer, obj?: TableA): TableA;
    static getSizePrefixedRootAsTableA(bb: flatbuffers.ByteBuffer, obj?: TableA): TableA;
    b(obj?: TableB): TableB | null;
    static getFullyQualifiedName(): string;
    static startTableA(builder: flatbuffers.Builder): void;
    static addB(builder: flatbuffers.Builder, bOffset: flatbuffers.Offset): void;
    static endTableA(builder: flatbuffers.Builder): flatbuffers.Offset;
    static createTableA(builder: flatbuffers.Builder, bOffset: flatbuffers.Offset): flatbuffers.Offset;
    serialize(): Uint8Array;
    static deserialize(buffer: Uint8Array): TableA;
    unpack(): TableAT;
    unpackTo(_o: TableAT): void;
}
export declare class TableAT implements flatbuffers.IGeneratedObject {
    b: TableBT | null;
    constructor(b?: TableBT | null);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
