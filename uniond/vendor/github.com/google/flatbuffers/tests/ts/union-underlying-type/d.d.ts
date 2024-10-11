import * as flatbuffers from 'flatbuffers';
import { AT } from '../union-underlying-type/a.js';
import { ABC } from '../union-underlying-type/abc.js';
import { BT } from '../union-underlying-type/b.js';
import { CT } from '../union-underlying-type/c.js';
export declare class D implements flatbuffers.IUnpackableObject<DT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): D;
    static getRootAsD(bb: flatbuffers.ByteBuffer, obj?: D): D;
    static getSizePrefixedRootAsD(bb: flatbuffers.ByteBuffer, obj?: D): D;
    testUnionType(): ABC;
    testUnion<T extends flatbuffers.Table>(obj: any): any | null;
    testVectorOfUnionType(index: number): ABC | null;
    testVectorOfUnionTypeLength(): number;
    testVectorOfUnionTypeArray(): Int32Array | null;
    testVectorOfUnion(index: number, obj: any): any | null;
    testVectorOfUnionLength(): number;
    static getFullyQualifiedName(): string;
    static startD(builder: flatbuffers.Builder): void;
    static addTestUnionType(builder: flatbuffers.Builder, testUnionType: ABC): void;
    static addTestUnion(builder: flatbuffers.Builder, testUnionOffset: flatbuffers.Offset): void;
    static addTestVectorOfUnionType(builder: flatbuffers.Builder, testVectorOfUnionTypeOffset: flatbuffers.Offset): void;
    static createTestVectorOfUnionTypeVector(builder: flatbuffers.Builder, data: ABC[]): flatbuffers.Offset;
    static startTestVectorOfUnionTypeVector(builder: flatbuffers.Builder, numElems: number): void;
    static addTestVectorOfUnion(builder: flatbuffers.Builder, testVectorOfUnionOffset: flatbuffers.Offset): void;
    static createTestVectorOfUnionVector(builder: flatbuffers.Builder, data: flatbuffers.Offset[]): flatbuffers.Offset;
    static startTestVectorOfUnionVector(builder: flatbuffers.Builder, numElems: number): void;
    static endD(builder: flatbuffers.Builder): flatbuffers.Offset;
    static createD(builder: flatbuffers.Builder, testUnionType: ABC, testUnionOffset: flatbuffers.Offset, testVectorOfUnionTypeOffset: flatbuffers.Offset, testVectorOfUnionOffset: flatbuffers.Offset): flatbuffers.Offset;
    unpack(): DT;
    unpackTo(_o: DT): void;
}
export declare class DT implements flatbuffers.IGeneratedObject {
    testUnionType: ABC;
    testUnion: AT | BT | CT | null;
    testVectorOfUnionType: (ABC)[];
    testVectorOfUnion: (AT | BT | CT)[];
    constructor(testUnionType?: ABC, testUnion?: AT | BT | CT | null, testVectorOfUnionType?: (ABC)[], testVectorOfUnion?: (AT | BT | CT)[]);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
