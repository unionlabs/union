import * as flatbuffers from 'flatbuffers';
import { KeyValue, KeyValueT } from '../reflection/key-value.js';
import { Type, TypeT } from '../reflection/type.js';
export declare class Field implements flatbuffers.IUnpackableObject<FieldT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): Field;
    static getRootAsField(bb: flatbuffers.ByteBuffer, obj?: Field): Field;
    static getSizePrefixedRootAsField(bb: flatbuffers.ByteBuffer, obj?: Field): Field;
    name(): string | null;
    name(optionalEncoding: flatbuffers.Encoding): string | Uint8Array | null;
    type(obj?: Type): Type | null;
    id(): number;
    mutate_id(value: number): boolean;
    offset(): number;
    mutate_offset(value: number): boolean;
    defaultInteger(): bigint;
    mutate_default_integer(value: bigint): boolean;
    defaultReal(): number;
    mutate_default_real(value: number): boolean;
    deprecated(): boolean;
    mutate_deprecated(value: boolean): boolean;
    required(): boolean;
    mutate_required(value: boolean): boolean;
    key(): boolean;
    mutate_key(value: boolean): boolean;
    attributes(index: number, obj?: KeyValue): KeyValue | null;
    attributesLength(): number;
    documentation(index: number): string;
    documentation(index: number, optionalEncoding: flatbuffers.Encoding): string | Uint8Array;
    documentationLength(): number;
    optional(): boolean;
    mutate_optional(value: boolean): boolean;
    /**
     * Number of padding octets to always add after this field. Structs only.
     */
    padding(): number;
    mutate_padding(value: number): boolean;
    /**
     * If the field uses 64-bit offsets.
     */
    offset64(): boolean;
    mutate_offset64(value: boolean): boolean;
    static getFullyQualifiedName(): string;
    static startField(builder: flatbuffers.Builder): void;
    static addName(builder: flatbuffers.Builder, nameOffset: flatbuffers.Offset): void;
    static addType(builder: flatbuffers.Builder, typeOffset: flatbuffers.Offset): void;
    static addId(builder: flatbuffers.Builder, id: number): void;
    static addOffset(builder: flatbuffers.Builder, offset: number): void;
    static addDefaultInteger(builder: flatbuffers.Builder, defaultInteger: bigint): void;
    static addDefaultReal(builder: flatbuffers.Builder, defaultReal: number): void;
    static addDeprecated(builder: flatbuffers.Builder, deprecated: boolean): void;
    static addRequired(builder: flatbuffers.Builder, required: boolean): void;
    static addKey(builder: flatbuffers.Builder, key: boolean): void;
    static addAttributes(builder: flatbuffers.Builder, attributesOffset: flatbuffers.Offset): void;
    static createAttributesVector(builder: flatbuffers.Builder, data: flatbuffers.Offset[]): flatbuffers.Offset;
    static startAttributesVector(builder: flatbuffers.Builder, numElems: number): void;
    static addDocumentation(builder: flatbuffers.Builder, documentationOffset: flatbuffers.Offset): void;
    static createDocumentationVector(builder: flatbuffers.Builder, data: flatbuffers.Offset[]): flatbuffers.Offset;
    static startDocumentationVector(builder: flatbuffers.Builder, numElems: number): void;
    static addOptional(builder: flatbuffers.Builder, optional: boolean): void;
    static addPadding(builder: flatbuffers.Builder, padding: number): void;
    static addOffset64(builder: flatbuffers.Builder, offset64: boolean): void;
    static endField(builder: flatbuffers.Builder): flatbuffers.Offset;
    unpack(): FieldT;
    unpackTo(_o: FieldT): void;
}
export declare class FieldT implements flatbuffers.IGeneratedObject {
    name: string | Uint8Array | null;
    type: TypeT | null;
    id: number;
    offset: number;
    defaultInteger: bigint;
    defaultReal: number;
    deprecated: boolean;
    required: boolean;
    key: boolean;
    attributes: (KeyValueT)[];
    documentation: (string)[];
    optional: boolean;
    padding: number;
    offset64: boolean;
    constructor(name?: string | Uint8Array | null, type?: TypeT | null, id?: number, offset?: number, defaultInteger?: bigint, defaultReal?: number, deprecated?: boolean, required?: boolean, key?: boolean, attributes?: (KeyValueT)[], documentation?: (string)[], optional?: boolean, padding?: number, offset64?: boolean);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
