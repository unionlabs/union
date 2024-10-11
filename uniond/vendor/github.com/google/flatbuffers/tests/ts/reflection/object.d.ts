import * as flatbuffers from 'flatbuffers';
import { Field, FieldT } from '../reflection/field.js';
import { KeyValue, KeyValueT } from '../reflection/key-value.js';
export declare class Object_ implements flatbuffers.IUnpackableObject<Object_T> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): Object_;
    static getRootAsObject(bb: flatbuffers.ByteBuffer, obj?: Object_): Object_;
    static getSizePrefixedRootAsObject(bb: flatbuffers.ByteBuffer, obj?: Object_): Object_;
    name(): string | null;
    name(optionalEncoding: flatbuffers.Encoding): string | Uint8Array | null;
    fields(index: number, obj?: Field): Field | null;
    fieldsLength(): number;
    isStruct(): boolean;
    mutate_is_struct(value: boolean): boolean;
    minalign(): number;
    mutate_minalign(value: number): boolean;
    bytesize(): number;
    mutate_bytesize(value: number): boolean;
    attributes(index: number, obj?: KeyValue): KeyValue | null;
    attributesLength(): number;
    documentation(index: number): string;
    documentation(index: number, optionalEncoding: flatbuffers.Encoding): string | Uint8Array;
    documentationLength(): number;
    /**
     * File that this Object is declared in.
     */
    declarationFile(): string | null;
    declarationFile(optionalEncoding: flatbuffers.Encoding): string | Uint8Array | null;
    static getFullyQualifiedName(): string;
    static startObject(builder: flatbuffers.Builder): void;
    static addName(builder: flatbuffers.Builder, nameOffset: flatbuffers.Offset): void;
    static addFields(builder: flatbuffers.Builder, fieldsOffset: flatbuffers.Offset): void;
    static createFieldsVector(builder: flatbuffers.Builder, data: flatbuffers.Offset[]): flatbuffers.Offset;
    static startFieldsVector(builder: flatbuffers.Builder, numElems: number): void;
    static addIsStruct(builder: flatbuffers.Builder, isStruct: boolean): void;
    static addMinalign(builder: flatbuffers.Builder, minalign: number): void;
    static addBytesize(builder: flatbuffers.Builder, bytesize: number): void;
    static addAttributes(builder: flatbuffers.Builder, attributesOffset: flatbuffers.Offset): void;
    static createAttributesVector(builder: flatbuffers.Builder, data: flatbuffers.Offset[]): flatbuffers.Offset;
    static startAttributesVector(builder: flatbuffers.Builder, numElems: number): void;
    static addDocumentation(builder: flatbuffers.Builder, documentationOffset: flatbuffers.Offset): void;
    static createDocumentationVector(builder: flatbuffers.Builder, data: flatbuffers.Offset[]): flatbuffers.Offset;
    static startDocumentationVector(builder: flatbuffers.Builder, numElems: number): void;
    static addDeclarationFile(builder: flatbuffers.Builder, declarationFileOffset: flatbuffers.Offset): void;
    static endObject(builder: flatbuffers.Builder): flatbuffers.Offset;
    static createObject(builder: flatbuffers.Builder, nameOffset: flatbuffers.Offset, fieldsOffset: flatbuffers.Offset, isStruct: boolean, minalign: number, bytesize: number, attributesOffset: flatbuffers.Offset, documentationOffset: flatbuffers.Offset, declarationFileOffset: flatbuffers.Offset): flatbuffers.Offset;
    unpack(): Object_T;
    unpackTo(_o: Object_T): void;
}
export declare class Object_T implements flatbuffers.IGeneratedObject {
    name: string | Uint8Array | null;
    fields: (FieldT)[];
    isStruct: boolean;
    minalign: number;
    bytesize: number;
    attributes: (KeyValueT)[];
    documentation: (string)[];
    declarationFile: string | Uint8Array | null;
    constructor(name?: string | Uint8Array | null, fields?: (FieldT)[], isStruct?: boolean, minalign?: number, bytesize?: number, attributes?: (KeyValueT)[], documentation?: (string)[], declarationFile?: string | Uint8Array | null);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
