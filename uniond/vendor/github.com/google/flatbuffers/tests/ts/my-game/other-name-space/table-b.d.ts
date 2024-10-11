import * as flatbuffers from 'flatbuffers';
import { TableA, TableAT } from '../../table-a.js';
export declare class TableB implements flatbuffers.IUnpackableObject<TableBT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): TableB;
    static getRootAsTableB(bb: flatbuffers.ByteBuffer, obj?: TableB): TableB;
    static getSizePrefixedRootAsTableB(bb: flatbuffers.ByteBuffer, obj?: TableB): TableB;
    a(obj?: TableA): TableA | null;
    static getFullyQualifiedName(): string;
    static startTableB(builder: flatbuffers.Builder): void;
    static addA(builder: flatbuffers.Builder, aOffset: flatbuffers.Offset): void;
    static endTableB(builder: flatbuffers.Builder): flatbuffers.Offset;
    static createTableB(builder: flatbuffers.Builder, aOffset: flatbuffers.Offset): flatbuffers.Offset;
    serialize(): Uint8Array;
    static deserialize(buffer: Uint8Array): TableB;
    unpack(): TableBT;
    unpackTo(_o: TableBT): void;
}
export declare class TableBT implements flatbuffers.IGeneratedObject {
    a: TableAT | null;
    constructor(a?: TableAT | null);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
