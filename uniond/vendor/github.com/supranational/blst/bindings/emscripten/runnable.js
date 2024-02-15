'use strict';

console.log("testing...");

const blst = require("blst");

blst['onRuntimeInitialized'] = function() {
  var msg = "assertion";          // this what we're signing
  var DST = "MY-DST";             // domain separation tag

  var SK = new blst.SecretKey();
  SK.keygen("*".repeat(32));

  ////////////////////////////////////////////////////////////////////////
  // generate public key and signature

  var pk = new blst.P1(SK);
  var pk_for_wire = pk.serialize();

  var sig = new blst.P2();
  var sig_for_wire = sig.hash_to(msg, DST, pk_for_wire)
                        .sign_with(SK)
                        .serialize();

  ////////////////////////////////////////////////////////////////////////
  // at this point 'pk_for_wire', 'sig_for_wire' and 'msg' are
  // "sent over network," so now on "receiver" side

  sig = new blst.P2_Affine(sig_for_wire);
  pk  = new blst.P1_Affine(pk_for_wire);

  if (!pk.in_group()) throw "disaster";   // vet the public key

  var ctx = new blst.Pairing(true, DST);
  ctx.aggregate(pk, sig, msg, pk_for_wire);
  ctx.commit();
  if (!ctx.finalverify()) throw "disaster";

  console.log("OK");
}
