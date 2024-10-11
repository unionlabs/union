import * as flatbuffers from 'flatbuffers';
import { Color } from '../../my-game/example/color.js';
import { Test, TestT } from '../../my-game/example/test.js';
export declare class Vec3 implements flatbuffers.IUnpackableObject<Vec3T> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): Vec3;
    x(): number;
    mutate_x(value: number): boolean;
    y(): number;
    mutate_y(value: number): boolean;
    z(): number;
    mutate_z(value: number): boolean;
    test1(): number;
    mutate_test1(value: number): boolean;
    test2(): Color;
    mutate_test2(value: Color): boolean;
    test3(obj?: Test): Test | null;
    static getFullyQualifiedName(): string;
    static sizeOf(): number;
    static createVec3(builder: flatbuffers.Builder, x: number, y: number, z: number, test1: number, test2: Color, test3_a: number, test3_b: number): flatbuffers.Offset;
    unpack(): Vec3T;
    unpackTo(_o: Vec3T): void;
}
export declare class Vec3T implements flatbuffers.IGeneratedObject {
    x: number;
    y: number;
    z: number;
    test1: number;
    test2: Color;
    test3: TestT | null;
    constructor(x?: number, y?: number, z?: number, test1?: number, test2?: Color, test3?: TestT | null);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
