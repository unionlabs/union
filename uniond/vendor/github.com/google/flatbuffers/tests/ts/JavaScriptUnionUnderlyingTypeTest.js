import assert from 'assert'
import * as flatbuffers from 'flatbuffers'
import {UnionUnderlyingType as Test} from './union_underlying_type_test.js'

function main() {
    let a = new Test.AT();
    a.a = 1;
    let b = new Test.BT();
    b.b = "foo";
    let c = new Test.CT();
    c.c = true;
    let d = new Test.DT();
    d.testUnionType = Test.ABC.A;
    d.testUnion = a;
    d.testVectorOfUnionType = [Test.ABC.A, Test.ABC.B, Test.ABC.C];
    d.testVectorOfUnion = [a, b, c];

    let fbb = new flatbuffers.Builder();
    let offset = d.pack(fbb);
    fbb.finish(offset);

    let unpacked = Test.D.getRootAsD(fbb.dataBuffer()).unpack();
    assert.equal(JSON.stringify(unpacked), JSON.stringify(d));
}

main()