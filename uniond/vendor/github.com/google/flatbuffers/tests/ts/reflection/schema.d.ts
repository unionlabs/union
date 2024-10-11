import * as flatbuffers from 'flatbuffers';
import { Enum, EnumT } from '../reflection/enum.js';
import { Object_, Object_T } from '../reflection/object.js';
import { SchemaFile, SchemaFileT } from '../reflection/schema-file.js';
import { Service, ServiceT } from '../reflection/service.js';
export declare class Schema implements flatbuffers.IUnpackableObject<SchemaT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): Schema;
    static getRootAsSchema(bb: flatbuffers.ByteBuffer, obj?: Schema): Schema;
    static getSizePrefixedRootAsSchema(bb: flatbuffers.ByteBuffer, obj?: Schema): Schema;
    static bufferHasIdentifier(bb: flatbuffers.ByteBuffer): boolean;
    objects(index: number, obj?: Object_): Object_ | null;
    objectsLength(): number;
    enums(index: number, obj?: Enum): Enum | null;
    enumsLength(): number;
    fileIdent(): string | null;
    fileIdent(optionalEncoding: flatbuffers.Encoding): string | Uint8Array | null;
    fileExt(): string | null;
    fileExt(optionalEncoding: flatbuffers.Encoding): string | Uint8Array | null;
    rootTable(obj?: Object_): Object_ | null;
    services(index: number, obj?: Service): Service | null;
    servicesLength(): number;
    advancedFeatures(): bigint;
    mutate_advanced_features(value: bigint): boolean;
    /**
     * All the files used in this compilation. Files are relative to where
     * flatc was invoked.
     */
    fbsFiles(index: number, obj?: SchemaFile): SchemaFile | null;
    fbsFilesLength(): number;
    static getFullyQualifiedName(): string;
    static startSchema(builder: flatbuffers.Builder): void;
    static addObjects(builder: flatbuffers.Builder, objectsOffset: flatbuffers.Offset): void;
    static createObjectsVector(builder: flatbuffers.Builder, data: flatbuffers.Offset[]): flatbuffers.Offset;
    static startObjectsVector(builder: flatbuffers.Builder, numElems: number): void;
    static addEnums(builder: flatbuffers.Builder, enumsOffset: flatbuffers.Offset): void;
    static createEnumsVector(builder: flatbuffers.Builder, data: flatbuffers.Offset[]): flatbuffers.Offset;
    static startEnumsVector(builder: flatbuffers.Builder, numElems: number): void;
    static addFileIdent(builder: flatbuffers.Builder, fileIdentOffset: flatbuffers.Offset): void;
    static addFileExt(builder: flatbuffers.Builder, fileExtOffset: flatbuffers.Offset): void;
    static addRootTable(builder: flatbuffers.Builder, rootTableOffset: flatbuffers.Offset): void;
    static addServices(builder: flatbuffers.Builder, servicesOffset: flatbuffers.Offset): void;
    static createServicesVector(builder: flatbuffers.Builder, data: flatbuffers.Offset[]): flatbuffers.Offset;
    static startServicesVector(builder: flatbuffers.Builder, numElems: number): void;
    static addAdvancedFeatures(builder: flatbuffers.Builder, advancedFeatures: bigint): void;
    static addFbsFiles(builder: flatbuffers.Builder, fbsFilesOffset: flatbuffers.Offset): void;
    static createFbsFilesVector(builder: flatbuffers.Builder, data: flatbuffers.Offset[]): flatbuffers.Offset;
    static startFbsFilesVector(builder: flatbuffers.Builder, numElems: number): void;
    static endSchema(builder: flatbuffers.Builder): flatbuffers.Offset;
    static finishSchemaBuffer(builder: flatbuffers.Builder, offset: flatbuffers.Offset): void;
    static finishSizePrefixedSchemaBuffer(builder: flatbuffers.Builder, offset: flatbuffers.Offset): void;
    unpack(): SchemaT;
    unpackTo(_o: SchemaT): void;
}
export declare class SchemaT implements flatbuffers.IGeneratedObject {
    objects: (Object_T)[];
    enums: (EnumT)[];
    fileIdent: string | Uint8Array | null;
    fileExt: string | Uint8Array | null;
    rootTable: Object_T | null;
    services: (ServiceT)[];
    advancedFeatures: bigint;
    fbsFiles: (SchemaFileT)[];
    constructor(objects?: (Object_T)[], enums?: (EnumT)[], fileIdent?: string | Uint8Array | null, fileExt?: string | Uint8Array | null, rootTable?: Object_T | null, services?: (ServiceT)[], advancedFeatures?: bigint, fbsFiles?: (SchemaFileT)[]);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
