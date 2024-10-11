import * as flatbuffers from 'flatbuffers';
export declare class HandFan implements flatbuffers.IUnpackableObject<HandFanT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): HandFan;
    static getRootAsHandFan(bb: flatbuffers.ByteBuffer, obj?: HandFan): HandFan;
    static getSizePrefixedRootAsHandFan(bb: flatbuffers.ByteBuffer, obj?: HandFan): HandFan;
    length(): number;
    mutate_length(value: number): boolean;
    static getFullyQualifiedName(): string;
    static startHandFan(builder: flatbuffers.Builder): void;
    static addLength(builder: flatbuffers.Builder, length: number): void;
    static endHandFan(builder: flatbuffers.Builder): flatbuffers.Offset;
    static createHandFan(builder: flatbuffers.Builder, length: number): flatbuffers.Offset;
    unpack(): HandFanT;
    unpackTo(_o: HandFanT): void;
}
export declare class HandFanT implements flatbuffers.IGeneratedObject {
    length: number;
    constructor(length?: number);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
