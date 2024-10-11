import * as flatbuffers from 'flatbuffers';
import { KeyValue, KeyValueT } from '../reflection/key-value.js';
import { Object_, Object_T } from '../reflection/object.js';
export declare class RPCCall implements flatbuffers.IUnpackableObject<RPCCallT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): RPCCall;
    static getRootAsRPCCall(bb: flatbuffers.ByteBuffer, obj?: RPCCall): RPCCall;
    static getSizePrefixedRootAsRPCCall(bb: flatbuffers.ByteBuffer, obj?: RPCCall): RPCCall;
    name(): string | null;
    name(optionalEncoding: flatbuffers.Encoding): string | Uint8Array | null;
    request(obj?: Object_): Object_ | null;
    response(obj?: Object_): Object_ | null;
    attributes(index: number, obj?: KeyValue): KeyValue | null;
    attributesLength(): number;
    documentation(index: number): string;
    documentation(index: number, optionalEncoding: flatbuffers.Encoding): string | Uint8Array;
    documentationLength(): number;
    static getFullyQualifiedName(): string;
    static startRPCCall(builder: flatbuffers.Builder): void;
    static addName(builder: flatbuffers.Builder, nameOffset: flatbuffers.Offset): void;
    static addRequest(builder: flatbuffers.Builder, requestOffset: flatbuffers.Offset): void;
    static addResponse(builder: flatbuffers.Builder, responseOffset: flatbuffers.Offset): void;
    static addAttributes(builder: flatbuffers.Builder, attributesOffset: flatbuffers.Offset): void;
    static createAttributesVector(builder: flatbuffers.Builder, data: flatbuffers.Offset[]): flatbuffers.Offset;
    static startAttributesVector(builder: flatbuffers.Builder, numElems: number): void;
    static addDocumentation(builder: flatbuffers.Builder, documentationOffset: flatbuffers.Offset): void;
    static createDocumentationVector(builder: flatbuffers.Builder, data: flatbuffers.Offset[]): flatbuffers.Offset;
    static startDocumentationVector(builder: flatbuffers.Builder, numElems: number): void;
    static endRPCCall(builder: flatbuffers.Builder): flatbuffers.Offset;
    unpack(): RPCCallT;
    unpackTo(_o: RPCCallT): void;
}
export declare class RPCCallT implements flatbuffers.IGeneratedObject {
    name: string | Uint8Array | null;
    request: Object_T | null;
    response: Object_T | null;
    attributes: (KeyValueT)[];
    documentation: (string)[];
    constructor(name?: string | Uint8Array | null, request?: Object_T | null, response?: Object_T | null, attributes?: (KeyValueT)[], documentation?: (string)[]);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
