"use strict";

console.log("testing...");

import { Blst } from "./blst.hpp";
const blst: Blst = require("blst");

const msg = "assertion"; // this what we're signing
const DST = "MY-DST"; // domain separation tag

const SK = new blst.SecretKey();
SK.keygen("*".repeat(32));

////////////////////////////////////////////////////////////////////////
// generate public key and signature

const pk = new blst.P1(SK);
const pk_for_wire = pk.serialize();

const sig = new blst.P2();
const sig_for_wire = sig
  .hash_to(msg, DST, pk_for_wire)
  .sign_with(SK)
  .serialize();

////////////////////////////////////////////////////////////////////////
// at this point 'pk_for_wire', 'sig_for_wire' and 'msg' are
// "sent over network," so now on "receiver" side

(function () {
  const sig = new blst.P2_Affine(sig_for_wire);
  const pk = new blst.P1_Affine(pk_for_wire);

  if (!pk.in_group()) throw "disaster"; // vet the public key

  var ctx = new blst.Pairing(true, DST);
  ctx.aggregate(pk, sig, msg, pk_for_wire);
  ctx.commit();
  if (!ctx.finalverify()) throw "disaster";

  console.log("OK");
})();
