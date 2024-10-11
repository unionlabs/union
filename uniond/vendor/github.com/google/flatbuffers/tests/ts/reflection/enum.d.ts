import * as flatbuffers from 'flatbuffers';
import { EnumVal, EnumValT } from '../reflection/enum-val.js';
import { KeyValue, KeyValueT } from '../reflection/key-value.js';
import { Type, TypeT } from '../reflection/type.js';
export declare class Enum implements flatbuffers.IUnpackableObject<EnumT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): Enum;
    static getRootAsEnum(bb: flatbuffers.ByteBuffer, obj?: Enum): Enum;
    static getSizePrefixedRootAsEnum(bb: flatbuffers.ByteBuffer, obj?: Enum): Enum;
    name(): string | null;
    name(optionalEncoding: flatbuffers.Encoding): string | Uint8Array | null;
    values(index: number, obj?: EnumVal): EnumVal | null;
    valuesLength(): number;
    isUnion(): boolean;
    mutate_is_union(value: boolean): boolean;
    underlyingType(obj?: Type): Type | null;
    attributes(index: number, obj?: KeyValue): KeyValue | null;
    attributesLength(): number;
    documentation(index: number): string;
    documentation(index: number, optionalEncoding: flatbuffers.Encoding): string | Uint8Array;
    documentationLength(): number;
    /**
     * File that this Enum is declared in.
     */
    declarationFile(): string | null;
    declarationFile(optionalEncoding: flatbuffers.Encoding): string | Uint8Array | null;
    static getFullyQualifiedName(): string;
    static startEnum(builder: flatbuffers.Builder): void;
    static addName(builder: flatbuffers.Builder, nameOffset: flatbuffers.Offset): void;
    static addValues(builder: flatbuffers.Builder, valuesOffset: flatbuffers.Offset): void;
    static createValuesVector(builder: flatbuffers.Builder, data: flatbuffers.Offset[]): flatbuffers.Offset;
    static startValuesVector(builder: flatbuffers.Builder, numElems: number): void;
    static addIsUnion(builder: flatbuffers.Builder, isUnion: boolean): void;
    static addUnderlyingType(builder: flatbuffers.Builder, underlyingTypeOffset: flatbuffers.Offset): void;
    static addAttributes(builder: flatbuffers.Builder, attributesOffset: flatbuffers.Offset): void;
    static createAttributesVector(builder: flatbuffers.Builder, data: flatbuffers.Offset[]): flatbuffers.Offset;
    static startAttributesVector(builder: flatbuffers.Builder, numElems: number): void;
    static addDocumentation(builder: flatbuffers.Builder, documentationOffset: flatbuffers.Offset): void;
    static createDocumentationVector(builder: flatbuffers.Builder, data: flatbuffers.Offset[]): flatbuffers.Offset;
    static startDocumentationVector(builder: flatbuffers.Builder, numElems: number): void;
    static addDeclarationFile(builder: flatbuffers.Builder, declarationFileOffset: flatbuffers.Offset): void;
    static endEnum(builder: flatbuffers.Builder): flatbuffers.Offset;
    unpack(): EnumT;
    unpackTo(_o: EnumT): void;
}
export declare class EnumT implements flatbuffers.IGeneratedObject {
    name: string | Uint8Array | null;
    values: (EnumValT)[];
    isUnion: boolean;
    underlyingType: TypeT | null;
    attributes: (KeyValueT)[];
    documentation: (string)[];
    declarationFile: string | Uint8Array | null;
    constructor(name?: string | Uint8Array | null, values?: (EnumValT)[], isUnion?: boolean, underlyingType?: TypeT | null, attributes?: (KeyValueT)[], documentation?: (string)[], declarationFile?: string | Uint8Array | null);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
