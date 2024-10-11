import * as flatbuffers from 'flatbuffers';
/**
 * File specific information.
 * Symbols declared within a file may be recovered by iterating over all
 * symbols and examining the `declaration_file` field.
 */
export declare class SchemaFile implements flatbuffers.IUnpackableObject<SchemaFileT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): SchemaFile;
    static getRootAsSchemaFile(bb: flatbuffers.ByteBuffer, obj?: SchemaFile): SchemaFile;
    static getSizePrefixedRootAsSchemaFile(bb: flatbuffers.ByteBuffer, obj?: SchemaFile): SchemaFile;
    /**
     * Filename, relative to project root.
     */
    filename(): string | null;
    filename(optionalEncoding: flatbuffers.Encoding): string | Uint8Array | null;
    /**
     * Names of included files, relative to project root.
     */
    includedFilenames(index: number): string;
    includedFilenames(index: number, optionalEncoding: flatbuffers.Encoding): string | Uint8Array;
    includedFilenamesLength(): number;
    static getFullyQualifiedName(): string;
    static startSchemaFile(builder: flatbuffers.Builder): void;
    static addFilename(builder: flatbuffers.Builder, filenameOffset: flatbuffers.Offset): void;
    static addIncludedFilenames(builder: flatbuffers.Builder, includedFilenamesOffset: flatbuffers.Offset): void;
    static createIncludedFilenamesVector(builder: flatbuffers.Builder, data: flatbuffers.Offset[]): flatbuffers.Offset;
    static startIncludedFilenamesVector(builder: flatbuffers.Builder, numElems: number): void;
    static endSchemaFile(builder: flatbuffers.Builder): flatbuffers.Offset;
    static createSchemaFile(builder: flatbuffers.Builder, filenameOffset: flatbuffers.Offset, includedFilenamesOffset: flatbuffers.Offset): flatbuffers.Offset;
    unpack(): SchemaFileT;
    unpackTo(_o: SchemaFileT): void;
}
export declare class SchemaFileT implements flatbuffers.IGeneratedObject {
    filename: string | Uint8Array | null;
    includedFilenames: (string)[];
    constructor(filename?: string | Uint8Array | null, includedFilenames?: (string)[]);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
