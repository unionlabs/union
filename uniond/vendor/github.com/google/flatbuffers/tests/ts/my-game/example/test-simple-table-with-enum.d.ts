import * as flatbuffers from 'flatbuffers';
import { Color } from '../../my-game/example/color.js';
export declare class TestSimpleTableWithEnum implements flatbuffers.IUnpackableObject<TestSimpleTableWithEnumT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): TestSimpleTableWithEnum;
    static getRootAsTestSimpleTableWithEnum(bb: flatbuffers.ByteBuffer, obj?: TestSimpleTableWithEnum): TestSimpleTableWithEnum;
    static getSizePrefixedRootAsTestSimpleTableWithEnum(bb: flatbuffers.ByteBuffer, obj?: TestSimpleTableWithEnum): TestSimpleTableWithEnum;
    color(): Color;
    mutate_color(value: Color): boolean;
    static getFullyQualifiedName(): string;
    static startTestSimpleTableWithEnum(builder: flatbuffers.Builder): void;
    static addColor(builder: flatbuffers.Builder, color: Color): void;
    static endTestSimpleTableWithEnum(builder: flatbuffers.Builder): flatbuffers.Offset;
    static createTestSimpleTableWithEnum(builder: flatbuffers.Builder, color: Color): flatbuffers.Offset;
    serialize(): Uint8Array;
    static deserialize(buffer: Uint8Array): TestSimpleTableWithEnum;
    unpack(): TestSimpleTableWithEnumT;
    unpackTo(_o: TestSimpleTableWithEnumT): void;
}
export declare class TestSimpleTableWithEnumT implements flatbuffers.IGeneratedObject {
    color: Color;
    constructor(color?: Color);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
