import * as flatbuffers from 'flatbuffers';
import { KeyValue, KeyValueT } from '../reflection/key-value.js';
import { RPCCall, RPCCallT } from '../reflection/rpccall.js';
export declare class Service implements flatbuffers.IUnpackableObject<ServiceT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): Service;
    static getRootAsService(bb: flatbuffers.ByteBuffer, obj?: Service): Service;
    static getSizePrefixedRootAsService(bb: flatbuffers.ByteBuffer, obj?: Service): Service;
    name(): string | null;
    name(optionalEncoding: flatbuffers.Encoding): string | Uint8Array | null;
    calls(index: number, obj?: RPCCall): RPCCall | null;
    callsLength(): number;
    attributes(index: number, obj?: KeyValue): KeyValue | null;
    attributesLength(): number;
    documentation(index: number): string;
    documentation(index: number, optionalEncoding: flatbuffers.Encoding): string | Uint8Array;
    documentationLength(): number;
    /**
     * File that this Service is declared in.
     */
    declarationFile(): string | null;
    declarationFile(optionalEncoding: flatbuffers.Encoding): string | Uint8Array | null;
    static getFullyQualifiedName(): string;
    static startService(builder: flatbuffers.Builder): void;
    static addName(builder: flatbuffers.Builder, nameOffset: flatbuffers.Offset): void;
    static addCalls(builder: flatbuffers.Builder, callsOffset: flatbuffers.Offset): void;
    static createCallsVector(builder: flatbuffers.Builder, data: flatbuffers.Offset[]): flatbuffers.Offset;
    static startCallsVector(builder: flatbuffers.Builder, numElems: number): void;
    static addAttributes(builder: flatbuffers.Builder, attributesOffset: flatbuffers.Offset): void;
    static createAttributesVector(builder: flatbuffers.Builder, data: flatbuffers.Offset[]): flatbuffers.Offset;
    static startAttributesVector(builder: flatbuffers.Builder, numElems: number): void;
    static addDocumentation(builder: flatbuffers.Builder, documentationOffset: flatbuffers.Offset): void;
    static createDocumentationVector(builder: flatbuffers.Builder, data: flatbuffers.Offset[]): flatbuffers.Offset;
    static startDocumentationVector(builder: flatbuffers.Builder, numElems: number): void;
    static addDeclarationFile(builder: flatbuffers.Builder, declarationFileOffset: flatbuffers.Offset): void;
    static endService(builder: flatbuffers.Builder): flatbuffers.Offset;
    static createService(builder: flatbuffers.Builder, nameOffset: flatbuffers.Offset, callsOffset: flatbuffers.Offset, attributesOffset: flatbuffers.Offset, documentationOffset: flatbuffers.Offset, declarationFileOffset: flatbuffers.Offset): flatbuffers.Offset;
    unpack(): ServiceT;
    unpackTo(_o: ServiceT): void;
}
export declare class ServiceT implements flatbuffers.IGeneratedObject {
    name: string | Uint8Array | null;
    calls: (RPCCallT)[];
    attributes: (KeyValueT)[];
    documentation: (string)[];
    declarationFile: string | Uint8Array | null;
    constructor(name?: string | Uint8Array | null, calls?: (RPCCallT)[], attributes?: (KeyValueT)[], documentation?: (string)[], declarationFile?: string | Uint8Array | null);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
