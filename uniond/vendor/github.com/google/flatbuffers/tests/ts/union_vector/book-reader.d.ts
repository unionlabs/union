import * as flatbuffers from 'flatbuffers';
export declare class BookReader implements flatbuffers.IUnpackableObject<BookReaderT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): BookReader;
    booksRead(): number;
    mutate_books_read(value: number): boolean;
    static getFullyQualifiedName(): string;
    static sizeOf(): number;
    static createBookReader(builder: flatbuffers.Builder, books_read: number): flatbuffers.Offset;
    unpack(): BookReaderT;
    unpackTo(_o: BookReaderT): void;
}
export declare class BookReaderT implements flatbuffers.IGeneratedObject {
    booksRead: number;
    constructor(booksRead?: number);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
