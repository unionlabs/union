import * as flatbuffers from 'flatbuffers';
import { AttackerT } from './attacker.js';
import { BookReaderT } from './book-reader.js';
import { Character } from './character.js';
import { RapunzelT } from './rapunzel.js';
export declare class Movie implements flatbuffers.IUnpackableObject<MovieT> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): Movie;
    static getRootAsMovie(bb: flatbuffers.ByteBuffer, obj?: Movie): Movie;
    static getSizePrefixedRootAsMovie(bb: flatbuffers.ByteBuffer, obj?: Movie): Movie;
    static bufferHasIdentifier(bb: flatbuffers.ByteBuffer): boolean;
    mainCharacterType(): Character;
    mainCharacter<T extends flatbuffers.Table>(obj: any | string): any | string | null;
    charactersType(index: number): Character | null;
    charactersTypeLength(): number;
    charactersTypeArray(): Uint8Array | null;
    characters(index: number, obj: any | string): any | string | null;
    charactersLength(): number;
    static getFullyQualifiedName(): string;
    static startMovie(builder: flatbuffers.Builder): void;
    static addMainCharacterType(builder: flatbuffers.Builder, mainCharacterType: Character): void;
    static addMainCharacter(builder: flatbuffers.Builder, mainCharacterOffset: flatbuffers.Offset): void;
    static addCharactersType(builder: flatbuffers.Builder, charactersTypeOffset: flatbuffers.Offset): void;
    static createCharactersTypeVector(builder: flatbuffers.Builder, data: Character[]): flatbuffers.Offset;
    static startCharactersTypeVector(builder: flatbuffers.Builder, numElems: number): void;
    static addCharacters(builder: flatbuffers.Builder, charactersOffset: flatbuffers.Offset): void;
    static createCharactersVector(builder: flatbuffers.Builder, data: flatbuffers.Offset[]): flatbuffers.Offset;
    static startCharactersVector(builder: flatbuffers.Builder, numElems: number): void;
    static endMovie(builder: flatbuffers.Builder): flatbuffers.Offset;
    static finishMovieBuffer(builder: flatbuffers.Builder, offset: flatbuffers.Offset): void;
    static finishSizePrefixedMovieBuffer(builder: flatbuffers.Builder, offset: flatbuffers.Offset): void;
    static createMovie(builder: flatbuffers.Builder, mainCharacterType: Character, mainCharacterOffset: flatbuffers.Offset, charactersTypeOffset: flatbuffers.Offset, charactersOffset: flatbuffers.Offset): flatbuffers.Offset;
    unpack(): MovieT;
    unpackTo(_o: MovieT): void;
}
export declare class MovieT implements flatbuffers.IGeneratedObject {
    mainCharacterType: Character;
    mainCharacter: AttackerT | BookReaderT | RapunzelT | string | null;
    charactersType: (Character)[];
    characters: (AttackerT | BookReaderT | RapunzelT | string)[];
    constructor(mainCharacterType?: Character, mainCharacter?: AttackerT | BookReaderT | RapunzelT | string | null, charactersType?: (Character)[], characters?: (AttackerT | BookReaderT | RapunzelT | string)[]);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
