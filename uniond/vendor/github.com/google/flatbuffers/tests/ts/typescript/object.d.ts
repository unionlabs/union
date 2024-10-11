import * as flatbuffers from 'flatbuffers';
import { Abc } from '../foobar/abc.js';
import { class_ as foobar_class_ } from '../foobar/class.js';
import { Schema, SchemaT } from '../reflection/schema.js';
import { class_ } from '../typescript/class.js';
export declare class Object_ implements flatbuffers.IUnpackableObject<Object_T> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): Object_;
    static getRootAsObject(bb: flatbuffers.ByteBuffer, obj?: Object_): Object_;
    static getSizePrefixedRootAsObject(bb: flatbuffers.ByteBuffer, obj?: Object_): Object_;
    return_(): number;
    mutate_return(value: number): boolean;
    if_(): number;
    mutate_if(value: number): boolean;
    switch_(): number;
    mutate_switch(value: number): boolean;
    enum_(): class_;
    mutate_enum(value: class_): boolean;
    enum2(): foobar_class_;
    mutate_enum2(value: foobar_class_): boolean;
    enum3(): Abc;
    mutate_enum3(value: Abc): boolean;
    reflect(obj?: Schema): Schema | null;
    static getFullyQualifiedName(): string;
    static startObject(builder: flatbuffers.Builder): void;
    static addReturn(builder: flatbuffers.Builder, return_: number): void;
    static addIf(builder: flatbuffers.Builder, if_: number): void;
    static addSwitch(builder: flatbuffers.Builder, switch_: number): void;
    static addEnum(builder: flatbuffers.Builder, enum_: class_): void;
    static addEnum2(builder: flatbuffers.Builder, enum2: foobar_class_): void;
    static addEnum3(builder: flatbuffers.Builder, enum3: Abc): void;
    static addReflect(builder: flatbuffers.Builder, reflectOffset: flatbuffers.Offset): void;
    static endObject(builder: flatbuffers.Builder): flatbuffers.Offset;
    unpack(): Object_T;
    unpackTo(_o: Object_T): void;
}
export declare class Object_T implements flatbuffers.IGeneratedObject {
    return_: number;
    if_: number;
    switch_: number;
    enum_: class_;
    enum2: foobar_class_;
    enum3: Abc;
    reflect: SchemaT | null;
    constructor(return_?: number, if_?: number, switch_?: number, enum_?: class_, enum2?: foobar_class_, enum3?: Abc, reflect?: SchemaT | null);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
