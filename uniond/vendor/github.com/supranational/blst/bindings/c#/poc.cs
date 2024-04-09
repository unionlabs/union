using System;
using System.Text;
using supranational;

class PoC {
  private static void Main(string[] args)
  {
    var msg = Encoding.UTF8.GetBytes("assertion");
    var DST = "MY-DST";

    var SK = new blst.SecretKey();
    SK.keygen(Encoding.UTF8.GetBytes(new string('*', 32)));

    // generate public key and serialize it...
    var pk_for_wire = new blst.P1(SK).serialize();

    // sign |msg| and serialize the signature...
    var sig_for_wire = new blst.P2().hash_to(msg, DST, pk_for_wire)
                                    .sign_with(SK)
                                    .serialize();

    // now on "receiving" side, start with deserialization...
    var _sig = new blst.P2_Affine(sig_for_wire);
    var _pk = new blst.P1_Affine(pk_for_wire);
    if (!_pk.in_group())
        throw new blst.Exception(blst.ERROR.POINT_NOT_IN_GROUP);
    var ctx = new blst.Pairing(true, DST);
    var err = ctx.aggregate(_pk, _sig, msg, pk_for_wire);
    if (err != blst.ERROR.SUCCESS)
        throw new blst.Exception(err);
    ctx.commit();
    if (!ctx.finalverify())
        throw new blst.Exception(blst.ERROR.VERIFY_FAIL);
    Console.WriteLine("OK");

    // exercise .as_fp12 by performing equivalent of ctx.finalverify above
    var C1 = new blst.PT(_sig);
    var C2 = ctx.as_fp12();
    if (!blst.PT.finalverify(C1, C2))
        throw new blst.Exception(blst.ERROR.VERIFY_FAIL);

    // test integers as scalar multiplicands
    var p = blst.G1();
    var q = p.dup().dbl().dbl().add(p);
    if (!p.mult(5).is_equal(q))
       throw new ApplicationException("disaster");
    if (!blst.G1().mult(-5).is_equal(q.neg()))
       throw new ApplicationException("disaster");

    // low-order sanity check
    var p11 = new blst.P1(fromHexString("80803f0d09fec09a95f2ee7495323c15c162270c7cceaffa8566e941c66bcf206e72955d58b3b32e564de3209d672ca5"));
    if (p11.in_group())
       throw new ApplicationException("disaster");
    if (!p11.mult(11).is_inf())
       throw new ApplicationException("disaster");
  }

  private static int fromHexChar(char c)
  {
    if      (c>='0' && c<='9')  return c - '0';
    else if (c>='a' && c<='f')  return c - 'a' + 10;
    else if (c>='A' && c<='F')  return c - 'A' + 10;
    throw new ArgumentOutOfRangeException("non-hex character");
  }

  private static byte[] fromHexString(string str)
  {
    if (str.Length%2 != 0)
        throw new ArgumentException("odd number of characters in hex string");

    char[] hex = str.ToCharArray();
    byte[] ret = new byte[hex.Length/2];

    for (int i=0; i<hex.Length; i+=2)
        ret[i/2] = (byte)(fromHexChar(hex[i]) << 4 | fromHexChar(hex[i+1]));

    return ret;
  }
}
