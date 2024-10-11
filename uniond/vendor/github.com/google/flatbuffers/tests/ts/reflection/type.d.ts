import * as flatbuffers from 'flatbuffers';
import { BaseType } from '../reflection/base-type.js';
export declare class Type implements flatbuffers.IUnpackableObject<TypeT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): Type;
    static getRootAsType(bb: flatbuffers.ByteBuffer, obj?: Type): Type;
    static getSizePrefixedRootAsType(bb: flatbuffers.ByteBuffer, obj?: Type): Type;
    baseType(): BaseType;
    mutate_base_type(value: BaseType): boolean;
    element(): BaseType;
    mutate_element(value: BaseType): boolean;
    index(): number;
    mutate_index(value: number): boolean;
    fixedLength(): number;
    mutate_fixed_length(value: number): boolean;
    /**
     * The size (octets) of the `base_type` field.
     */
    baseSize(): number;
    mutate_base_size(value: number): boolean;
    /**
     * The size (octets) of the `element` field, if present.
     */
    elementSize(): number;
    mutate_element_size(value: number): boolean;
    static getFullyQualifiedName(): string;
    static startType(builder: flatbuffers.Builder): void;
    static addBaseType(builder: flatbuffers.Builder, baseType: BaseType): void;
    static addElement(builder: flatbuffers.Builder, element: BaseType): void;
    static addIndex(builder: flatbuffers.Builder, index: number): void;
    static addFixedLength(builder: flatbuffers.Builder, fixedLength: number): void;
    static addBaseSize(builder: flatbuffers.Builder, baseSize: number): void;
    static addElementSize(builder: flatbuffers.Builder, elementSize: number): void;
    static endType(builder: flatbuffers.Builder): flatbuffers.Offset;
    static createType(builder: flatbuffers.Builder, baseType: BaseType, element: BaseType, index: number, fixedLength: number, baseSize: number, elementSize: number): flatbuffers.Offset;
    unpack(): TypeT;
    unpackTo(_o: TypeT): void;
}
export declare class TypeT implements flatbuffers.IGeneratedObject {
    baseType: BaseType;
    element: BaseType;
    index: number;
    fixedLength: number;
    baseSize: number;
    elementSize: number;
    constructor(baseType?: BaseType, element?: BaseType, index?: number, fixedLength?: number, baseSize?: number, elementSize?: number);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
