import * as flatbuffers from 'flatbuffers';
import { KeyValue, KeyValueT } from '../reflection/key-value.js';
import { Type, TypeT } from '../reflection/type.js';
export declare class EnumVal implements flatbuffers.IUnpackableObject<EnumValT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): EnumVal;
    static getRootAsEnumVal(bb: flatbuffers.ByteBuffer, obj?: EnumVal): EnumVal;
    static getSizePrefixedRootAsEnumVal(bb: flatbuffers.ByteBuffer, obj?: EnumVal): EnumVal;
    name(): string | null;
    name(optionalEncoding: flatbuffers.Encoding): string | Uint8Array | null;
    value(): bigint;
    mutate_value(value: bigint): boolean;
    unionType(obj?: Type): Type | null;
    documentation(index: number): string;
    documentation(index: number, optionalEncoding: flatbuffers.Encoding): string | Uint8Array;
    documentationLength(): number;
    attributes(index: number, obj?: KeyValue): KeyValue | null;
    attributesLength(): number;
    static getFullyQualifiedName(): string;
    static startEnumVal(builder: flatbuffers.Builder): void;
    static addName(builder: flatbuffers.Builder, nameOffset: flatbuffers.Offset): void;
    static addValue(builder: flatbuffers.Builder, value: bigint): void;
    static addUnionType(builder: flatbuffers.Builder, unionTypeOffset: flatbuffers.Offset): void;
    static addDocumentation(builder: flatbuffers.Builder, documentationOffset: flatbuffers.Offset): void;
    static createDocumentationVector(builder: flatbuffers.Builder, data: flatbuffers.Offset[]): flatbuffers.Offset;
    static startDocumentationVector(builder: flatbuffers.Builder, numElems: number): void;
    static addAttributes(builder: flatbuffers.Builder, attributesOffset: flatbuffers.Offset): void;
    static createAttributesVector(builder: flatbuffers.Builder, data: flatbuffers.Offset[]): flatbuffers.Offset;
    static startAttributesVector(builder: flatbuffers.Builder, numElems: number): void;
    static endEnumVal(builder: flatbuffers.Builder): flatbuffers.Offset;
    unpack(): EnumValT;
    unpackTo(_o: EnumValT): void;
}
export declare class EnumValT implements flatbuffers.IGeneratedObject {
    name: string | Uint8Array | null;
    value: bigint;
    unionType: TypeT | null;
    documentation: (string)[];
    attributes: (KeyValueT)[];
    constructor(name?: string | Uint8Array | null, value?: bigint, unionType?: TypeT | null, documentation?: (string)[], attributes?: (KeyValueT)[]);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
