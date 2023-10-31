Ops!(

"aaa" = [
    b""           , [0x37              ], X, X86_ONLY;
]
"aad" = [
    b""           , [0xD5, 0x0A        ], X, X86_ONLY;
]
"aam" = [
    b""           , [0xD4, 0x0A        ], X, X86_ONLY;
]
"aas" = [
    b""           , [0x3F              ], X, X86_ONLY;
]
"adc" = [
    b"Abib"       , [0x14              ], X;
    b"mbib"       , [0x80              ], 2, LOCK;
    b"mbrb"       , [0x10              ], X, LOCK | ENC_MR;
    b"rbib"       , [0x80              ], 2;
    b"rbrb"       , [0x10              ], X, ENC_MR;
    b"rbvb"       , [0x12              ], X;
    b"r*ib"       , [0x83              ], 2, AUTO_SIZE  | EXACT_SIZE;
    b"A*i*"       , [0x15              ], X, AUTO_SIZE;
    b"m*i*"       , [0x81              ], 2, AUTO_SIZE | LOCK;
    b"m*ib"       , [0x83              ], 2, AUTO_SIZE | LOCK;
    b"m*r*"       , [0x11              ], X, AUTO_SIZE | LOCK | ENC_MR;
    b"r*i*"       , [0x81              ], 2, AUTO_SIZE ;
    b"r*r*"       , [0x11              ], X, AUTO_SIZE | ENC_MR;
    b"r*v*"       , [0x13              ], X, AUTO_SIZE;
]
"adcx" = [
    b"rqvq"       , [0x0F, 0x38, 0xF6  ], X, WITH_REXW | PREF_66;
]
"add" = [
    b"Abib"       , [0x04              ], X;
    b"mbib"       , [0x80              ], 0, LOCK;
    b"mbrb"       , [0x00              ], X, LOCK | ENC_MR;
    b"rbib"       , [0x80              ], 0;
    b"rbrb"       , [0x00              ], X, ENC_MR;
    b"rbvb"       , [0x02              ], X;
    b"r*ib"       , [0x83              ], 0, AUTO_SIZE  | EXACT_SIZE;
    b"A*i*"       , [0x05              ], X, AUTO_SIZE;
    b"m*i*"       , [0x81              ], 0, AUTO_SIZE | LOCK;
    b"m*ib"       , [0x83              ], 0, AUTO_SIZE | LOCK;
    b"m*r*"       , [0x01              ], X, AUTO_SIZE | LOCK | ENC_MR;
    b"r*i*"       , [0x81              ], 0, AUTO_SIZE ;
    b"r*r*"       , [0x01              ], X, AUTO_SIZE | ENC_MR;
    b"r*v*"       , [0x03              ], X, AUTO_SIZE;
]
"addpd" = [
    b"yowo"       , [0x0F, 0x58        ], X, PREF_66, SSE2;
]
"addps" = [
    b"yowo"       , [0x0F, 0x58        ], X, DEFAULT, SSE;
]
"addsd" = [
    b"yomq"       , [0x0F, 0x58        ], X, PREF_F2, SSE2;
    b"yoyo"       , [0x0F, 0x58        ], X, PREF_F2, SSE2;
]
"addss" = [
    b"yomd"       , [0x0F, 0x58        ], X, PREF_F3, SSE;
    b"yoyo"       , [0x0F, 0x58        ], X, PREF_F3, SSE;
]
"addsubpd" = [
    b"yowo"       , [0x0F, 0xD0        ], X, PREF_66, SSE3;
]
"addsubps" = [
    b"yowo"       , [0x0F, 0xD0        ], X, PREF_F2, SSE3;
]
"adox" = [
    b"rqvq"       , [0x0F, 0x38, 0xF6  ], X, WITH_REXW | PREF_F3;
]
"aesdec" = [
    b"yowo"       , [0x0F, 0x38, 0xDE  ], X, PREF_66, SSE;
]
"aesdeclast" = [
    b"yowo"       , [0x0F, 0x38, 0xDF  ], X, PREF_66, SSE;
]
"aesenc" = [
    b"yowo"       , [0x0F, 0x38, 0xDC  ], X, PREF_66, SSE;
]
"aesenclast" = [
    b"yowo"       , [0x0F, 0x38, 0xDD  ], X, PREF_66, SSE;
]
"aesimc" = [
    b"yowo"       , [0x0F, 0x38, 0xDB  ], X, PREF_66, SSE;
]
"aeskeygenassist" = [
    b"yowoib"     , [0x0F, 0x3A, 0xDF  ], X, PREF_66, SSE;
]
"and" = [
    b"Abib"       , [0x24              ], X;
    b"mbib"       , [0x80              ], 4, LOCK;
    b"mbrb"       , [0x20              ], X, LOCK | ENC_MR;
    b"rbib"       , [0x80              ], 4;
    b"rbrb"       , [0x20              ], X, ENC_MR;
    b"rbvb"       , [0x22              ], X;
    b"r*ib"       , [0x83              ], 4, AUTO_SIZE  | EXACT_SIZE;
    b"A*i*"       , [0x25              ], X, AUTO_SIZE;
    b"m*i*"       , [0x81              ], 4, AUTO_SIZE | LOCK;
    b"m*ib"       , [0x83              ], 4, AUTO_SIZE | LOCK;
    b"m*r*"       , [0x21              ], X, AUTO_SIZE | LOCK | ENC_MR;
    b"r*i*"       , [0x81              ], 4, AUTO_SIZE ;
    b"r*r*"       , [0x21              ], X, AUTO_SIZE | ENC_MR;
    b"r*v*"       , [0x23              ], X, AUTO_SIZE;
]
"andn" = [
    b"r*r*v*"     , [0x02, 0xF2        ], X, VEX_OP | AUTO_REXW, BMI1;
]
"andnpd" = [
    b"yowo"       , [0x0F, 0x55        ], X, PREF_66, SSE2;
]
"andnps" = [
    b"yowo"       , [0x0F, 0x55        ], X, DEFAULT, SSE;
]
"andpd" = [
    b"yowo"       , [0x0F, 0x54        ], X, PREF_66, SSE2;
]
"andps" = [
    b"yowo"       , [0x0F, 0x54        ], X, DEFAULT, SSE;
]
"arpl" = [
    b"vwrw"       , [0x63              ], X, X86_ONLY;
]
"bextr" = [
    b"r*v*id"     , [0x10, 0x10        ], X, XOP_OP | AUTO_REXW, TBM;
    b"r*v*r*"     , [0x02, 0xF7        ], X, VEX_OP | AUTO_REXW | ENC_MR, BMI1;
]
"blcfill" = [
    b"r*v*"       , [0x09, 0x01        ], 1, XOP_OP | AUTO_REXW | ENC_VM, TBM;
]
"blci" = [
    b"r*v*"       , [0x09, 0x02        ], 6, XOP_OP | AUTO_REXW | ENC_VM, TBM;
]
"blcic" = [
    b"r*v*"       , [0x09, 0x01        ], 5, XOP_OP | AUTO_REXW | ENC_VM, TBM;
]
"blcmsk" = [
    b"r*v*"       , [0x09, 0x02        ], 1, XOP_OP | AUTO_REXW | ENC_VM, TBM;
]
"blcs" = [
    b"r*v*"       , [0x09, 0x01        ], 3, XOP_OP | AUTO_REXW | ENC_VM, TBM;
]
"blendpd" = [
    b"yomqib"     , [0x0F, 0x3A, 0x0D  ], X, PREF_66, SSE41;
    b"yoyoib"     , [0x0F, 0x3A, 0x0D  ], X, PREF_66, SSE41;
]
"blendps" = [
    b"yomqib"     , [0x0F, 0x3A, 0x0C  ], X, PREF_66, SSE41;
    b"yoyoib"     , [0x0F, 0x3A, 0x0C  ], X, PREF_66, SSE41;
]
"blendvpd" = [
    b"yomq"       , [0x0F, 0x38, 0x15  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x15  ], X, PREF_66, SSE41;
]
"blendvps" = [
    b"yomq"       , [0x0F, 0x38, 0x14  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x14  ], X, PREF_66, SSE41;
]
"blsfill" = [
    b"r*v*"       , [0x09, 0x01        ], 2, XOP_OP | AUTO_REXW | ENC_VM, TBM;
]
"blsi" = [
    b"r*v*"       , [0x02, 0xF3        ], 3, VEX_OP | AUTO_REXW | ENC_VM, BMI1;
]
"blsic" = [
    b"r*v*"       , [0x09, 0x01        ], 6, XOP_OP | AUTO_REXW | ENC_VM, TBM;
]
"blsmsk" = [
    b"r*v*"       , [0x02, 0xF3        ], 2, VEX_OP | AUTO_REXW | ENC_VM, BMI1;
]
"blsr" = [
    b"r*v*"       , [0x02, 0xF3        ], 1, VEX_OP | AUTO_REXW | ENC_VM, BMI1;
]
"bound" = [
    b"r*m!"       , [0x62              ], X, AUTO_SIZE | X86_ONLY;
]
"bndcl" = [
    b"bom!"       , [0x0F, 0x1A        ], X, PREF_F3, MPX;
    b"borq"       , [0x0F, 0x1A        ], X,  PREF_F3, MPX;
]
"bndcn" = [
    b"bom!"       , [0x0F, 0x1B        ], X, PREF_F2, MPX;
    b"borq"       , [0x0F, 0x1B        ], X,  PREF_F2, MPX;
]
"bndcu" = [
    b"bom!"       , [0x0F, 0x1A        ], X, PREF_F2, MPX;
    b"borq"       , [0x0F, 0x1A        ], X,  PREF_F2, MPX;
]
"bndldx" = [
    b"bom!"       , [0x0F, 0x1A        ], X, ENC_MIB, MPX;
]
"bndmk" = [
    b"bom!"       , [0x0F, 0x1B        ], X, ENC_MIB | PREF_F3, MPX;
]
"bndmov" = [
    b"bobo"       , [0x0F, 0x1A        ], X, PREF_66, MPX;
    b"bobo"       , [0x0F, 0x1B        ], X, ENC_MR | PREF_66, MPX;
    b"bom!"       , [0x0F, 0x1A        ], X, PREF_66, MPX;
    b"m!bo"       , [0x0F, 0x1B        ], X, ENC_MR | PREF_66, MPX;
]
"bndstx" = [
    b"m!bo"       , [0x0F, 0x1B        ], X, ENC_MR | ENC_MIB, MPX;
]
"bsf" = [
    b"r*v*"       , [0x0F, 0xBC        ], X, AUTO_SIZE;
]
"bsr" = [
    b"r*v*"       , [0x0F, 0xBD        ], X, AUTO_SIZE;
]
"bswap" = [
    b"r*"         , [0x0F, 0xC8        ], X, AUTO_REXW | SHORT_ARG;
]
"bt" = [
    b"v*ib"       , [0x0F, 0xBA        ], 4, AUTO_SIZE;
    b"v*r*"       , [0x0F, 0xA3        ], X, AUTO_SIZE | ENC_MR;
]
"btc" = [
    b"r*ib"       , [0x0F, 0xBA        ], 7, AUTO_SIZE  | EXACT_SIZE;
    b"m*ib"       , [0x0F, 0xBA        ], 7, AUTO_SIZE | LOCK;
    b"m*r*"       , [0x0F, 0xBB        ], X, AUTO_SIZE | LOCK | ENC_MR;
    b"r*r*"       , [0x0F, 0xBB        ], X, AUTO_SIZE | ENC_MR;
]
"btr" = [
    b"r*ib"       , [0x0F, 0xBA        ], 6, AUTO_SIZE  | EXACT_SIZE;
    b"m*ib"       , [0x0F, 0xBA        ], 6, AUTO_SIZE | LOCK;
    b"m*r*"       , [0x0F, 0xB3        ], X, AUTO_SIZE | LOCK | ENC_MR;
    b"r*r*"       , [0x0F, 0xB3        ], X, AUTO_SIZE | ENC_MR;
]
"bts" = [
    b"r*ib"       , [0x0F, 0xBA        ], 5, AUTO_SIZE  | EXACT_SIZE;
    b"m*ib"       , [0x0F, 0xBA        ], 5, AUTO_SIZE | LOCK;
    b"m*r*"       , [0x0F, 0xAB        ], X, AUTO_SIZE | LOCK | ENC_MR;
    b"r*r*"       , [0x0F, 0xAB        ], X, AUTO_SIZE | ENC_MR;
]
"bzhi" = [
    b"r*v*r*"     , [0x02, 0xF5        ], X, VEX_OP | AUTO_REXW | ENC_MR, BMI2;
]
"cbw" = [
    b""           , [0x98              ], X, WORD_SIZE;
]
"cdq" = [
    b""           , [0x99              ], X;
]
"cdqe" = [
    b""           , [0x98              ], X, WITH_REXW;
]
"clac" = [
    b""           , [0x0F, 0x01, 0xCA  ], X;
]
"clc" = [
    b""           , [0xF8              ], X;
]
"cld" = [
    b""           , [0xFC              ], X;
]
"clflush" = [
    b"mb"         , [0x0F, 0xAE        ], 7, DEFAULT, SSE2;
]
"clgi" = [
    b""           , [0x0F, 0x01, 0xDD  ], X, DEFAULT, VMX | AMD;
]
"cli" = [
    b""           , [0xFA              ], X;
]
"clts" = [
    b""           , [0x0F, 0x06        ], X;
]
"clzero" = [
    b""           , [0x0F, 0x01, 0xFC  ], X, DEFAULT, AMD;
]
"cmc" = [
    b""           , [0xF5              ], X;
]
"cmp" = [
    b"Abib"       , [0x3C              ], X;
    b"rbvb"       , [0x3A              ], X;
    b"vbib"       , [0x80              ], 7;
    b"vbrb"       , [0x38              ], X, ENC_MR;
    b"A*i*"       , [0x3D              ], X, AUTO_SIZE;
    b"r*v*"       , [0x3B              ], X, AUTO_SIZE;
    b"v*i*"       , [0x81              ], 7, AUTO_SIZE;
    b"v*ib"       , [0x83              ], 7, AUTO_SIZE;
    b"v*r*"       , [0x39              ], X, AUTO_SIZE | ENC_MR;
]
"cmpeqpd" = [
    b"yowo"       , [0x0F, 0xC2, 0x00  ], X, PREF_66 | IMM_OP, SSE2;
]
"cmpeqps" = [
    b"yowo"       , [0x0F, 0xC2, 0x00  ], X, IMM_OP, SSE;
]
"cmpeqsd" = [
    b"yomq"       , [0x0F, 0xC2, 0x00  ], X, PREF_F2 | IMM_OP, SSE2;
    b"yoyo"       , [0x0F, 0xC2, 0x00  ], X, PREF_F2 | IMM_OP, SSE2;
]
"cmpeqss" = [
    b"yomd"       , [0x0F, 0xC2, 0x00  ], X, PREF_F3 | IMM_OP, SSE;
    b"yoyo"       , [0x0F, 0xC2, 0x00  ], X, PREF_F3 | IMM_OP, SSE;
]
"cmplepd" = [
    b"yowo"       , [0x0F, 0xC2, 0x02  ], X, IMM_OP | PREF_66, SSE2;
]
"cmpleps" = [
    b"yowo"       , [0x0F, 0xC2, 0x02  ], X, IMM_OP, SSE;
]
"cmplesd" = [
    b"yomq"       , [0x0F, 0xC2, 0x02  ], X, PREF_F2 | IMM_OP, SSE2;
    b"yoyo"       , [0x0F, 0xC2, 0x02  ], X, PREF_F2 | IMM_OP, SSE2;
]
"cmpless" = [
    b"yomd"       , [0x0F, 0xC2, 0x02  ], X, PREF_F3 | IMM_OP, SSE;
    b"yoyo"       , [0x0F, 0xC2, 0x02  ], X, PREF_F3 | IMM_OP, SSE;
]
"cmpltpd" = [
    b"yowo"       , [0x0F, 0xC2, 0x01  ], X, IMM_OP | PREF_66, SSE2;
]
"cmpltps" = [
    b"yowo"       , [0x0F, 0xC2, 0x01  ], X, IMM_OP, SSE;
]
"cmpltsd" = [
    b"yomq"       , [0x0F, 0xC2, 0x01  ], X, PREF_F2 | IMM_OP, SSE2;
    b"yoyo"       , [0x0F, 0xC2, 0x01  ], X, PREF_F2 | IMM_OP, SSE2;
]
"cmpltss" = [
    b"yomd"       , [0x0F, 0xC2, 0x01  ], X, IMM_OP | PREF_F3, SSE;
    b"yoyo"       , [0x0F, 0xC2, 0x01  ], X, IMM_OP | PREF_F3, SSE;
]
"cmpneqpd" = [
    b"yowo"       , [0x0F, 0xC2, 0x04  ], X, PREF_66 | IMM_OP, SSE2;
]
"cmpneqps" = [
    b"yowo"       , [0x0F, 0xC2, 0x04  ], X, IMM_OP, SSE;
]
"cmpneqsd" = [
    b"yomq"       , [0x0F, 0xC2, 0x04  ], X, PREF_F2 | IMM_OP, SSE2;
    b"yoyo"       , [0x0F, 0xC2, 0x04  ], X, PREF_F2 | IMM_OP, SSE2;
]
"cmpneqss" = [
    b"yomd"       , [0x0F, 0xC2, 0x04  ], X, IMM_OP | PREF_F3, SSE;
    b"yoyo"       , [0x0F, 0xC2, 0x04  ], X, IMM_OP | PREF_F3, SSE;
]
"cmpnlepd" = [
    b"yowo"       , [0x0F, 0xC2, 0x06  ], X, IMM_OP | PREF_66, SSE2;
]
"cmpnleps" = [
    b"yowo"       , [0x0F, 0xC2, 0x06  ], X, IMM_OP, SSE;
]
"cmpnlesd" = [
    b"yomq"       , [0x0F, 0xC2, 0x06  ], X, PREF_F2 | IMM_OP, SSE2;
    b"yoyo"       , [0x0F, 0xC2, 0x06  ], X, PREF_F2 | IMM_OP, SSE2;
]
"cmpnless" = [
    b"yomd"       , [0x0F, 0xC2, 0x06  ], X, IMM_OP | PREF_F3, SSE;
    b"yoyo"       , [0x0F, 0xC2, 0x06  ], X, IMM_OP | PREF_F3, SSE;
]
"cmpnltpd" = [
    b"yowo"       , [0x0F, 0xC2, 0x05  ], X, PREF_66 | IMM_OP, SSE2;
]
"cmpnltps" = [
    b"yowo"       , [0x0F, 0xC2, 0x05  ], X, IMM_OP, SSE;
]
"cmpnltsd" = [
    b"yomq"       , [0x0F, 0xC2, 0x05  ], X, IMM_OP | PREF_F2, SSE2;
    b"yoyo"       , [0x0F, 0xC2, 0x05  ], X, IMM_OP | PREF_F2, SSE2;
]
"cmpnltss" = [
    b"yomd"       , [0x0F, 0xC2, 0x05  ], X, PREF_F3 | IMM_OP, SSE;
    b"yoyo"       , [0x0F, 0xC2, 0x05  ], X, PREF_F3 | IMM_OP, SSE;
]
"cmpordpd" = [
    b"yowo"       , [0x0F, 0xC2, 0x07  ], X, IMM_OP | PREF_66, SSE2;
]
"cmpordps" = [
    b"yowo"       , [0x0F, 0xC2, 0x07  ], X, IMM_OP, SSE;
]
"cmpordsd" = [
    b"yomq"       , [0x0F, 0xC2, 0x07  ], X, PREF_F2 | IMM_OP, SSE2;
    b"yoyo"       , [0x0F, 0xC2, 0x07  ], X, PREF_F2 | IMM_OP, SSE2;
]
"cmpordss" = [
    b"yomd"       , [0x0F, 0xC2, 0x07  ], X, IMM_OP | PREF_F3, SSE;
    b"yoyo"       , [0x0F, 0xC2, 0x07  ], X, IMM_OP | PREF_F3, SSE;
]
"cmppd" = [
    b"yowoib"     , [0x0F, 0xC2        ], X, PREF_66, SSE2;
]
"cmpps" = [
    b"yom!ib"     , [0x0F, 0xC2        ], X, DEFAULT, SSE;
    b"yoyoib"     , [0x0F, 0xC2        ], X, DEFAULT, SSE;
]
"cmpsb" = [
    b""           , [0xA6              ], X, REPE;
]
"cmpsd" = [
    b""           , [0xA7              ], X, REPE;
    b"yowoib"     , [0x0F, 0xC2        ], X, PREF_F2, SSE2;
]
"cmpsq" = [
    b""           , [0xA7              ], X, REPE | WITH_REXW;
]
"cmpss" = [
    b"yom!ib"     , [0x0F, 0xC2        ], X, PREF_F3, SSE;
    b"yoyoib"     , [0x0F, 0xC2        ], X, PREF_F3, SSE;
]
"cmpsw" = [
    b""           , [0xA7              ], X, REPE | WORD_SIZE;
]
"cmpunordpd" = [
    b"yowo"       , [0x0F, 0xC2, 0x03  ], X, PREF_66 | IMM_OP, SSE2;
]
"cmpunordps" = [
    b"yowo"       , [0x0F, 0xC2, 0x03  ], X, IMM_OP, SSE;
]
"cmpunordsd" = [
    b"yomq"       , [0x0F, 0xC2, 0x03  ], X, PREF_F2 | IMM_OP, SSE2;
    b"yoyo"       , [0x0F, 0xC2, 0x03  ], X, PREF_F2 | IMM_OP, SSE2;
]
"cmpunordss" = [
    b"yomd"       , [0x0F, 0xC2, 0x03  ], X, PREF_F3 | IMM_OP, SSE;
    b"yoyo"       , [0x0F, 0xC2, 0x03  ], X, PREF_F3 | IMM_OP, SSE;
]
"cmpxchg" = [
    b"mbrb"       , [0x0F, 0xB0        ], X, LOCK | ENC_MR;
    b"rbrb"       , [0x0F, 0xB0        ], X, ENC_MR;
    b"m*r*"       , [0x0F, 0xB1        ], X, AUTO_SIZE | LOCK | ENC_MR;
    b"r*r*"       , [0x0F, 0xB1        ], X, AUTO_SIZE | ENC_MR;
]
"cmpxchg16b" = [
    b"mo"         , [0x0F, 0xC7        ], 1, LOCK | WITH_REXW;
]
"cmpxchg8b" = [
    b"mq"         , [0x0F, 0xC7        ], 1, LOCK;
]
"comisd" = [
    b"yomq"       , [0x0F, 0x2F        ], X, PREF_66, SSE2;
    b"yoyo"       , [0x0F, 0x2F        ], X, PREF_66, SSE2;
]
"comiss" = [
    b"yomd"       , [0x0F, 0x2F        ], X, DEFAULT, SSE;
    b"yoyo"       , [0x0F, 0x2F        ], X, DEFAULT, SSE;
]
"cpu_read" = [
    b""           , [0x0F, 0x3D        ], X, DEFAULT, CYRIX;
]
"cpu_write" = [
    b""           , [0x0F, 0x3C        ], X, DEFAULT, CYRIX;
]
"cpuid" = [
    b""           , [0x0F, 0xA2        ], X;
]
"cqo" = [
    b""           , [0x99              ], X, WITH_REXW;
]
"cvtdq2pd" = [
    b"yomq"       , [0x0F, 0xE6        ], X, PREF_F3, SSE2;
    b"yoyo"       , [0x0F, 0xE6        ], X, PREF_F3, SSE2;
]
"cvtdq2ps" = [
    b"yowo"       , [0x0F, 0x5B        ], X, DEFAULT, SSE2;
]
"cvtpd2dq" = [
    b"yowo"       , [0x0F, 0xE6        ], X, PREF_F2, SSE2;
]
"cvtpd2pi" = [
    b"xqwo"       , [0x0F, 0x2D        ], X, PREF_66, SSE2;
]
"cvtpd2ps" = [
    b"yowo"       , [0x0F, 0x5A        ], X, PREF_66, SSE2;
]
"cvtpi2pd" = [
    b"youq"       , [0x0F, 0x2A        ], X, PREF_66, SSE2;
]
"cvtpi2ps" = [
    b"youq"       , [0x0F, 0x2A        ], X, DEFAULT, MMX | SSE;
]
"cvtps2dq" = [
    b"yowo"       , [0x0F, 0x5B        ], X, PREF_66, SSE2;
]
"cvtps2pd" = [
    b"yomq"       , [0x0F, 0x5A        ], X, DEFAULT, SSE2;
    b"yoyo"       , [0x0F, 0x5A        ], X, DEFAULT, SSE2;
]
"cvtps2pi" = [
    b"xqmq"       , [0x0F, 0x2D        ], X, DEFAULT, SSE | MMX;
    b"xqyo"       , [0x0F, 0x2D        ], X, DEFAULT, SSE | MMX;
]
"cvtsd2si" = [
    b"rdmq"       , [0x0F, 0x2D        ], X, PREF_F2, SSE2;
    b"rdyo"       , [0x0F, 0x2D        ], X, PREF_F2, SSE2;
    b"rqmq"       , [0x0F, 0x2D        ], X, WITH_REXW | PREF_F2, SSE2;
    b"rqyo"       , [0x0F, 0x2D        ], X, WITH_REXW | PREF_F2, SSE2;
]
"cvtsd2ss" = [
    b"yomq"       , [0x0F, 0x5A        ], X, PREF_F2, SSE2;
    b"yoyo"       , [0x0F, 0x5A        ], X, PREF_F2, SSE2;
]
"cvtsi2sd" = [
    b"yovd"       , [0x0F, 0x2A        ], X, PREF_F2, SSE2;
    b"yovq"       , [0x0F, 0x2A        ], X, WITH_REXW | PREF_F2, SSE2;
]
"cvtsi2ss" = [
    b"yovd"       , [0x0F, 0x2A        ], X, PREF_F3, SSE;
    b"yovq"       , [0x0F, 0x2A        ], X, WITH_REXW | PREF_F3, SSE;
]
"cvtss2sd" = [
    b"yomd"       , [0x0F, 0x5A        ], X, PREF_F3, SSE2;
    b"yoyo"       , [0x0F, 0x5A        ], X, PREF_F3, SSE2;
]
"cvtss2si" = [
    b"rdmd"       , [0x0F, 0x2D        ], X, PREF_F3, SSE;
    b"rdyo"       , [0x0F, 0x2D        ], X, PREF_F3, SSE;
    b"rqmd"       , [0x0F, 0x2D        ], X, WITH_REXW | PREF_F3, SSE;
    b"rqyo"       , [0x0F, 0x2D        ], X, WITH_REXW | PREF_F3, SSE;
]
"cvttpd2dq" = [
    b"yowo"       , [0x0F, 0xE6        ], X, PREF_66, SSE2;
]
"cvttpd2pi" = [
    b"xqwo"       , [0x0F, 0x2C        ], X, PREF_66, SSE2;
]
"cvttps2dq" = [
    b"yowo"       , [0x0F, 0x5B        ], X, PREF_F3, SSE2;
]
"cvttps2pi" = [
    b"xqmq"       , [0x0F, 0x2C        ], X, DEFAULT, SSE | MMX;
    b"xqyo"       , [0x0F, 0x2C        ], X, DEFAULT, SSE | MMX;
]
"cvttsd2si" = [
    b"rdmq"       , [0x0F, 0x2C        ], X, PREF_F2, SSE2;
    b"rdyo"       , [0x0F, 0x2C        ], X, PREF_F2, SSE2;
    b"rqmq"       , [0x0F, 0x2C        ], X, WITH_REXW | PREF_F2, SSE2;
    b"rqyo"       , [0x0F, 0x2C        ], X, WITH_REXW | PREF_F2, SSE2;
]
"cvttss2si" = [
    b"rdmd"       , [0x0F, 0x2C        ], X, PREF_F3, SSE;
    b"rdyo"       , [0x0F, 0x2C        ], X, PREF_F3, SSE;
    b"rqmd"       , [0x0F, 0x2C        ], X, WITH_REXW | PREF_F3, SSE;
    b"rqyo"       , [0x0F, 0x2C        ], X, WITH_REXW | PREF_F3, SSE;
]
"cwd" = [
    b""           , [0x99              ], X, WORD_SIZE;
]
"cwde" = [
    b""           , [0x98              ], X;
]
"daa" = [
    b""           , [0x27              ], X, X86_ONLY;
]
"das" = [
    b""           , [0x2F              ], X, X86_ONLY;
]
"dec" = [
    b"mb"         , [0xFE              ], 1, LOCK;
    b"rb"         , [0xFE              ], 1;
    b"m*"         , [0xFF              ], 1, AUTO_SIZE | LOCK;
    b"r*"         , [0x48              ], 0, X86_ONLY | SHORT_ARG;
    b"r*"         , [0xFF              ], 1, AUTO_SIZE ;
]
"div" = [
    b"vb"         , [0xF6              ], 6;
    b"v*"         , [0xF7              ], 6, AUTO_SIZE;
]
"divpd" = [
    b"yowo"       , [0x0F, 0x5E        ], X, PREF_66, SSE2;
]
"divps" = [
    b"yowo"       , [0x0F, 0x5E        ], X, DEFAULT, SSE;
]
"divsd" = [
    b"yomq"       , [0x0F, 0x5E        ], X, PREF_F2, SSE2;
    b"yoyo"       , [0x0F, 0x5E        ], X, PREF_F2, SSE2;
]
"divss" = [
    b"yomd"       , [0x0F, 0x5E        ], X, PREF_F3, SSE;
    b"yoyo"       , [0x0F, 0x5E        ], X, PREF_F3, SSE;
]
"dmint" = [
    b""           , [0x0F, 0x39        ], X, DEFAULT, CYRIX;
]
"dppd" = [
    b"yomqib"     , [0x0F, 0x3A, 0x41  ], X, PREF_66, SSE41;
    b"yoyoib"     , [0x0F, 0x3A, 0x41  ], X, PREF_66, SSE41;
]
"dpps" = [
    b"yomqib"     , [0x0F, 0x3A, 0x40  ], X, PREF_66, SSE41;
    b"yoyoib"     , [0x0F, 0x3A, 0x40  ], X, PREF_66, SSE41;
]
"emms" = [
    b""           , [0x0F, 0x77        ], X, DEFAULT, MMX;
]
"enter" = [
    b"iwib"       , [0xC8              ], X;
]
"extractps" = [
    b"rqyoib"     , [0x0F, 0x3A, 0x17  ], X, WITH_REXW | ENC_MR | PREF_66, SSE41;
    b"vdyoib"     , [0x0F, 0x3A, 0x17  ], X, ENC_MR | PREF_66, SSE41;
]
"extrq" = [
    b"yoibib"     , [0x0F, 0x78        ], 0, PREF_66, SSE4A | AMD;
    b"yoyo"       , [0x0F, 0x79        ], X, PREF_66, SSE4A | AMD;
]
"f2xm1" = [
    b""           , [0xD9, 0xF0        ], X, DEFAULT, FPU;
]
"fabs" = [
    b""           , [0xD9, 0xE1        ], X, DEFAULT, FPU;
]
"fadd" = [
    b""           , [0xDE, 0xC1        ], X, DEFAULT, FPU;
    b"Xpfp"       , [0xD8, 0xC0        ], X, SHORT_ARG, FPU;
    b"fp"         , [0xD8, 0xC0        ], X, SHORT_ARG, FPU;
    b"fpXp"       , [0xDC, 0xC0        ], X, SHORT_ARG, FPU;
    b"fpXp"       , [0xDC, 0xC0        ], X, SHORT_ARG, FPU;
    b"md"         , [0xD8              ], 0, EXACT_SIZE, FPU;
    b"mq"         , [0xDC              ], 0, EXACT_SIZE, FPU;
]
"faddp" = [
    b""           , [0xDE, 0xC1        ], X, DEFAULT, FPU;
    b"fp"         , [0xDE, 0xC0        ], X, SHORT_ARG, FPU;
    b"fpXp"       , [0xDE, 0xC0        ], X, SHORT_ARG, FPU;
]
"fbld" = [
    b"m!"         , [0xDF              ], 4, DEFAULT, FPU;
]
"fbstp" = [
    b"m!"         , [0xDF              ], 6, DEFAULT, FPU;
]
"fchs" = [
    b""           , [0xD9, 0xE0        ], X, DEFAULT, FPU;
]
"fclex" = [
    b""           , [0x9B, 0xDB, 0xE2  ], X, DEFAULT, FPU;
]
"fcmovb" = [
    b""           , [0xDA, 0xC1        ], X, DEFAULT, FPU;
    b"Xpfp"       , [0xDA, 0xC0        ], X, SHORT_ARG, FPU;
    b"fp"         , [0xDA, 0xC0        ], X, SHORT_ARG, FPU;
]
"fcmovbe" = [
    b""           , [0xDA, 0xD1        ], X, DEFAULT, FPU;
    b"Xpfp"       , [0xDA, 0xD0        ], X, SHORT_ARG, FPU;
    b"fp"         , [0xDA, 0xD0        ], X, SHORT_ARG, FPU;
]
"fcmove" = [
    b""           , [0xDA, 0xC9        ], X, DEFAULT, FPU;
    b"Xpfp"       , [0xDA, 0xC8        ], X, SHORT_ARG, FPU;
    b"fp"         , [0xDA, 0xC8        ], X, SHORT_ARG, FPU;
]
"fcmovnb" = [
    b""           , [0xDB, 0xC1        ], X, DEFAULT, FPU;
    b"Xpfp"       , [0xDB, 0xC0        ], X, SHORT_ARG, FPU;
    b"fp"         , [0xDB, 0xC0        ], X, SHORT_ARG, FPU;
]
"fcmovnbe" = [
    b""           , [0xDB, 0xD1        ], X, DEFAULT, FPU;
    b"Xpfp"       , [0xDB, 0xD0        ], X, SHORT_ARG, FPU;
    b"fp"         , [0xDB, 0xD0        ], X, SHORT_ARG, FPU;
]
"fcmovne" = [
    b""           , [0xDB, 0xC9        ], X, DEFAULT, FPU;
    b"Xpfp"       , [0xDB, 0xC8        ], X, SHORT_ARG, FPU;
    b"fp"         , [0xDB, 0xC8        ], X, SHORT_ARG, FPU;
]
"fcmovnu" = [
    b""           , [0xDB, 0xD9        ], X, DEFAULT, FPU;
    b"Xpfp"       , [0xDB, 0xD8        ], X, SHORT_ARG, FPU;
    b"fp"         , [0xDB, 0xD8        ], X, SHORT_ARG, FPU;
]
"fcmovu" = [
    b""           , [0xDA, 0xD9        ], X, DEFAULT, FPU;
    b"Xpfp"       , [0xDA, 0xD8        ], X, SHORT_ARG, FPU;
    b"fp"         , [0xDA, 0xD8        ], X, SHORT_ARG, FPU;
]
"fcom" = [
    b""           , [0xD8, 0xD1        ], X, DEFAULT, FPU;
    b"Xpfp"       , [0xD8, 0xD0        ], X, SHORT_ARG, FPU;
    b"fp"         , [0xD8, 0xD0        ], X, SHORT_ARG, FPU;
    b"md"         , [0xD8              ], 2, EXACT_SIZE, FPU;
    b"mq"         , [0xDC              ], 2, EXACT_SIZE, FPU;
]
"fcomi" = [
    b""           , [0xDB, 0xF1        ], X, DEFAULT, FPU;
    b"Xpfp"       , [0xDB, 0xF0        ], X, SHORT_ARG, FPU;
    b"fp"         , [0xDB, 0xF0        ], X, SHORT_ARG, FPU;
]
"fcomip" = [
    b""           , [0xDF, 0xF1        ], X, DEFAULT, FPU;
    b"Xpfp"       , [0xDF, 0xF0        ], X, SHORT_ARG, FPU;
    b"fp"         , [0xDF, 0xF0        ], X, SHORT_ARG, FPU;
]
"fcomp" = [
    b""           , [0xD8, 0xD9        ], X, DEFAULT, FPU;
    b"Xpfp"       , [0xD8, 0xD8        ], X, SHORT_ARG, FPU;
    b"fp"         , [0xD8, 0xD8        ], X, SHORT_ARG, FPU;
    b"md"         , [0xD8              ], 3, EXACT_SIZE, FPU;
    b"mq"         , [0xDC              ], 3, EXACT_SIZE, FPU;
]
"fcompp" = [
    b""           , [0xDE, 0xD9        ], X, DEFAULT, FPU;
]
"fcos" = [
    b""           , [0xD9, 0xFF        ], X, DEFAULT, FPU;
]
"fdecstp" = [
    b""           , [0xD9, 0xF6        ], X, DEFAULT, FPU;
]
"fdisi" = [
    b""           , [0x9B, 0xDB, 0xE1  ], X, DEFAULT, FPU;
]
"fdiv" = [
    b""           , [0xDE, 0xF9        ], X, DEFAULT, FPU;
    b"Xpfp"       , [0xD8, 0xF0        ], X, SHORT_ARG, FPU;
    b"fp"         , [0xD8, 0xF0        ], X, SHORT_ARG, FPU;
    b"fpXp"       , [0xDC, 0xF8        ], X, SHORT_ARG, FPU;
    b"fpXp"       , [0xDC, 0xF8        ], X, SHORT_ARG, FPU;
    b"md"         , [0xD8              ], 6, EXACT_SIZE, FPU;
    b"mq"         , [0xDC              ], 6, EXACT_SIZE, FPU;
]
"fdivp" = [
    b""           , [0xDE, 0xF9        ], X, DEFAULT, FPU;
    b"fp"         , [0xDE, 0xF8        ], X, SHORT_ARG, FPU;
    b"fpXp"       , [0xDE, 0xF8        ], X, SHORT_ARG, FPU;
]
"fdivr" = [
    b""           , [0xDE, 0xF1        ], X, DEFAULT, FPU;
    b"Xpfp"       , [0xD8, 0xF8        ], X, SHORT_ARG, FPU;
    b"fp"         , [0xD8, 0xF8        ], X, SHORT_ARG, FPU;
    b"fpXp"       , [0xDC, 0xF0        ], X, SHORT_ARG, FPU;
    b"fpXp"       , [0xDC, 0xF0        ], X, SHORT_ARG, FPU;
    b"md"         , [0xD8              ], 7, EXACT_SIZE, FPU;
    b"mq"         , [0xDC              ], 7, EXACT_SIZE, FPU;
]
"fdivrp" = [
    b""           , [0xDE, 0xF1        ], X, DEFAULT, FPU;
    b"fp"         , [0xDE, 0xF0        ], X, SHORT_ARG, FPU;
    b"fpXp"       , [0xDE, 0xF0        ], X, SHORT_ARG, FPU;
]
"femms" = [
    b""           , [0x0F, 0x0E        ], X, DEFAULT, TDNOW;
]
"feni" = [
    b""           , [0x9B, 0xDB, 0xE0  ], X, DEFAULT, FPU;
]
"ffree" = [
    b""           , [0xDD, 0xC1        ], X, DEFAULT, FPU;
    b"fp"         , [0xDD, 0xC0        ], X, SHORT_ARG, FPU;
]
"fiadd" = [
    b"md"         , [0xDA              ], 0, EXACT_SIZE, FPU;
    b"mw"         , [0xDE              ], 0, DEFAULT, FPU;
]
"ficom" = [
    b"md"         , [0xDA              ], 2, EXACT_SIZE, FPU;
    b"mw"         , [0xDE              ], 2, DEFAULT, FPU;
]
"ficomp" = [
    b"md"         , [0xDA              ], 3, EXACT_SIZE, FPU;
    b"mw"         , [0xDE              ], 3, DEFAULT, FPU;
]
"fidiv" = [
    b"md"         , [0xDA              ], 6, EXACT_SIZE, FPU;
    b"mw"         , [0xDE              ], 6, DEFAULT, FPU;
]
"fidivr" = [
    b"md"         , [0xDA              ], 7, EXACT_SIZE, FPU;
    b"mw"         , [0xDE              ], 7, DEFAULT, FPU;
]
"fild" = [
    b"md"         , [0xDB              ], 0, EXACT_SIZE, FPU;
    b"mq"         , [0xDF              ], 5, EXACT_SIZE, FPU;
    b"mw"         , [0xDF              ], 0, DEFAULT, FPU;
]
"fimul" = [
    b"md"         , [0xDA              ], 1, EXACT_SIZE, FPU;
    b"mw"         , [0xDE              ], 1, DEFAULT, FPU;
]
"fincstp" = [
    b""           , [0xD9, 0xF7        ], X, DEFAULT, FPU;
]
"finit" = [
    b""           , [0x9B, 0xDB, 0xE3  ], X, DEFAULT, FPU;
]
"fist" = [
    b"md"         , [0xDB              ], 2, EXACT_SIZE, FPU;
    b"mw"         , [0xDF              ], 2, DEFAULT, FPU;
]
"fistp" = [
    b"md"         , [0xDB              ], 3, EXACT_SIZE, FPU;
    b"mq"         , [0xDF              ], 7, EXACT_SIZE, FPU;
    b"mw"         , [0xDF              ], 3, DEFAULT, FPU;
]
"fisttp" = [
    b"md"         , [0xDB              ], 1, EXACT_SIZE, FPU;
    b"mq"         , [0xDD              ], 1, EXACT_SIZE, FPU;
    b"mw"         , [0xDF              ], 1, DEFAULT, FPU;
]
"fisub" = [
    b"md"         , [0xDA              ], 4, EXACT_SIZE, FPU;
    b"mw"         , [0xDE              ], 4, DEFAULT, FPU;
]
"fisubr" = [
    b"md"         , [0xDA              ], 5, EXACT_SIZE, FPU;
    b"mw"         , [0xDE              ], 5, DEFAULT, FPU;
]
"fld" = [
    b""           , [0xD9, 0xC1        ], X, DEFAULT, FPU;
    b"fp"         , [0xD9, 0xC0        ], X, SHORT_ARG, FPU;
    b"md"         , [0xD9              ], 0, EXACT_SIZE, FPU;
    b"mp"         , [0xDB              ], 5, EXACT_SIZE, FPU;
    b"mq"         , [0xDD              ], 0, EXACT_SIZE, FPU;
]
"fld1" = [
    b""           , [0xD9, 0xE8        ], X, DEFAULT, FPU;
]
"fldcw" = [
    b"mw"         , [0xD9              ], 5, DEFAULT, FPU;
]
"fldenv" = [
    b"m!"         , [0xD9              ], 4, DEFAULT, FPU;
]
"fldl2e" = [
    b""           , [0xD9, 0xEA        ], X, DEFAULT, FPU;
]
"fldl2t" = [
    b""           , [0xD9, 0xE9        ], X, DEFAULT, FPU;
]
"fldlg2" = [
    b""           , [0xD9, 0xEC        ], X, DEFAULT, FPU;
]
"fldln2" = [
    b""           , [0xD9, 0xED        ], X, DEFAULT, FPU;
]
"fldpi" = [
    b""           , [0xD9, 0xEB        ], X, DEFAULT, FPU;
]
"fldz" = [
    b""           , [0xD9, 0xEE        ], X, DEFAULT, FPU;
]
"fmul" = [
    b""           , [0xDE, 0xC9        ], X, DEFAULT, FPU;
    b"Xpfp"       , [0xD8, 0xC8        ], X, SHORT_ARG, FPU;
    b"fp"         , [0xD8, 0xC8        ], X, SHORT_ARG, FPU;
    b"fpXp"       , [0xDC, 0xC8        ], X, SHORT_ARG, FPU;
    b"fpXp"       , [0xDC, 0xC8        ], X, SHORT_ARG, FPU;
    b"md"         , [0xD8              ], 1, EXACT_SIZE, FPU;
    b"mq"         , [0xDC              ], 1, EXACT_SIZE, FPU;
]
"fmulp" = [
    b""           , [0xDE, 0xC9        ], X, DEFAULT, FPU;
    b"fp"         , [0xDE, 0xC8        ], X, SHORT_ARG, FPU;
    b"fpXp"       , [0xDE, 0xC8        ], X, SHORT_ARG, FPU;
]
"fnclex" = [
    b""           , [0xDB, 0xE2        ], X, DEFAULT, FPU;
]
"fndisi" = [
    b""           , [0xDB, 0xE1        ], X, DEFAULT, FPU;
]
"fneni" = [
    b""           , [0xDB, 0xE0        ], X, DEFAULT, FPU;
]
"fninit" = [
    b""           , [0xDB, 0xE3        ], X, DEFAULT, FPU;
]
"fnop" = [
    b""           , [0xD9, 0xD0        ], X, DEFAULT, FPU;
]
"fnsave" = [
    b"m!"         , [0xDD              ], 6, DEFAULT, FPU;
]
"fnstcw" = [
    b"mw"         , [0xD9              ], 7, DEFAULT, FPU;
]
"fnstenv" = [
    b"m!"         , [0xD9              ], 6, DEFAULT, FPU;
]
"fnstsw" = [
    b"Aw"         , [0xDF, 0xE0        ], X, DEFAULT, FPU;
    b"mw"         , [0xDD              ], 7, DEFAULT, FPU;
]
"fpatan" = [
    b""           , [0xD9, 0xF3        ], X, DEFAULT, FPU;
]
"fprem" = [
    b""           , [0xD9, 0xF8        ], X, DEFAULT, FPU;
]
"fprem1" = [
    b""           , [0xD9, 0xF5        ], X, DEFAULT, FPU;
]
"fptan" = [
    b""           , [0xD9, 0xF2        ], X, DEFAULT, FPU;
]
"frndint" = [
    b""           , [0xD9, 0xFC        ], X, DEFAULT, FPU;
]
"frstor" = [
    b"m!"         , [0xDD              ], 4, DEFAULT, FPU;
]
"fsave" = [
    b"m!"         , [0x9B, 0xDD        ], 6, DEFAULT, FPU;
]
"fscale" = [
    b""           , [0xD9, 0xFD        ], X, DEFAULT, FPU;
]
"fsetpm" = [
    b""           , [0xDB, 0xE4        ], X, DEFAULT, FPU;
]
"fsin" = [
    b""           , [0xD9, 0xFE        ], X, DEFAULT, FPU;
]
"fsincos" = [
    b""           , [0xD9, 0xFB        ], X, DEFAULT, FPU;
]
"fsqrt" = [
    b""           , [0xD9, 0xFA        ], X, DEFAULT, FPU;
]
"fst" = [
    b""           , [0xDD, 0xD1        ], X, DEFAULT, FPU;
    b"fp"         , [0xDD, 0xD0        ], X, SHORT_ARG, FPU;
    b"md"         , [0xD9              ], 2, EXACT_SIZE, FPU;
    b"mq"         , [0xDD              ], 2, EXACT_SIZE, FPU;
]
"fstcw" = [
    b"mw"         , [0x9B, 0xD9        ], 7, DEFAULT, FPU;
]
"fstenv" = [
    b"m!"         , [0x9B, 0xD9        ], 6, DEFAULT, FPU;
]
"fstp" = [
    b""           , [0xDD, 0xD9        ], X, DEFAULT, FPU;
    b"fp"         , [0xDD, 0xD8        ], X, SHORT_ARG, FPU;
    b"md"         , [0xD9              ], 3, EXACT_SIZE, FPU;
    b"mp"         , [0xDB              ], 7, EXACT_SIZE, FPU;
    b"mq"         , [0xDD              ], 3, EXACT_SIZE, FPU;
]
"fstsw" = [
    b"Aw"         , [0x9B, 0xDF, 0xE0  ], X, DEFAULT, FPU;
    b"mw"         , [0x9B, 0xDD        ], 7, DEFAULT, FPU;
]
"fsub" = [
    b""           , [0xDE, 0xE9        ], X, DEFAULT, FPU;
    b"Xpfp"       , [0xD8, 0xE0        ], X, SHORT_ARG, FPU;
    b"fp"         , [0xD8, 0xE0        ], X, SHORT_ARG, FPU;
    b"fpXp"       , [0xDC, 0xE8        ], X, SHORT_ARG, FPU;
    b"fpXp"       , [0xDC, 0xE8        ], X, SHORT_ARG, FPU;
    b"md"         , [0xD8              ], 4, EXACT_SIZE, FPU;
    b"mq"         , [0xDC              ], 4, EXACT_SIZE, FPU;
]
"fsubp" = [
    b""           , [0xDE, 0xE9        ], X, DEFAULT, FPU;
    b"fp"         , [0xDE, 0xE8        ], X, SHORT_ARG, FPU;
    b"fpXp"       , [0xDE, 0xE8        ], X, SHORT_ARG, FPU;
]
"fsubr" = [
    b""           , [0xDE, 0xE1        ], X, DEFAULT, FPU;
    b"Xpfp"       , [0xD8, 0xE8        ], X, SHORT_ARG, FPU;
    b"fp"         , [0xD8, 0xE8        ], X, SHORT_ARG, FPU;
    b"fpXp"       , [0xDC, 0xE0        ], X, SHORT_ARG, FPU;
    b"fpXp"       , [0xDC, 0xE0        ], X, SHORT_ARG, FPU;
    b"md"         , [0xD8              ], 5, EXACT_SIZE, FPU;
    b"mq"         , [0xDC              ], 5, EXACT_SIZE, FPU;
]
"fsubrp" = [
    b""           , [0xDE, 0xE1        ], X, DEFAULT, FPU;
    b"fp"         , [0xDE, 0xE0        ], X, SHORT_ARG, FPU;
    b"fpXp"       , [0xDE, 0xE0        ], X, SHORT_ARG, FPU;
]
"ftst" = [
    b""           , [0xD9, 0xE4        ], X, DEFAULT, FPU;
]
"fucom" = [
    b""           , [0xDD, 0xE1        ], X, DEFAULT, FPU;
    b"Xpfp"       , [0xDD, 0xE0        ], X, SHORT_ARG, FPU;
    b"fp"         , [0xDD, 0xE0        ], X, SHORT_ARG, FPU;
]
"fucomi" = [
    b""           , [0xDB, 0xE9        ], X, DEFAULT, FPU;
    b"Xpfp"       , [0xDB, 0xE8        ], X, SHORT_ARG, FPU;
    b"fp"         , [0xDB, 0xE8        ], X, SHORT_ARG, FPU;
]
"fucomip" = [
    b""           , [0xDF, 0xE9        ], X, DEFAULT, FPU;
    b"Xpfp"       , [0xDF, 0xE8        ], X, SHORT_ARG, FPU;
    b"fp"         , [0xDF, 0xE8        ], X, SHORT_ARG, FPU;
]
"fucomp" = [
    b""           , [0xDD, 0xE9        ], X, DEFAULT, FPU;
    b"Xpfp"       , [0xDD, 0xE8        ], X, SHORT_ARG, FPU;
    b"fp"         , [0xDD, 0xE8        ], X, SHORT_ARG, FPU;
]
"fucompp" = [
    b""           , [0xDA, 0xE9        ], X, DEFAULT, FPU;
]
"fwait" = [
    b""           , [0x9B              ], X;
]
"fxam" = [
    b""           , [0xD9, 0xE5        ], X, DEFAULT, FPU;
]
"fxch" = [
    b""           , [0xD9, 0xC9        ], X, DEFAULT, FPU;
    b"Xpfp"       , [0xD9, 0xC8        ], X, SHORT_ARG, FPU;
    b"fp"         , [0xD9, 0xC8        ], X, SHORT_ARG, FPU;
    b"fpXp"       , [0xD9, 0xC8        ], X, SHORT_ARG, FPU;
]
"fxrstor" = [
    b"m!"         , [0x0F, 0xAE        ], 1, DEFAULT, SSE | FPU;
]
"fxrstor64" = [
    b"m!"         , [0x0F, 0xAE        ], 1, WITH_REXW, FPU | SSE;
]
"fxsave" = [
    b"m!"         , [0x0F, 0xAE        ], 0, DEFAULT, FPU | SSE;
]
"fxsave64" = [
    b"m!"         , [0x0F, 0xAE        ], 0, WITH_REXW, SSE | FPU;
]
"fxtract" = [
    b""           , [0xD9, 0xF4        ], X, DEFAULT, FPU;
]
"fyl2x" = [
    b""           , [0xD9, 0xF1        ], X, DEFAULT, FPU;
]
"fyl2xp1" = [
    b""           , [0xD9, 0xF9        ], X, DEFAULT, FPU;
]
"getsec" = [
    b""           , [0x0F, 0x37        ], X;
]
"haddpd" = [
    b"yowo"       , [0x0F, 0x7C        ], X, PREF_66, SSE3;
]
"haddps" = [
    b"yowo"       , [0x0F, 0x7C        ], X, PREF_F2, SSE3;
]
"hlt" = [
    b""           , [0xF4              ], X;
]
"hsubpd" = [
    b"yowo"       , [0x0F, 0x7D        ], X, PREF_66, SSE3;
]
"hsubps" = [
    b"yowo"       , [0x0F, 0x7D        ], X, PREF_F2, SSE3;
]
"icebp" = [
    b""           , [0xF1              ], X;
]
"idiv" = [
    b"vb"         , [0xF6              ], 7;
    b"v*"         , [0xF7              ], 7, AUTO_SIZE;
]
"inc" = [
    b"mb"         , [0xFE              ], 0, LOCK;
    b"rb"         , [0xFE              ], 0;
    b"m*"         , [0xFF              ], 0, AUTO_SIZE | LOCK;
    b"r*"         , [0x40              ], 0, X86_ONLY | SHORT_ARG;
    b"r*"         , [0xFF              ], 0, AUTO_SIZE ;
]
"insb" = [
    b""           , [0x6C              ], X, REP;
]
"insd" = [
    b""           , [0x6D              ], X, REP;
]
"insertps" = [
    b"yomdib"     , [0x0F, 0x3A, 0x21  ], X, PREF_66, SSE41;
    b"yoyoib"     , [0x0F, 0x3A, 0x21  ], X, PREF_66, SSE41;
]
"insertq" = [
    b"yoyo"       , [0x0F, 0x79        ], X, PREF_F2, SSE4A | AMD;
    b"yoyoibib"   , [0x0F, 0x78        ], X, PREF_F2, AMD | SSE4A;
]
"insw" = [
    b""           , [0x6D              ], X, WORD_SIZE | REP;
]
"int" = [
    b"ib"         , [0xCD              ], X;
]
"into" = [
    b""           , [0xCE              ], X, X86_ONLY;
]
"int01" = [
    b""           , [0xF1              ], X;
]
"int03" = [
    b""           , [0xCC              ], X;
]
"int1" = [
    b""           , [0xF1              ], X;
]
"int3" = [
    b""           , [0xCC              ], X;
]
"invd" = [
    b""           , [0x0F, 0x08        ], X;
]
"invept" = [
    b"rqmo"       , [0x0F, 0x38, 0x80  ], X, PREF_66, VMX;
]
"invlpg" = [
    b"m!"         , [0x0F, 0x01        ], 7;
]
"invlpga" = [
    b""           , [0x0F, 0x01, 0xDF  ], X, DEFAULT, AMD;
    b"AqBd"       , [0x0F, 0x01, 0xDF  ], X, DEFAULT, AMD;
]
"invpcid" = [
    b"rqmo"       , [0x0F, 0x38, 0x82  ], X, PREF_66, INVPCID;
]
"invvpid" = [
    b"rqmo"       , [0x0F, 0x38, 0x81  ], X, PREF_66, VMX;
]
"iret" = [
    b""           , [0xCF              ], X;
]
"iretd" = [
    b""           , [0xCF              ], X;
]
"iretq" = [
    b""           , [0xCF              ], X, WITH_REXW;
]
"iretw" = [
    b""           , [0xCF              ], X, WORD_SIZE;
]
"jecxz" = [
    b"ob"         , [0xE3              ], X, PREF_67;
]
"jrcxz" = [
    b"ob"         , [0xE3              ], X;
]
"lahf" = [
    b""           , [0x9F              ], X;
]
"lar" = [
    b"r*mw"       , [0x0F, 0x02        ], X, AUTO_SIZE;
    b"r*r*"       , [0x0F, 0x02        ], X, AUTO_SIZE;
]
"lddqu" = [
    b"yomo"       , [0x0F, 0xF0        ], X, PREF_F2, SSE3;
]
"ldmxcsr" = [
    b"md"         , [0x0F, 0xAE        ], 2, DEFAULT, SSE;
]
"lds" = [
    b"r*m!"       , [0xC5              ], X, AUTO_SIZE | X86_ONLY;
]
"lea" = [
    b"r*m!"       , [0x8D              ], X, AUTO_SIZE;
]
"leave" = [
    b""           , [0xC9              ], X;
]
"les" = [
    b"r*m!"       , [0xC4              ], X, AUTO_SIZE | X86_ONLY;
]
"lfence" = [
    b""           , [0x0F, 0xAE, 0xE8  ], X, DEFAULT, AMD;
]
"lfs" = [
    b"r*m!"       , [0x0F, 0xB4        ], X, AUTO_SIZE;
]
"lgdt" = [
    b"m!"         , [0x0F, 0x01        ], 2;
]
"lgs" = [
    b"r*m!"       , [0x0F, 0xB5        ], X, AUTO_SIZE;
]
"lidt" = [
    b"m!"         , [0x0F, 0x01        ], 3;
]
"lldt" = [
    b"m!"         , [0x0F, 0x00        ], 2;
    b"rw"         , [0x0F, 0x00        ], 2;
]
"llwpcb" = [
    b"r*"         , [0x09, 0x12        ], 0, XOP_OP | AUTO_REXW, AMD;
]
"lmsw" = [
    b"m!"         , [0x0F, 0x01        ], 6;
    b"rw"         , [0x0F, 0x01        ], 6;
]
"lodsb" = [
    b""           , [0xAC              ], X, REP;
]
"lodsd" = [
    b""           , [0xAD              ], X, REP;
]
"lodsq" = [
    b""           , [0xAD              ], X, WITH_REXW | REP;
]
"lodsw" = [
    b""           , [0xAD              ], X, WORD_SIZE | REP;
]
"loop" = [
    b"ob"         , [0xE2              ], X;
]
"loope" = [
    b"ob"         , [0xE1              ], X;
]
"loopne" = [
    b"ob"         , [0xE0              ], X;
]
"loopnz" = [
    b"ob"         , [0xE0              ], X;
]
"loopz" = [
    b"ob"         , [0xE1              ], X;
]
"lsl" = [
    b"r*mw"       , [0x0F, 0x03        ], X, AUTO_SIZE;
    b"r*r*"       , [0x0F, 0x03        ], X, AUTO_SIZE;
]
"lss" = [
    b"r*m!"       , [0x0F, 0xB2        ], X, AUTO_SIZE;
]
"ltr" = [
    b"m!"         , [0x0F, 0x00        ], 3;
    b"rw"         , [0x0F, 0x00        ], 3;
]
"lwpins" = [
    b"r*v*id"     , [0x10, 0x12        ], 0, XOP_OP | AUTO_REXW | ENC_VM, AMD;
]
"lwpval" = [
    b"r*v*id"     , [0x10, 0x12        ], 1, XOP_OP | AUTO_REXW | ENC_VM, AMD;
]
"lzcnt" = [
    b"r*v*"       , [0x0F, 0xBD        ], X, AUTO_SIZE | PREF_F3, AMD;
]
"maskmovdqu" = [
    b"yoyo"       , [0x0F, 0xF7        ], X, PREF_66, SSE2;
]
"maskmovq" = [
    b"xqxq"       , [0x0F, 0xF7        ], X, DEFAULT, MMX;
]
"maxpd" = [
    b"yowo"       , [0x0F, 0x5F        ], X, PREF_66, SSE2;
]
"maxps" = [
    b"yowo"       , [0x0F, 0x5F        ], X, DEFAULT, SSE;
]
"maxsd" = [
    b"yomq"       , [0x0F, 0x5F        ], X, PREF_F2, SSE2;
    b"yoyo"       , [0x0F, 0x5F        ], X, PREF_F2, SSE2;
]
"maxss" = [
    b"yomd"       , [0x0F, 0x5F        ], X, PREF_F3, SSE;
    b"yoyo"       , [0x0F, 0x5F        ], X, PREF_F3, SSE;
]
"mfence" = [
    b""           , [0x0F, 0xAE, 0xF0  ], X, DEFAULT, AMD;
]
"minpd" = [
    b"yowo"       , [0x0F, 0x5D        ], X, PREF_66, SSE2;
]
"minps" = [
    b"yowo"       , [0x0F, 0x5D        ], X, DEFAULT, SSE;
]
"minsd" = [
    b"yomq"       , [0x0F, 0x5D        ], X, PREF_F2, SSE2;
    b"yoyo"       , [0x0F, 0x5D        ], X, PREF_F2, SSE2;
]
"minss" = [
    b"yomd"       , [0x0F, 0x5D        ], X, PREF_F3, SSE;
    b"yoyo"       , [0x0F, 0x5D        ], X, PREF_F3, SSE;
]
"monitor" = [
    b""           , [0x0F, 0x01, 0xC8  ], X;
    b"AqBdCd"     , [0x0F, 0x01, 0xC8  ], X;
]
"monitorx" = [
    b""           , [0x0F, 0x01, 0xFA  ], X, DEFAULT, AMD;
    b"A*BdCd"     , [0x0F, 0x01, 0xFA  ], X, DEFAULT, AMD;
]
"montmul" = [
    b""           , [0x0F, 0xA6, 0xC0  ], X, PREF_F3, CYRIX;
]
"movapd" = [
    b"moyo"       , [0x0F, 0x29        ], X, ENC_MR | PREF_66, SSE2;
    b"yomo"       , [0x0F, 0x28        ], X, PREF_66, SSE2;
    b"yoyo"       , [0x0F, 0x28        ], X, PREF_66, SSE2;
    b"yoyo"       , [0x0F, 0x29        ], X, ENC_MR | PREF_66, SSE2;
]
"movaps" = [
    b"yowo"       , [0x0F, 0x28        ], X, DEFAULT, SSE;
    b"woyo"       , [0x0F, 0x29        ], X, ENC_MR, SSE;
]
"movbe" = [
    b"m*r*"       , [0x0F, 0x38, 0xF1  ], X, AUTO_SIZE | ENC_MR;
    b"r*m*"       , [0x0F, 0x38, 0xF0  ], X, AUTO_SIZE;
]
"movd" = [
    b"mdyo"       , [0x0F, 0x7E        ], X, ENC_MR | PREF_66, SSE2;
    b"xqvd"       , [0x0F, 0x6E        ], X, DEFAULT, MMX;
    b"xqvq"       , [0x0F, 0x6E        ], X, WITH_REXW, MMX;
    b"yomd"       , [0x0F, 0x6E        ], X, PREF_66, SSE2;
    b"yovd"       , [0x0F, 0x6E        ], X, PREF_66, SSE2;
    b"vdxq"       , [0x0F, 0x7E        ], X, ENC_MR, MMX;
    b"vdyo"       , [0x0F, 0x7E        ], X, ENC_MR | PREF_66, SSE2;
    b"vqxq"       , [0x0F, 0x7E        ], X, WITH_REXW | ENC_MR, MMX;
]
"movddup" = [
    b"yomq"       , [0x0F, 0x12        ], X, PREF_F2, SSE3;
    b"yoyo"       , [0x0F, 0x12        ], X, PREF_F2, SSE3;
]
"movdq2q" = [
    b"xqyo"       , [0x0F, 0xD6        ], X, PREF_F2, SSE2;
]
"movdqa" = [
    b"moyo"       , [0x0F, 0x7F        ], X, ENC_MR | PREF_66, SSE2;
    b"yomo"       , [0x0F, 0x6F        ], X, PREF_66, SSE2;
    b"yoyo"       , [0x0F, 0x6F        ], X, PREF_66, SSE2;
    b"yoyo"       , [0x0F, 0x7F        ], X, ENC_MR | PREF_66, SSE2;
]
"movdqu" = [
    b"moyo"       , [0x0F, 0x7F        ], X, ENC_MR | PREF_F3, SSE2;
    b"yomo"       , [0x0F, 0x6F        ], X, PREF_F3, SSE2;
    b"yoyo"       , [0x0F, 0x6F        ], X, PREF_F3, SSE2;
    b"yoyo"       , [0x0F, 0x7F        ], X, ENC_MR | PREF_F3, SSE2;
]
"movhlps" = [
    b"yoyo"       , [0x0F, 0x12        ], X, DEFAULT, SSE;
]
"movhpd" = [
    b"m!yo"       , [0x0F, 0x17        ], X, ENC_MR | PREF_66, SSE2;
    b"yom!"       , [0x0F, 0x16        ], X, PREF_66, SSE2;
]
"movhps" = [
    b"mqyo"       , [0x0F, 0x17        ], X, ENC_MR, SSE;
    b"yomq"       , [0x0F, 0x16        ], X, DEFAULT, SSE;
]
"movlhps" = [
    b"yoyo"       , [0x0F, 0x16        ], X, DEFAULT, SSE;
]
"movlpd" = [
    b"mqyo"       , [0x0F, 0x13        ], X, ENC_MR | PREF_66, SSE2;
    b"yomq"       , [0x0F, 0x12        ], X, PREF_66, SSE2;
]
"movlps" = [
    b"mqyo"       , [0x0F, 0x13        ], X, ENC_MR, SSE;
    b"yomq"       , [0x0F, 0x12        ], X, DEFAULT, SSE;
]
"movmskpd" = [
    b"rdyo"       , [0x0F, 0x50        ], X, PREF_66, SSE2;
    b"rqyo"       , [0x0F, 0x50        ], X, WITH_REXW | PREF_66, SSE2;
]
"movmskps" = [
    b"rdyo"       , [0x0F, 0x50        ], X, DEFAULT, SSE;
    b"rqyo"       , [0x0F, 0x50        ], X, WITH_REXW, SSE;
]
"movntdq" = [
    b"moyo"       , [0x0F, 0xE7        ], X, ENC_MR | PREF_66, SSE2;
]
"movntdqa" = [
    b"yomo"       , [0x0F, 0x38, 0x2A  ], X, PREF_66, SSE41;
]
"movnti" = [
    b"mdrd"       , [0x0F, 0xC3        ], X, ENC_MR;
    b"mqrq"       , [0x0F, 0xC3        ], X, WITH_REXW | ENC_MR;
]
"movntpd" = [
    b"moyo"       , [0x0F, 0x2B        ], X, ENC_MR | PREF_66, SSE2;
]
"movntps" = [
    b"moyo"       , [0x0F, 0x2B        ], X, ENC_MR, SSE;
]
"movntq" = [
    b"mqxq"       , [0x0F, 0xE7        ], X, ENC_MR, MMX;
]
"movntsd" = [
    b"mqyo"       , [0x0F, 0x2B        ], X, ENC_MR | PREF_F2, AMD | SSE4A;
]
"movntss" = [
    b"mdyo"       , [0x0F, 0x2B        ], X, ENC_MR | PREF_F3, SSE4A | AMD;
]
"movq" = [
    b"mqyo"       , [0x0F, 0xD6        ], X, ENC_MR | PREF_66, SSE2;
    b"xquq"       , [0x0F, 0x6F        ], X, DEFAULT, MMX;
    b"xqvq"       , [0x0F, 0x6E        ], X, WITH_REXW, MMX;
    b"yomq"       , [0x0F, 0x7E        ], X, PREF_F3, SSE2;
    b"yovq"       , [0x0F, 0x6E        ], X, WITH_REXW | PREF_66, SSE2;
    b"yoyo"       , [0x0F, 0x7E        ], X, PREF_F3, SSE2;
    b"yoyo"       , [0x0F, 0xD6        ], X, ENC_MR | PREF_66, SSE2;
    b"uqxq"       , [0x0F, 0x7F        ], X, ENC_MR, MMX;
    b"vqxq"       , [0x0F, 0x7E        ], X, WITH_REXW | ENC_MR, MMX;
    b"vqyo"       , [0x0F, 0x7E        ], X, WITH_REXW | ENC_MR | PREF_66, SSE2;
]
"movq2dq" = [
    b"yoxq"       , [0x0F, 0xD6        ], X, PREF_F3, SSE2;
]
"movsb" = [
    b""           , [0xA4              ], X, REP;
]
"movsd" = [
    b""           , [0xA5              ], X, REP;
    b"mqyo"       , [0x0F, 0x11        ], X, ENC_MR | PREF_F2, SSE2;
    b"yomq"       , [0x0F, 0x10        ], X, PREF_F2, SSE2;
    b"yoyo"       , [0x0F, 0x10        ], X, PREF_F2, SSE2;
    b"yoyo"       , [0x0F, 0x11        ], X, ENC_MR | PREF_F2, SSE2;
]
"movshdup" = [
    b"yomq"       , [0x0F, 0x16        ], X, PREF_F3, SSE3;
    b"yoyo"       , [0x0F, 0x16        ], X, PREF_F3, SSE3;
]
"movsldup" = [
    b"yomq"       , [0x0F, 0x12        ], X, PREF_F3, SSE3;
    b"yoyo"       , [0x0F, 0x12        ], X, PREF_F3, SSE3;
]
"movsq" = [
    b""           , [0xA5              ], X, WITH_REXW | REP;
]
"movss" = [
    b"mdyo"       , [0x0F, 0x11        ], X, ENC_MR | PREF_F3, SSE;
    b"yomd"       , [0x0F, 0x10        ], X, PREF_F3, SSE;
    b"yoyo"       , [0x0F, 0x10        ], X, PREF_F3, SSE;
]
"movsw" = [
    b""           , [0xA5              ], X, WORD_SIZE | REP;
]
"movsx" = [
    b"rqvd"       , [0x63              ], X, WITH_REXW;
    b"rwmb"       , [0x0F, 0xBE        ], X, WORD_SIZE;
    b"r*vb"       , [0x0F, 0xBE        ], X, AUTO_SIZE;
    b"r*vw"       , [0x0F, 0xBF        ], X, AUTO_REXW | EXACT_SIZE;
]
"movsxd" = [
    b"rqvd"       , [0x63              ], X, WITH_REXW;
]
"movupd" = [
    b"moyo"       , [0x0F, 0x11        ], X, ENC_MR | PREF_66, SSE2;
    b"yomo"       , [0x0F, 0x10        ], X, PREF_66, SSE2;
    b"yoyo"       , [0x0F, 0x10        ], X, PREF_66, SSE2;
    b"yoyo"       , [0x0F, 0x11        ], X, ENC_MR | PREF_66, SSE2;
]
"movups" = [
    b"yowo"       , [0x0F, 0x10        ], X, DEFAULT, SSE;
    b"woyo"       , [0x0F, 0x11        ], X, ENC_MR, SSE;
]
"movzx" = [
    b"rwmb"       , [0x0F, 0xB6        ], X, WORD_SIZE;
    b"r*vb"       , [0x0F, 0xB6        ], X, AUTO_SIZE;
    b"r*vw"       , [0x0F, 0xB7        ], X, AUTO_REXW | EXACT_SIZE;
]
"mpsadbw" = [
    b"yomqib"     , [0x0F, 0x3A, 0x42  ], X, PREF_66, SSE41;
    b"yoyoib"     , [0x0F, 0x3A, 0x42  ], X, PREF_66, SSE41;
]
"mul" = [
    b"vb"         , [0xF6              ], 4;
    b"v*"         , [0xF7              ], 4, AUTO_SIZE;
]
"mulpd" = [
    b"yowo"       , [0x0F, 0x59        ], X, PREF_66, SSE2;
]
"mulps" = [
    b"yowo"       , [0x0F, 0x59        ], X, DEFAULT, SSE;
]
"mulsd" = [
    b"yomq"       , [0x0F, 0x59        ], X, PREF_F2, SSE2;
    b"yoyo"       , [0x0F, 0x59        ], X, PREF_F2, SSE2;
]
"mulss" = [
    b"yomd"       , [0x0F, 0x59        ], X, PREF_F3, SSE;
    b"yoyo"       , [0x0F, 0x59        ], X, PREF_F3, SSE;
]
"mulx" = [
    b"r*r*v*"     , [0x02, 0xF6        ], X, VEX_OP | AUTO_REXW | PREF_F2, BMI2;
]
"mwait" = [
    b""           , [0x0F, 0x01, 0xC9  ], X;
    b"AdBd"       , [0x0F, 0x01, 0xC9  ], X;
]
"mwaitx" = [
    b""           , [0x0F, 0x01, 0xFB  ], X, DEFAULT, AMD;
    b"AdBd"       , [0x0F, 0x01, 0xFB  ], X, DEFAULT, AMD;
]
"neg" = [
    b"mb"         , [0xF6              ], 3, LOCK;
    b"rb"         , [0xF6              ], 3;
    b"m*"         , [0xF7              ], 3, AUTO_SIZE | LOCK;
    b"r*"         , [0xF7              ], 3, AUTO_SIZE ;
]
"nop" = [
    b""           , [0x90              ], X;
    b"v*"         , [0x0F, 0x1F        ], 0, AUTO_SIZE;
]
"not" = [
    b"mb"         , [0xF6              ], 2, LOCK;
    b"rb"         , [0xF6              ], 2;
    b"m*"         , [0xF7              ], 2, AUTO_SIZE | LOCK;
    b"r*"         , [0xF7              ], 2, AUTO_SIZE ;
]
"or" = [
    b"Abib"       , [0x0C              ], X;
    b"mbib"       , [0x80              ], 1, LOCK;
    b"mbrb"       , [0x08              ], X, LOCK | ENC_MR;
    b"rbib"       , [0x80              ], 1;
    b"rbrb"       , [0x08              ], X, ENC_MR;
    b"rbvb"       , [0x0A              ], X;
    b"r*ib"       , [0x83              ], 1, AUTO_SIZE  | EXACT_SIZE;
    b"A*i*"       , [0x0D              ], X, AUTO_SIZE;
    b"m*i*"       , [0x81              ], 1, AUTO_SIZE | LOCK;
    b"m*ib"       , [0x83              ], 1, AUTO_SIZE | LOCK;
    b"m*r*"       , [0x09              ], X, AUTO_SIZE | LOCK | ENC_MR;
    b"r*i*"       , [0x81              ], 1, AUTO_SIZE ;
    b"r*r*"       , [0x09              ], X, AUTO_SIZE | ENC_MR;
    b"r*v*"       , [0x0B              ], X, AUTO_SIZE;
]
"orpd" = [
    b"yowo"       , [0x0F, 0x56        ], X, PREF_66, SSE2;
]
"orps" = [
    b"yowo"       , [0x0F, 0x56        ], X, DEFAULT, SSE;
]
"outsb" = [
    b""           , [0x6E              ], X, REP;
]
"outsd" = [
    b""           , [0x6F              ], X, REP;
]
"outsw" = [
    b""           , [0x6F              ], X, WORD_SIZE | REP;
]
"pabsb" = [
    b"xquq"       , [0x0F, 0x38, 0x1C  ], X, DEFAULT, MMX | SSSE3;
    b"yomq"       , [0x0F, 0x38, 0x1C  ], X, PREF_66, SSSE3;
    b"yoyo"       , [0x0F, 0x38, 0x1C  ], X, PREF_66, SSSE3;
]
"pabsd" = [
    b"xquq"       , [0x0F, 0x38, 0x1E  ], X, DEFAULT, MMX | SSSE3;
    b"yomq"       , [0x0F, 0x38, 0x1E  ], X, PREF_66, SSSE3;
    b"yoyo"       , [0x0F, 0x38, 0x1E  ], X, PREF_66, SSSE3;
]
"pabsw" = [
    b"xquq"       , [0x0F, 0x38, 0x1D  ], X, DEFAULT, SSSE3 | MMX;
    b"yomq"       , [0x0F, 0x38, 0x1D  ], X, PREF_66, SSSE3;
    b"yoyo"       , [0x0F, 0x38, 0x1D  ], X, PREF_66, SSSE3;
]
"packssdw" = [
    b"xquq"       , [0x0F, 0x6B        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0x6B        ], X, PREF_66, SSE2;
]
"packsswb" = [
    b"xquq"       , [0x0F, 0x63        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0x63        ], X, PREF_66, SSE2;
]
"packusdw" = [
    b"yomq"       , [0x0F, 0x38, 0x2B  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x2B  ], X, PREF_66, SSE41;
]
"packuswb" = [
    b"xquq"       , [0x0F, 0x67        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0x67        ], X, PREF_66, SSE2;
]
"paddb" = [
    b"xquq"       , [0x0F, 0xFC        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xFC        ], X, PREF_66, SSE2;
]
"paddd" = [
    b"xquq"       , [0x0F, 0xFE        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xFE        ], X, PREF_66, SSE2;
]
"paddq" = [
    b"xquq"       , [0x0F, 0xD4        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xD4        ], X, PREF_66, SSE2;
]
"paddsb" = [
    b"xquq"       , [0x0F, 0xEC        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xEC        ], X, PREF_66, SSE2;
]
"paddsiw" = [
    b"xquq"       , [0x0F, 0x51        ], X, DEFAULT, MMX | CYRIX;
]
"paddsw" = [
    b"xquq"       , [0x0F, 0xED        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xED        ], X, PREF_66, SSE2;
]
"paddusb" = [
    b"xquq"       , [0x0F, 0xDC        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xDC        ], X, PREF_66, SSE2;
]
"paddusw" = [
    b"xquq"       , [0x0F, 0xDD        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xDD        ], X, PREF_66, SSE2;
]
"paddw" = [
    b"xquq"       , [0x0F, 0xFD        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xFD        ], X, PREF_66, SSE2;
]
"palignr" = [
    b"xquqib"     , [0x0F, 0x3A, 0x0F  ], X, DEFAULT, SSSE3 | MMX;
    b"yomqib"     , [0x0F, 0x3A, 0x0F  ], X, PREF_66, SSSE3;
    b"yoyoib"     , [0x0F, 0x3A, 0x0F  ], X, PREF_66, SSSE3;
]
"pand" = [
    b"xquq"       , [0x0F, 0xDB        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xDB        ], X, PREF_66, SSE2;
]
"pandn" = [
    b"xquq"       , [0x0F, 0xDF        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xDF        ], X, PREF_66, SSE2;
]
"pause" = [
    b""           , [0x90              ], X, PREF_F3;
]
"paveb" = [
    b"xquq"       , [0x0F, 0x50        ], X, DEFAULT, MMX | CYRIX;
]
"pavgb" = [
    b"xquq"       , [0x0F, 0xE0        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xE0        ], X, PREF_66, SSE2;
]
"pavgusb" = [
    b"xquq"       , [0x0F, 0x0F, 0xBF  ], X, IMM_OP, TDNOW;
]
"pavgw" = [
    b"xquq"       , [0x0F, 0xE3        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xE3        ], X, PREF_66, SSE2;
]
"pblendvb" = [
    b"yomq"       , [0x0F, 0x38, 0x10  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x10  ], X, PREF_66, SSE41;
]
"pblendw" = [
    b"yomqib"     , [0x0F, 0x3A, 0x0E  ], X, PREF_66, SSE41;
    b"yoyoib"     , [0x0F, 0x3A, 0x0E  ], X, PREF_66, SSE41;
]
"pclmulhqhqdq" = [
    b"yowo"       , [0x0F, 0x3A, 0x44, 0x11], X, IMM_OP | PREF_66, SSE;
]
"pclmulhqlqdq" = [
    b"yowo"       , [0x0F, 0x3A, 0x44, 0x01], X, PREF_66 | IMM_OP, SSE;
]
"pclmullqhqdq" = [
    b"yowo"       , [0x0F, 0x3A, 0x44, 0x10], X, PREF_66 | IMM_OP, SSE;
]
"pclmullqlqdq" = [
    b"yowo"       , [0x0F, 0x3A, 0x44, 0x00], X, PREF_66 | IMM_OP, SSE;
]
"pclmulqdq" = [
    b"yowoib"     , [0x0F, 0x3A, 0x44  ], X, PREF_66, SSE;
]
"pcmpeqb" = [
    b"xquq"       , [0x0F, 0x74        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0x74        ], X, PREF_66, SSE2;
]
"pcmpeqd" = [
    b"xquq"       , [0x0F, 0x76        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0x76        ], X, PREF_66, SSE2;
]
"pcmpeqq" = [
    b"yomq"       , [0x0F, 0x38, 0x29  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x29  ], X, PREF_66, SSE41;
]
"pcmpeqw" = [
    b"xquq"       , [0x0F, 0x75        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0x75        ], X, PREF_66, SSE2;
]
"pcmpestri" = [
    b"yomqib"     , [0x0F, 0x3A, 0x61  ], X, PREF_66, SSE42;
    b"yoyoib"     , [0x0F, 0x3A, 0x61  ], X, PREF_66, SSE42;
]
"pcmpestrm" = [
    b"yomqib"     , [0x0F, 0x3A, 0x60  ], X, PREF_66, SSE42;
    b"yoyoib"     , [0x0F, 0x3A, 0x60  ], X, PREF_66, SSE42;
]
"pcmpgtb" = [
    b"xquq"       , [0x0F, 0x64        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0x64        ], X, PREF_66, SSE2;
]
"pcmpgtd" = [
    b"xquq"       , [0x0F, 0x66        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0x66        ], X, PREF_66, SSE2;
]
"pcmpgtq" = [
    b"yomq"       , [0x0F, 0x38, 0x37  ], X, PREF_66, SSE42;
    b"yoyo"       , [0x0F, 0x38, 0x37  ], X, PREF_66, SSE42;
]
"pcmpgtw" = [
    b"xquq"       , [0x0F, 0x65        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0x65        ], X, PREF_66, SSE2;
]
"pcmpistri" = [
    b"yomqib"     , [0x0F, 0x3A, 0x63  ], X, PREF_66, SSE42;
    b"yoyoib"     , [0x0F, 0x3A, 0x63  ], X, PREF_66, SSE42;
]
"pcmpistrm" = [
    b"yomqib"     , [0x0F, 0x3A, 0x62  ], X, PREF_66, SSE42;
    b"yoyoib"     , [0x0F, 0x3A, 0x62  ], X, PREF_66, SSE42;
]
"pdep" = [
    b"r*r*v*"     , [0x02, 0xF5        ], X, VEX_OP | AUTO_REXW | PREF_F2, BMI2;
]
"pdistib" = [
    b"xqmq"       , [0x0F, 0x54        ], X, DEFAULT, MMX | CYRIX;
]
"pext" = [
    b"r*r*v*"     , [0x02, 0xF5        ], X, VEX_OP | AUTO_REXW | PREF_F3, BMI2;
]
"pextrb" = [
    b"mbyoib"     , [0x0F, 0x3A, 0x14  ], X, ENC_MR | PREF_66, SSE41;
    b"rdyoib"     , [0x0F, 0x3A, 0x14  ], X, ENC_MR | PREF_66, SSE41;
    b"rqyoib"     , [0x0F, 0x3A, 0x14  ], X, WITH_REXW | ENC_MR | PREF_66, SSE41;
]
"pextrd" = [
    b"vdyoib"     , [0x0F, 0x3A, 0x16  ], X, ENC_MR | PREF_66, SSE41;
]
"pextrq" = [
    b"vqyoib"     , [0x0F, 0x3A, 0x16  ], X, WITH_REXW | ENC_MR | PREF_66, SSE41;
]
"pextrw" = [
    b"mwyoib"     , [0x0F, 0x3A, 0x15  ], X, ENC_MR | PREF_66, SSE41;
    b"rdxqib"     , [0x0F, 0xC5        ], X, DEFAULT, MMX;
    b"rdyoib"     , [0x0F, 0xC5        ], X, PREF_66, SSE2;
    b"rdyoib"     , [0x0F, 0x3A, 0x15  ], X, ENC_MR | PREF_66, SSE41;
    b"rqyoib"     , [0x0F, 0x3A, 0x15  ], X, WITH_REXW | ENC_MR | PREF_66, SSE41;
]
"pf2id" = [
    b"xquq"       , [0x0F, 0x0F, 0x1D  ], X, IMM_OP, TDNOW;
]
"pf2iw" = [
    b"xquq"       , [0x0F, 0x0F, 0x1C  ], X, IMM_OP, TDNOW;
]
"pfacc" = [
    b"xquq"       , [0x0F, 0x0F, 0xAE  ], X, IMM_OP, TDNOW;
]
"pfadd" = [
    b"xquq"       , [0x0F, 0x0F, 0x9E  ], X, IMM_OP, TDNOW;
]
"pfcmpeq" = [
    b"xquq"       , [0x0F, 0x0F, 0xB0  ], X, IMM_OP, TDNOW;
]
"pfcmpge" = [
    b"xquq"       , [0x0F, 0x0F, 0x90  ], X, IMM_OP, TDNOW;
]
"pfcmpgt" = [
    b"xquq"       , [0x0F, 0x0F, 0xA0  ], X, IMM_OP, TDNOW;
]
"pfmax" = [
    b"xquq"       , [0x0F, 0x0F, 0xA4  ], X, IMM_OP, TDNOW;
]
"pfmin" = [
    b"xquq"       , [0x0F, 0x0F, 0x94  ], X, IMM_OP, TDNOW;
]
"pfmul" = [
    b"xquq"       , [0x0F, 0x0F, 0xB4  ], X, IMM_OP, TDNOW;
]
"pfnacc" = [
    b"xquq"       , [0x0F, 0x0F, 0x8A  ], X, IMM_OP, TDNOW;
]
"pfpnacc" = [
    b"xquq"       , [0x0F, 0x0F, 0x8E  ], X, IMM_OP, TDNOW;
]
"pfrcp" = [
    b"xquq"       , [0x0F, 0x0F, 0x96  ], X, IMM_OP, TDNOW;
]
"pfrcpit1" = [
    b"xquq"       , [0x0F, 0x0F, 0xA6  ], X, IMM_OP, TDNOW;
]
"pfrcpit2" = [
    b"xquq"       , [0x0F, 0x0F, 0xB6  ], X, IMM_OP, TDNOW;
]
"pfrcpv" = [
    b"xquq"       , [0x0F, 0x0F, 0x86  ], X, IMM_OP, TDNOW | CYRIX;
]
"pfrsqit1" = [
    b"xquq"       , [0x0F, 0x0F, 0xA7  ], X, IMM_OP, TDNOW;
]
"pfrsqrt" = [
    b"xquq"       , [0x0F, 0x0F, 0x97  ], X, IMM_OP, TDNOW;
]
"pfrsqrtv" = [
    b"xquq"       , [0x0F, 0x0F, 0x87  ], X, IMM_OP, CYRIX | TDNOW;
]
"pfsub" = [
    b"xquq"       , [0x0F, 0x0F, 0x9A  ], X, IMM_OP, TDNOW;
]
"pfsubr" = [
    b"xquq"       , [0x0F, 0x0F, 0xAA  ], X, IMM_OP, TDNOW;
]
"phaddd" = [
    b"xquq"       , [0x0F, 0x38, 0x02  ], X, DEFAULT, MMX | SSSE3;
    b"yomq"       , [0x0F, 0x38, 0x02  ], X, PREF_66, SSSE3;
    b"yoyo"       , [0x0F, 0x38, 0x02  ], X, PREF_66, SSSE3;
]
"phaddsw" = [
    b"xquq"       , [0x0F, 0x38, 0x03  ], X, DEFAULT, MMX | SSSE3;
    b"yomq"       , [0x0F, 0x38, 0x03  ], X, PREF_66, SSSE3;
    b"yoyo"       , [0x0F, 0x38, 0x03  ], X, PREF_66, SSSE3;
]
"phaddw" = [
    b"xquq"       , [0x0F, 0x38, 0x01  ], X, DEFAULT, MMX | SSSE3;
    b"yomq"       , [0x0F, 0x38, 0x01  ], X, PREF_66, SSSE3;
    b"yoyo"       , [0x0F, 0x38, 0x01  ], X, PREF_66, SSSE3;
]
"phminposuw" = [
    b"yomq"       , [0x0F, 0x38, 0x41  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x41  ], X, PREF_66, SSE41;
]
"phsubd" = [
    b"xquq"       , [0x0F, 0x38, 0x06  ], X, DEFAULT, SSSE3 | MMX;
    b"yomq"       , [0x0F, 0x38, 0x06  ], X, PREF_66, SSSE3;
    b"yoyo"       , [0x0F, 0x38, 0x06  ], X, PREF_66, SSSE3;
]
"phsubsw" = [
    b"xquq"       , [0x0F, 0x38, 0x07  ], X, DEFAULT, SSSE3 | MMX;
    b"yomq"       , [0x0F, 0x38, 0x07  ], X, PREF_66, SSSE3;
    b"yoyo"       , [0x0F, 0x38, 0x07  ], X, PREF_66, SSSE3;
]
"phsubw" = [
    b"xquq"       , [0x0F, 0x38, 0x05  ], X, DEFAULT, SSSE3 | MMX;
    b"yomq"       , [0x0F, 0x38, 0x05  ], X, PREF_66, SSSE3;
    b"yoyo"       , [0x0F, 0x38, 0x05  ], X, PREF_66, SSSE3;
]
"pi2fd" = [
    b"xquq"       , [0x0F, 0x0F, 0x0D  ], X, IMM_OP, TDNOW;
]
"pi2fw" = [
    b"xquq"       , [0x0F, 0x0F, 0x0C  ], X, IMM_OP, TDNOW;
]
"pinsrb" = [
    b"yom!ib"     , [0x0F, 0x3A, 0x20  ], X, PREF_66, SSE41;
    b"yordib"     , [0x0F, 0x3A, 0x20  ], X, PREF_66, SSE41;
    b"yovbib"     , [0x0F, 0x3A, 0x20  ], X, PREF_66, SSE41;
]
"pinsrd" = [
    b"yom!ib"     , [0x0F, 0x3A, 0x22  ], X, PREF_66, SSE41;
    b"yovdib"     , [0x0F, 0x3A, 0x22  ], X, PREF_66, SSE41;
]
"pinsrq" = [
    b"yom!ib"     , [0x0F, 0x3A, 0x22  ], X, WITH_REXW | PREF_66, SSE41;
    b"yovqib"     , [0x0F, 0x3A, 0x22  ], X, WITH_REXW | PREF_66, SSE41;
]
"pinsrw" = [
    b"xqm!ib"     , [0x0F, 0xC4        ], X, DEFAULT, MMX;
    b"xqrdib"     , [0x0F, 0xC4        ], X, DEFAULT, MMX;
    b"xqvwib"     , [0x0F, 0xC4        ], X, DEFAULT, MMX;
    b"yom!ib"     , [0x0F, 0xC4        ], X, PREF_66, SSE2;
    b"yomwib"     , [0x0F, 0xC4        ], X, PREF_66, SSE2;
    b"yordib"     , [0x0F, 0xC4        ], X, PREF_66, SSE2;
    b"yorwib"     , [0x0F, 0xC4        ], X, PREF_66, SSE2;
]
"pmachriw" = [
    b"xqmq"       , [0x0F, 0x5E        ], X, DEFAULT, MMX | CYRIX;
]
"pmaddubsw" = [
    b"xquq"       , [0x0F, 0x38, 0x04  ], X, DEFAULT, MMX | SSSE3;
    b"yomq"       , [0x0F, 0x38, 0x04  ], X, PREF_66, SSSE3;
    b"yoyo"       , [0x0F, 0x38, 0x04  ], X, PREF_66, SSSE3;
]
"pmaddwd" = [
    b"xquq"       , [0x0F, 0xF5        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xF5        ], X, PREF_66, SSE2;
]
"pmagw" = [
    b"xquq"       , [0x0F, 0x52        ], X, DEFAULT, CYRIX | MMX;
]
"pmaxsb" = [
    b"yomq"       , [0x0F, 0x38, 0x3C  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x3C  ], X, PREF_66, SSE41;
]
"pmaxsd" = [
    b"yomq"       , [0x0F, 0x38, 0x3D  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x3D  ], X, PREF_66, SSE41;
]
"pmaxsw" = [
    b"xquq"       , [0x0F, 0xEE        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xEE        ], X, PREF_66, SSE2;
]
"pmaxub" = [
    b"xquq"       , [0x0F, 0xDE        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xDE        ], X, PREF_66, SSE2;
]
"pmaxud" = [
    b"yomq"       , [0x0F, 0x38, 0x3F  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x3F  ], X, PREF_66, SSE41;
]
"pmaxuw" = [
    b"yomq"       , [0x0F, 0x38, 0x3E  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x3E  ], X, PREF_66, SSE41;
]
"pminsb" = [
    b"yomq"       , [0x0F, 0x38, 0x38  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x38  ], X, PREF_66, SSE41;
]
"pminsd" = [
    b"yomq"       , [0x0F, 0x38, 0x39  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x39  ], X, PREF_66, SSE41;
]
"pminsw" = [
    b"xquq"       , [0x0F, 0xEA        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xEA        ], X, PREF_66, SSE2;
]
"pminub" = [
    b"xquq"       , [0x0F, 0xDA        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xDA        ], X, PREF_66, SSE2;
]
"pminud" = [
    b"yomq"       , [0x0F, 0x38, 0x3B  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x3B  ], X, PREF_66, SSE41;
]
"pminuw" = [
    b"yomq"       , [0x0F, 0x38, 0x3A  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x3A  ], X, PREF_66, SSE41;
]
"pmovmskb" = [
    b"rdxq"       , [0x0F, 0xD7        ], X, DEFAULT, MMX;
    b"rdyo"       , [0x0F, 0xD7        ], X, PREF_66, SSE2;
]
"pmovsxbd" = [
    b"yomd"       , [0x0F, 0x38, 0x21  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x21  ], X, PREF_66, SSE41;
]
"pmovsxbq" = [
    b"yomw"       , [0x0F, 0x38, 0x22  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x22  ], X, PREF_66, SSE41;
]
"pmovsxbw" = [
    b"yomq"       , [0x0F, 0x38, 0x20  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x20  ], X, PREF_66, SSE41;
]
"pmovsxdq" = [
    b"yomq"       , [0x0F, 0x38, 0x25  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x25  ], X, PREF_66, SSE41;
]
"pmovsxwd" = [
    b"yomq"       , [0x0F, 0x38, 0x23  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x23  ], X, PREF_66, SSE41;
]
"pmovsxwq" = [
    b"yomd"       , [0x0F, 0x38, 0x24  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x24  ], X, PREF_66, SSE41;
]
"pmovzxbd" = [
    b"yomd"       , [0x0F, 0x38, 0x31  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x31  ], X, PREF_66, SSE41;
]
"pmovzxbq" = [
    b"yomw"       , [0x0F, 0x38, 0x32  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x32  ], X, PREF_66, SSE41;
]
"pmovzxbw" = [
    b"yomq"       , [0x0F, 0x38, 0x30  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x30  ], X, PREF_66, SSE41;
]
"pmovzxdq" = [
    b"yomq"       , [0x0F, 0x38, 0x35  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x35  ], X, PREF_66, SSE41;
]
"pmovzxwd" = [
    b"yomq"       , [0x0F, 0x38, 0x33  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x33  ], X, PREF_66, SSE41;
]
"pmovzxwq" = [
    b"yomd"       , [0x0F, 0x38, 0x34  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x34  ], X, PREF_66, SSE41;
]
"pmuldq" = [
    b"yomq"       , [0x0F, 0x38, 0x28  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x28  ], X, PREF_66, SSE41;
]
"pmulhriw" = [
    b"xquq"       , [0x0F, 0x5D        ], X, DEFAULT, CYRIX | MMX;
]
"pmulhrsw" = [
    b"xquq"       , [0x0F, 0x38, 0x0B  ], X, DEFAULT, MMX | SSSE3;
    b"yomq"       , [0x0F, 0x38, 0x0B  ], X, PREF_66, SSSE3;
    b"yoyo"       , [0x0F, 0x38, 0x0B  ], X, PREF_66, SSSE3;
]
"pmulhrwa" = [
    b"xquq"       , [0x0F, 0x0F, 0xB7  ], X, IMM_OP, TDNOW;
]
"pmulhrwc" = [
    b"xquq"       , [0x0F, 0x59        ], X, DEFAULT, MMX | CYRIX;
]
"pmulhuw" = [
    b"xquq"       , [0x0F, 0xE4        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xE4        ], X, PREF_66, SSE2;
]
"pmulhw" = [
    b"xquq"       , [0x0F, 0xE5        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xE5        ], X, PREF_66, SSE2;
]
"pmulld" = [
    b"yomq"       , [0x0F, 0x38, 0x40  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x40  ], X, PREF_66, SSE41;
]
"pmullw" = [
    b"xquq"       , [0x0F, 0xD5        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xD5        ], X, PREF_66, SSE2;
]
"pmuludq" = [
    b"xquq"       , [0x0F, 0xF4        ], X, DEFAULT, SSE2;
    b"yowo"       , [0x0F, 0xF4        ], X, PREF_66, SSE2;
]
"pmvgezb" = [
    b"xqmq"       , [0x0F, 0x5C        ], X, DEFAULT, CYRIX | MMX;
]
"pmvlzb" = [
    b"xqmq"       , [0x0F, 0x5B        ], X, DEFAULT, CYRIX | MMX;
]
"pmvnzb" = [
    b"xqmq"       , [0x0F, 0x5A        ], X, DEFAULT, CYRIX | MMX;
]
"pmvzb" = [
    b"xqmq"       , [0x0F, 0x58        ], X, DEFAULT, MMX | CYRIX;
]
"pop" = [
    b"Qw"         , [0x07              ], X, X86_ONLY;
    b"Sw"         , [0x17              ], X, X86_ONLY;
    b"Tw"         , [0x1F              ], X, X86_ONLY;
    b"Uw"         , [0x0F, 0xA1        ], X;
    b"Vw"         , [0x0F, 0xA9        ], X;
    b"r*"         , [0x58              ], X, AUTO_NO32 | SHORT_ARG;
    b"v*"         , [0x8F              ], 0, AUTO_NO32;
]
"popa" = [
    b""           , [0x61              ], X, X86_ONLY | WORD_SIZE;
]
"popad" = [
    b""           , [0x61              ], X, X86_ONLY;
]
"popcnt" = [
    b"r*v*"       , [0x0F, 0xB8        ], X, AUTO_SIZE | PREF_F3;
]
"popf" = [
    b""           , [0x9D              ], X;
]
"popfq" = [
    b""           , [0x9D              ], X;
]
"popfw" = [
    b""           , [0x9D              ], X, WORD_SIZE;
]
"por" = [
    b"xquq"       , [0x0F, 0xEB        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xEB        ], X, PREF_66, SSE2;
]
"prefetch" = [
    b"mq"         , [0x0F, 0x0D        ], 0, DEFAULT, TDNOW;
]
"prefetchnta" = [
    b"mb"         , [0x0F, 0x18        ], 0;
]
"prefetcht0" = [
    b"mb"         , [0x0F, 0x18        ], 1;
]
"prefetcht1" = [
    b"mb"         , [0x0F, 0x18        ], 2;
]
"prefetcht2" = [
    b"mb"         , [0x0F, 0x18        ], 3;
]
"prefetchw" = [
    b"mq"         , [0x0F, 0x0D        ], 1, DEFAULT, TDNOW;
]
"prefetchwt1" = [
    b"mb"         , [0x0F, 0x0D        ], 2, DEFAULT, PREFETCHWT1;
]
"psadbw" = [
    b"xquq"       , [0x0F, 0xF6        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xF6        ], X, PREF_66, SSE2;
]
"pshufb" = [
    b"xquq"       , [0x0F, 0x38, 0x00  ], X, DEFAULT, SSSE3 | MMX;
    b"yomq"       , [0x0F, 0x38, 0x00  ], X, PREF_66, SSSE3;
    b"yoyo"       , [0x0F, 0x38, 0x00  ], X, PREF_66, SSSE3;
]
"pshufd" = [
    b"yowoib"     , [0x0F, 0x70        ], X, PREF_66, SSE2;
]
"pshufhw" = [
    b"yowoib"     , [0x0F, 0x70        ], X, PREF_F3, SSE2;
]
"pshuflw" = [
    b"yowoib"     , [0x0F, 0x70        ], X, PREF_F2, SSE2;
]
"pshufw" = [
    b"xquqib"     , [0x0F, 0x70        ], X, DEFAULT, MMX;
]
"psignb" = [
    b"xquq"       , [0x0F, 0x38, 0x08  ], X, DEFAULT, MMX | SSSE3;
    b"yomq"       , [0x0F, 0x38, 0x08  ], X, PREF_66, SSSE3;
    b"yoyo"       , [0x0F, 0x38, 0x08  ], X, PREF_66, SSSE3;
]
"psignd" = [
    b"xquq"       , [0x0F, 0x38, 0x0A  ], X, DEFAULT, SSSE3 | MMX;
    b"yomq"       , [0x0F, 0x38, 0x0A  ], X, PREF_66, SSSE3;
    b"yoyo"       , [0x0F, 0x38, 0x0A  ], X, PREF_66, SSSE3;
]
"psignw" = [
    b"xquq"       , [0x0F, 0x38, 0x09  ], X, DEFAULT, MMX | SSSE3;
    b"yomq"       , [0x0F, 0x38, 0x09  ], X, PREF_66, SSSE3;
    b"yoyo"       , [0x0F, 0x38, 0x09  ], X, PREF_66, SSSE3;
]
"pslld" = [
    b"xqib"       , [0x0F, 0x72        ], 6, DEFAULT, MMX;
    b"xquq"       , [0x0F, 0xF2        ], X, DEFAULT, MMX;
    b"yoib"       , [0x0F, 0x72        ], 6, PREF_66, SSE2;
    b"yowo"       , [0x0F, 0xF2        ], X, PREF_66, SSE2;
]
"pslldq" = [
    b"yoib"       , [0x0F, 0x73        ], 7, PREF_66, SSE2;
]
"psllq" = [
    b"xqib"       , [0x0F, 0x73        ], 6, DEFAULT, MMX;
    b"xquq"       , [0x0F, 0xF3        ], X, DEFAULT, MMX;
    b"yoib"       , [0x0F, 0x73        ], 6, PREF_66, SSE2;
    b"yowo"       , [0x0F, 0xF3        ], X, PREF_66, SSE2;
]
"psllw" = [
    b"xqib"       , [0x0F, 0x71        ], 6, DEFAULT, MMX;
    b"xquq"       , [0x0F, 0xF1        ], X, DEFAULT, MMX;
    b"yoib"       , [0x0F, 0x71        ], 6, PREF_66, SSE2;
    b"yowo"       , [0x0F, 0xF1        ], X, PREF_66, SSE2;
]
"psrad" = [
    b"xqib"       , [0x0F, 0x72        ], 4, DEFAULT, MMX;
    b"xquq"       , [0x0F, 0xE2        ], X, DEFAULT, MMX;
    b"yoib"       , [0x0F, 0x72        ], 4, PREF_66, SSE2;
    b"yowo"       , [0x0F, 0xE2        ], X, PREF_66, SSE2;
]
"psraw" = [
    b"xqib"       , [0x0F, 0x71        ], 4, DEFAULT, MMX;
    b"xquq"       , [0x0F, 0xE1        ], X, DEFAULT, MMX;
    b"yoib"       , [0x0F, 0x71        ], 4, PREF_66, SSE2;
    b"yowo"       , [0x0F, 0xE1        ], X, PREF_66, SSE2;
]
"psrld" = [
    b"xqib"       , [0x0F, 0x72        ], 2, DEFAULT, MMX;
    b"xquq"       , [0x0F, 0xD2        ], X, DEFAULT, MMX;
    b"yoib"       , [0x0F, 0x72        ], 2, PREF_66, SSE2;
    b"yowo"       , [0x0F, 0xD2        ], X, PREF_66, SSE2;
]
"psrldq" = [
    b"yoib"       , [0x0F, 0x73        ], 3, PREF_66, SSE2;
]
"psrlq" = [
    b"xqib"       , [0x0F, 0x73        ], 2, DEFAULT, MMX;
    b"xquq"       , [0x0F, 0xD3        ], X, DEFAULT, MMX;
    b"yoib"       , [0x0F, 0x73        ], 2, PREF_66, SSE2;
    b"yowo"       , [0x0F, 0xD3        ], X, PREF_66, SSE2;
]
"psrlw" = [
    b"xqib"       , [0x0F, 0x71        ], 2, DEFAULT, MMX;
    b"xquq"       , [0x0F, 0xD1        ], X, DEFAULT, MMX;
    b"yoib"       , [0x0F, 0x71        ], 2, PREF_66, SSE2;
    b"yowo"       , [0x0F, 0xD1        ], X, PREF_66, SSE2;
]
"psubb" = [
    b"xquq"       , [0x0F, 0xF8        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xF8        ], X, PREF_66, SSE2;
]
"psubd" = [
    b"xquq"       , [0x0F, 0xFA        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xFA        ], X, PREF_66, SSE2;
]
"psubq" = [
    b"xquq"       , [0x0F, 0xFB        ], X, DEFAULT, SSE2;
    b"yowo"       , [0x0F, 0xFB        ], X, PREF_66, SSE2;
]
"psubsb" = [
    b"xquq"       , [0x0F, 0xE8        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xE8        ], X, PREF_66, SSE2;
]
"psubsiw" = [
    b"xquq"       , [0x0F, 0x55        ], X, DEFAULT, CYRIX | MMX;
]
"psubsw" = [
    b"xquq"       , [0x0F, 0xE9        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xE9        ], X, PREF_66, SSE2;
]
"psubusb" = [
    b"xquq"       , [0x0F, 0xD8        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xD8        ], X, PREF_66, SSE2;
]
"psubusw" = [
    b"xquq"       , [0x0F, 0xD9        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xD9        ], X, PREF_66, SSE2;
]
"psubw" = [
    b"xquq"       , [0x0F, 0xF9        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xF9        ], X, PREF_66, SSE2;
]
"pswapd" = [
    b"xquq"       , [0x0F, 0x0F, 0xBB  ], X, IMM_OP, TDNOW;
]
"ptest" = [
    b"yomq"       , [0x0F, 0x38, 0x17  ], X, PREF_66, SSE41;
    b"yoyo"       , [0x0F, 0x38, 0x17  ], X, PREF_66, SSE41;
]
"punpckhbw" = [
    b"xquq"       , [0x0F, 0x68        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0x68        ], X, PREF_66, SSE2;
]
"punpckhdq" = [
    b"xquq"       , [0x0F, 0x6A        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0x6A        ], X, PREF_66, SSE2;
]
"punpckhqdq" = [
    b"yowo"       , [0x0F, 0x6D        ], X, PREF_66, SSE2;
]
"punpckhwd" = [
    b"xquq"       , [0x0F, 0x69        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0x69        ], X, PREF_66, SSE2;
]
"punpcklbw" = [
    b"xquq"       , [0x0F, 0x60        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0x60        ], X, PREF_66, SSE2;
]
"punpckldq" = [
    b"xquq"       , [0x0F, 0x62        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0x62        ], X, PREF_66, SSE2;
]
"punpcklqdq" = [
    b"yowo"       , [0x0F, 0x6C        ], X, PREF_66, SSE2;
]
"punpcklwd" = [
    b"xquq"       , [0x0F, 0x61        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0x61        ], X, PREF_66, SSE2;
]
"push" = [
    b"Qw"         , [0x06              ], X, X86_ONLY;
    b"Rw"         , [0x0E              ], X, X86_ONLY;
    b"Sw"         , [0x16              ], X, X86_ONLY;
    b"Tw"         , [0x1E              ], X, X86_ONLY;
    b"Uw"         , [0x0F, 0xA0        ], X;
    b"Vw"         , [0x0F, 0xA8        ], X;
    b"ib"         , [0x6A              ], X, EXACT_SIZE;
    b"iw"         , [0x68              ], X, EXACT_SIZE | WORD_SIZE;
    b"id"         , [0x68              ], X;
    b"r*"         , [0x50              ], X, AUTO_NO32 | SHORT_ARG;
    b"v*"         , [0xFF              ], 6, AUTO_NO32;
]
"pusha" = [
    b""           , [0x60              ], X, X86_ONLY | WORD_SIZE;
]
"pushad" = [
    b""           , [0x60              ], X, X86_ONLY;
]
"pushf" = [
    b""           , [0x9C              ], X;
]
"pushfq" = [
    b""           , [0x9C              ], X;
]
"pushfw" = [
    b""           , [0x9C              ], X, WORD_SIZE;
]
"pxor" = [
    b"xquq"       , [0x0F, 0xEF        ], X, DEFAULT, MMX;
    b"yowo"       , [0x0F, 0xEF        ], X, PREF_66, SSE2;
]
"rcl" = [
    b"vbBb"       , [0xD2              ], 2;
    b"vbib"       , [0xC0              ], 2;
    b"v*Bb"       , [0xD3              ], 2, AUTO_SIZE;
    b"v*ib"       , [0xC1              ], 2, AUTO_SIZE;
]
"rcpps" = [
    b"yowo"       , [0x0F, 0x53        ], X, DEFAULT, SSE;
]
"rcpss" = [
    b"yomd"       , [0x0F, 0x53        ], X, PREF_F3, SSE;
    b"yoyo"       , [0x0F, 0x53        ], X, PREF_F3, SSE;
]
"rcr" = [
    b"vbBb"       , [0xD2              ], 3;
    b"vbib"       , [0xC0              ], 3;
    b"v*Bb"       , [0xD3              ], 3, AUTO_SIZE;
    b"v*ib"       , [0xC1              ], 3, AUTO_SIZE;
]
"rdfsbase" = [
    b"rd"         , [0x0F, 0xAE        ], 0, PREF_F3;
    b"rq"         , [0x0F, 0xAE        ], 0, WITH_REXW | PREF_F3;
]
"rdgsbase" = [
    b"rd"         , [0x0F, 0xAE        ], 1, PREF_F3;
    b"rq"         , [0x0F, 0xAE        ], 1, WITH_REXW | PREF_F3;
]
"rdm" = [
    b""           , [0x0F, 0x3A        ], X, DEFAULT, CYRIX;
]
"rdmsr" = [
    b""           , [0x0F, 0x32        ], X;
]
"rdpid" = [
    b"rq"         , [0x0F, 0xC7        ], 7, PREF_F3;
]
"rdpkru" = [
    b""           , [0x0F, 0x01, 0xEE  ], X;
]
"rdpmc" = [
    b""           , [0x0F, 0x33        ], X;
]
"rdrand" = [
    b"rq"         , [0x0F, 0xC7        ], 6, WITH_REXW;
]
"rdseed" = [
    b"rq"         , [0x0F, 0xC7        ], 7, WITH_REXW;
]
"rdshr" = [
    b"vd"         , [0x0F, 0x36        ], 0, DEFAULT, CYRIX;
]
"rdtsc" = [
    b""           , [0x0F, 0x31        ], X;
]
"rdtscp" = [
    b""           , [0x0F, 0x01, 0xF9  ], X;
]
"ret" = [
    b""           , [0xC3              ], X;
    b"iw"         , [0xC2              ], X;
]
"retf" = [
    b""           , [0xCB              ], X;
    b"iw"         , [0xCA              ], X;
]
"retn" = [
    b""           , [0xC3              ], X;
    b"iw"         , [0xC2              ], X;
]
"rol" = [
    b"vbBb"       , [0xD2              ], 0;
    b"vbib"       , [0xC0              ], 0;
    b"v*Bb"       , [0xD3              ], 0, AUTO_SIZE;
    b"v*ib"       , [0xC1              ], 0, AUTO_SIZE;
]
"ror" = [
    b"vbBb"       , [0xD2              ], 1;
    b"vbib"       , [0xC0              ], 1;
    b"v*Bb"       , [0xD3              ], 1, AUTO_SIZE;
    b"v*ib"       , [0xC1              ], 1, AUTO_SIZE;
]
"rorx" = [
    b"r*v*ib"     , [0x03, 0xF0        ], X, VEX_OP | AUTO_REXW | PREF_F2, BMI2;
]
"roundpd" = [
    b"yomqib"     , [0x0F, 0x3A, 0x09  ], X, PREF_66, SSE41;
    b"yoyoib"     , [0x0F, 0x3A, 0x09  ], X, PREF_66, SSE41;
]
"roundps" = [
    b"yomqib"     , [0x0F, 0x3A, 0x08  ], X, PREF_66, SSE41;
    b"yoyoib"     , [0x0F, 0x3A, 0x08  ], X, PREF_66, SSE41;
]
"roundsd" = [
    b"yomqib"     , [0x0F, 0x3A, 0x0B  ], X, PREF_66, SSE41;
    b"yoyoib"     , [0x0F, 0x3A, 0x0B  ], X, PREF_66, SSE41;
]
"roundss" = [
    b"yomqib"     , [0x0F, 0x3A, 0x0A  ], X, PREF_66, SSE41;
    b"yoyoib"     , [0x0F, 0x3A, 0x0A  ], X, PREF_66, SSE41;
]
"rsdc" = [
    b"swmp"       , [0x0F, 0x79        ], X, EXACT_SIZE, CYRIX;
]
"rsldt" = [
    b"mp"         , [0x0F, 0x7B        ], 0, EXACT_SIZE, CYRIX;
]
"rsm" = [
    b""           , [0x0F, 0xAA        ], X;
]
"rsqrtps" = [
    b"yowo"       , [0x0F, 0x52        ], X, DEFAULT, SSE;
]
"rsqrtss" = [
    b"yomd"       , [0x0F, 0x52        ], X, PREF_F3, SSE;
    b"yoyo"       , [0x0F, 0x52        ], X, PREF_F3, SSE;
]
"rsts" = [
    b"mp"         , [0x0F, 0x7D        ], 0, EXACT_SIZE, CYRIX;
]
"sahf" = [
    b""           , [0x9E              ], X;
]
"sal" = [
    b"vbBb"       , [0xD2              ], 4;
    b"vbib"       , [0xC0              ], 4;
    b"v*Bb"       , [0xD3              ], 4, AUTO_SIZE;
    b"v*ib"       , [0xC1              ], 4, AUTO_SIZE;
]
"sar" = [
    b"vbBb"       , [0xD2              ], 7;
    b"vbib"       , [0xC0              ], 7;
    b"v*Bb"       , [0xD3              ], 7, AUTO_SIZE;
    b"v*ib"       , [0xC1              ], 7, AUTO_SIZE;
]
"sarx" = [
    b"r*v*r*"     , [0x02, 0xF7        ], X, VEX_OP | AUTO_REXW | ENC_MR | PREF_F3, BMI2;
]
"sbb" = [
    b"Abib"       , [0x1C              ], X;
    b"mbib"       , [0x80              ], 3, LOCK;
    b"mbrb"       , [0x18              ], X, LOCK | ENC_MR;
    b"rbib"       , [0x80              ], 3;
    b"rbrb"       , [0x18              ], X, ENC_MR;
    b"rbvb"       , [0x1A              ], X;
    b"r*ib"       , [0x83              ], 3, AUTO_SIZE  | EXACT_SIZE;
    b"A*i*"       , [0x1D              ], X, AUTO_SIZE;
    b"m*i*"       , [0x81              ], 3, AUTO_SIZE | LOCK;
    b"m*ib"       , [0x83              ], 3, AUTO_SIZE | LOCK;
    b"m*r*"       , [0x19              ], X, AUTO_SIZE | LOCK | ENC_MR;
    b"r*i*"       , [0x81              ], 3, AUTO_SIZE ;
    b"r*r*"       , [0x19              ], X, AUTO_SIZE | ENC_MR;
    b"r*v*"       , [0x1B              ], X, AUTO_SIZE;
]
"scasb" = [
    b""           , [0xAE              ], X, REPE;
]
"scasd" = [
    b""           , [0xAF              ], X, REPE;
]
"scasq" = [
    b""           , [0xAF              ], X, REPE | WITH_REXW;
]
"scasw" = [
    b""           , [0xAF              ], X, REPE | WORD_SIZE;
]
"sfence" = [
    b""           , [0x0F, 0xAE, 0xF8  ], X, DEFAULT, AMD;
]
"sgdt" = [
    b"m!"         , [0x0F, 0x01        ], 0;
]
"sha1msg1" = [
    b"yowo"       , [0x0F, 0x38, 0xC9  ], X, DEFAULT, SHA;
]
"sha1msg2" = [
    b"yowo"       , [0x0F, 0x38, 0xCA  ], X, DEFAULT, SHA;
]
"sha1nexte" = [
    b"yowo"       , [0x0F, 0x38, 0xC8  ], X, DEFAULT, SHA;
]
"sha1rnds4" = [
    b"yowoib"     , [0x0F, 0x3A, 0xCC  ], X, DEFAULT, SHA;
]
"sha256msg1" = [
    b"yowo"       , [0x0F, 0x38, 0xCC  ], X, DEFAULT, SHA;
]
"sha256msg2" = [
    b"yowo"       , [0x0F, 0x38, 0xCD  ], X, DEFAULT, SHA;
]
"sha256rnds2" = [
    b"yowo"       , [0x0F, 0x38, 0xCB  ], X, DEFAULT, SHA;
]
"shl" = [
    b"vbBb"       , [0xD2              ], 4;
    b"vbib"       , [0xC0              ], 4;
    b"v*Bb"       , [0xD3              ], 4, AUTO_SIZE;
    b"v*ib"       , [0xC1              ], 4, AUTO_SIZE;
]
"shld" = [
    b"v*r*Bb"     , [0x0F, 0xA5        ], X, AUTO_SIZE | ENC_MR;
    b"v*r*ib"     , [0x0F, 0xA4        ], X, AUTO_SIZE | ENC_MR;
]
"shlx" = [
    b"r*v*r*"     , [0x02, 0xF7        ], X, VEX_OP | AUTO_REXW | ENC_MR | PREF_66, BMI2;
]
"shr" = [
    b"vbBb"       , [0xD2              ], 5;
    b"vbib"       , [0xC0              ], 5;
    b"v*Bb"       , [0xD3              ], 5, AUTO_SIZE;
    b"v*ib"       , [0xC1              ], 5, AUTO_SIZE;
]
"shrd" = [
    b"v*r*Bb"     , [0x0F, 0xAD        ], X, AUTO_SIZE | ENC_MR;
    b"v*r*ib"     , [0x0F, 0xAC        ], X, AUTO_SIZE | ENC_MR;
]
"shrx" = [
    b"r*v*r*"     , [0x02, 0xF7        ], X, VEX_OP | AUTO_REXW | ENC_MR | PREF_F2, BMI2;
]
"shufpd" = [
    b"yowoib"     , [0x0F, 0xC6        ], X, PREF_66, SSE2;
]
"shufps" = [
    b"yowoib"     , [0x0F, 0xC6        ], X, DEFAULT, SSE;
]
"sidt" = [
    b"m!"         , [0x0F, 0x01        ], 1;
]
"skinit" = [
    b""           , [0x0F, 0x01, 0xDE  ], X;
]
"sldt" = [
    b"m!"         , [0x0F, 0x00        ], 0;
    b"r*"         , [0x0F, 0x00        ], 0, AUTO_SIZE;
]
"slwpcb" = [
    b"r*"         , [0x09, 0x12        ], 1, XOP_OP | AUTO_REXW, AMD;
]
"smint" = [
    b""           , [0x0F, 0x38        ], X, DEFAULT, CYRIX;
]
"smsw" = [
    b"m!"         , [0x0F, 0x01        ], 4;
    b"r*"         , [0x0F, 0x01        ], 4, AUTO_SIZE;
]
"sqrtpd" = [
    b"yowo"       , [0x0F, 0x51        ], X, PREF_66, SSE2;
]
"sqrtps" = [
    b"yowo"       , [0x0F, 0x51        ], X, DEFAULT, SSE;
]
"sqrtsd" = [
    b"yomq"       , [0x0F, 0x51        ], X, PREF_F2, SSE2;
    b"yoyo"       , [0x0F, 0x51        ], X, PREF_F2, SSE2;
]
"sqrtss" = [
    b"yomd"       , [0x0F, 0x51        ], X, PREF_F3, SSE;
    b"yoyo"       , [0x0F, 0x51        ], X, PREF_F3, SSE;
]
"stac" = [
    b""           , [0x0F, 0x01, 0xCB  ], X;
]
"stc" = [
    b""           , [0xF9              ], X;
]
"std" = [
    b""           , [0xFD              ], X;
]
"stgi" = [
    b""           , [0x0F, 0x01, 0xDC  ], X, DEFAULT, VMX | AMD;
]
"sti" = [
    b""           , [0xFB              ], X;
]
"stmxcsr" = [
    b"md"         , [0x0F, 0xAE        ], 3, DEFAULT, SSE;
]
"stosb" = [
    b""           , [0xAA              ], X, REP;
]
"stosd" = [
    b""           , [0xAB              ], X, REP;
]
"stosq" = [
    b""           , [0xAB              ], X, WITH_REXW | REP;
]
"stosw" = [
    b""           , [0xAB              ], X, WORD_SIZE | REP;
]
"str" = [
    b"m!"         , [0x0F, 0x00        ], 1;
    b"r*"         , [0x0F, 0x00        ], 1, AUTO_SIZE;
]
"sub" = [
    b"Abib"       , [0x2C              ], X;
    b"mbib"       , [0x80              ], 5, LOCK;
    b"mbrb"       , [0x28              ], X, LOCK | ENC_MR;
    b"rbib"       , [0x80              ], 5;
    b"rbrb"       , [0x28              ], X, ENC_MR;
    b"rbvb"       , [0x2A              ], X;
    b"r*ib"       , [0x83              ], 5, AUTO_SIZE  | EXACT_SIZE;
    b"A*i*"       , [0x2D              ], X, AUTO_SIZE;
    b"m*i*"       , [0x81              ], 5, AUTO_SIZE | LOCK;
    b"m*ib"       , [0x83              ], 5, AUTO_SIZE | LOCK;
    b"m*r*"       , [0x29              ], X, AUTO_SIZE | LOCK | ENC_MR;
    b"r*i*"       , [0x81              ], 5, AUTO_SIZE ;
    b"r*r*"       , [0x29              ], X, AUTO_SIZE | ENC_MR;
    b"r*v*"       , [0x2B              ], X, AUTO_SIZE;
]
"subpd" = [
    b"yowo"       , [0x0F, 0x5C        ], X, PREF_66, SSE2;
]
"subps" = [
    b"yowo"       , [0x0F, 0x5C        ], X, DEFAULT, SSE;
]
"subsd" = [
    b"yomq"       , [0x0F, 0x5C        ], X, PREF_F2, SSE2;
    b"yoyo"       , [0x0F, 0x5C        ], X, PREF_F2, SSE2;
]
"subss" = [
    b"yomd"       , [0x0F, 0x5C        ], X, PREF_F3, SSE;
    b"yoyo"       , [0x0F, 0x5C        ], X, PREF_F3, SSE;
]
"svdc" = [
    b"mpsw"       , [0x0F, 0x78        ], X, ENC_MR | EXACT_SIZE, CYRIX;
]
"svldt" = [
    b"mp"         , [0x0F, 0x7A        ], 0, EXACT_SIZE, CYRIX;
]
"svts" = [
    b"mp"         , [0x0F, 0x7C        ], 0, EXACT_SIZE, CYRIX;
]
"swapgs" = [
    b""           , [0x0F, 0x01, 0xF8  ], X;
]
"syscall" = [
    b""           , [0x0F, 0x05        ], X, DEFAULT, AMD;
]
"sysenter" = [
    b""           , [0x0F, 0x34        ], X, X86_ONLY;
]
"sysexit" = [
    b""           , [0x0F, 0x35        ], X, X86_ONLY;
]
"sysret" = [
    b""           , [0x0F, 0x07        ], X, DEFAULT, AMD;
]
"t1mskc" = [
    b"r*v*"       , [0x09, 0x01        ], 7, XOP_OP | AUTO_REXW | ENC_VM, TBM;
]
"test" = [
    b"Abib"       , [0xA8              ], X;
    b"rbmb"       , [0x84              ], X;
    b"vbib"       , [0xF6              ], 0;
    b"vbrb"       , [0x84              ], X, ENC_MR;
    b"A*i*"       , [0xA9              ], X, AUTO_SIZE;
    b"r*m*"       , [0x85              ], X, AUTO_SIZE;
    b"v*i*"       , [0xF7              ], 0, AUTO_SIZE;
    b"v*r*"       , [0x85              ], X, AUTO_SIZE | ENC_MR;
]
"tzcnt" = [
    b"r*v*"       , [0x0F, 0xBC        ], X, AUTO_SIZE | PREF_F3, BMI1;
]
"tzmsk" = [
    b"r*v*"       , [0x09, 0x01        ], 4, XOP_OP | AUTO_REXW | ENC_VM, TBM;
]
"ucomisd" = [
    b"yomq"       , [0x0F, 0x2E        ], X, PREF_66, SSE2;
    b"yoyo"       , [0x0F, 0x2E        ], X, PREF_66, SSE2;
]
"ucomiss" = [
    b"yomd"       , [0x0F, 0x2E        ], X, DEFAULT, SSE;
    b"yoyo"       , [0x0F, 0x2E        ], X, DEFAULT, SSE;
]
"ud1" = [
    b"rdvd"       , [0x0F, 0xB9        ], X, EXACT_SIZE;
]
"ud2" = [
    b""           , [0x0F, 0x0B        ], X;
]
"ud2a" = [
    b""           , [0x0F, 0x0B        ], X;
]
"unpckhpd" = [
    b"yowo"       , [0x0F, 0x15        ], X, PREF_66, SSE2;
]
"unpckhps" = [
    b"yowo"       , [0x0F, 0x15        ], X, DEFAULT, SSE;
]
"unpcklpd" = [
    b"yowo"       , [0x0F, 0x14        ], X, PREF_66, SSE2;
]
"unpcklps" = [
    b"yowo"       , [0x0F, 0x14        ], X, DEFAULT, SSE;
]
"vaddpd" = [
    b"y*y*w*"     , [0x01, 0x58        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vaddps" = [
    b"y*y*w*"     , [0x01, 0x58        ], X, VEX_OP | AUTO_VEXL, AVX;
]
"vaddsd" = [
    b"yoyomq"     , [0x01, 0x58        ], X, VEX_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0x58        ], X, VEX_OP | PREF_F2, AVX;
]
"vaddss" = [
    b"yoyomd"     , [0x01, 0x58        ], X, VEX_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0x58        ], X, VEX_OP | PREF_F3, AVX;
]
"vaddsubpd" = [
    b"y*y*w*"     , [0x01, 0xD0        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vaddsubps" = [
    b"y*y*w*"     , [0x01, 0xD0        ], X, VEX_OP | AUTO_VEXL | PREF_F2, AVX;
]
"vaesdec" = [
    b"yoyowo"     , [0x02, 0xDE        ], X, VEX_OP | PREF_66, AVX;
]
"vaesdeclast" = [
    b"yoyowo"     , [0x02, 0xDF        ], X, VEX_OP | PREF_66, AVX;
]
"vaesenc" = [
    b"yoyowo"     , [0x02, 0xDC        ], X, VEX_OP | PREF_66, AVX;
]
"vaesenclast" = [
    b"yoyowo"     , [0x02, 0xDD        ], X, VEX_OP | PREF_66, AVX;
]
"vaesimc" = [
    b"yowo"       , [0x02, 0xDB        ], X, VEX_OP | PREF_66, AVX;
]
"vaeskeygenassist" = [
    b"yowoib"     , [0x03, 0xDF        ], X, VEX_OP | PREF_66, AVX;
]
"vandnpd" = [
    b"y*y*w*"     , [0x01, 0x55        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vandnps" = [
    b"y*y*w*"     , [0x01, 0x55        ], X, VEX_OP | AUTO_VEXL, AVX;
]
"vandpd" = [
    b"y*y*w*"     , [0x01, 0x54        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vandps" = [
    b"y*y*w*"     , [0x01, 0x54        ], X, VEX_OP | AUTO_VEXL, AVX;
]
"vblendpd" = [
    b"y*y*w*ib"   , [0x03, 0x0D        ], X, VEX_OP | AUTO_VEXL | ENC_MR | PREF_66, AVX;
]
"vblendps" = [
    b"y*y*w*ib"   , [0x03, 0x0C        ], X, VEX_OP | AUTO_VEXL | ENC_MR | PREF_66, AVX;
]
"vblendvpd" = [
    b"y*y*w*y*"   , [0x03, 0x4B        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vblendvps" = [
    b"y*y*w*y*"   , [0x03, 0x4A        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vbroadcastf128" = [
    b"yhmo"       , [0x02, 0x1A        ], X, WITH_VEXL | VEX_OP | PREF_66, AVX;
]
"vbroadcasti128" = [
    b"yhmo"       , [0x02, 0x5A        ], X, WITH_VEXL | VEX_OP | PREF_66, AVX2;
]
"vbroadcastsd" = [
    b"yhmq"       , [0x02, 0x19        ], X, VEX_OP | WITH_VEXL | PREF_66, AVX;
    b"yhyo"       , [0x02, 0x19        ], X, VEX_OP | WITH_VEXL | PREF_66, AVX;
]
"vbroadcastss" = [
    b"y*md"       , [0x02, 0x18        ], X, VEX_OP | PREF_66, AVX;
    b"y*yo"       , [0x02, 0x18        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vcmpeq_ospd" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x10  ], X, VEX_OP | PREF_66 | IMM_OP, AVX;
    b"yoyowo"     , [0x01, 0xC2, 0x10  ], X, VEX_OP | IMM_OP | PREF_66, AVX;
]
"vcmpeq_osps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x10  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpeq_ossd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x10  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x10  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
]
"vcmpeq_osss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x10  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x10  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
]
"vcmpeq_uqpd" = [
    b"yhyhwh"     , [0x01, 0xC2, 0x08  ], X, WITH_VEXL | VEX_OP | IMM_OP | PREF_66, AVX;
    b"yoyowo"     , [0x01, 0xC2, 0x08  ], X, VEX_OP | PREF_66 | IMM_OP, AVX;
]
"vcmpeq_uqps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x08  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpeq_uqsd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x08  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x08  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
]
"vcmpeq_uqss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x08  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x08  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
]
"vcmpeq_uspd" = [
    b"yhyhwh"     , [0x01, 0xC2, 0x18  ], X, WITH_VEXL | VEX_OP | IMM_OP | PREF_66, AVX;
    b"yoyowo"     , [0x01, 0xC2, 0x18  ], X, VEX_OP | PREF_66 | IMM_OP, AVX;
]
"vcmpeq_usps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x18  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpeq_ussd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x18  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x18  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
]
"vcmpeq_usss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x18  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x18  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
]
"vcmpeqpd" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x00  ], X, VEX_OP | AUTO_VEXL | IMM_OP | PREF_66, AVX;
]
"vcmpeqps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x00  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpeqsd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x00  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x00  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
]
"vcmpeqss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x00  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x00  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
]
"vcmpfalse_oqpd" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x0B  ], X, VEX_OP | AUTO_VEXL | IMM_OP | PREF_66, AVX;
]
"vcmpfalse_oqps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x0B  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpfalse_oqsd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x0B  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x0B  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
]
"vcmpfalse_oqss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x0B  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x0B  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
]
"vcmpfalse_ospd" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x1B  ], X, VEX_OP | AUTO_VEXL | PREF_66 | IMM_OP, AVX;
]
"vcmpfalse_osps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x1B  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpfalse_ossd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x1B  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x1B  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
]
"vcmpfalse_osss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x1B  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x1B  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
]
"vcmpfalsepd" = [
    b"yhyhwh"     , [0x01, 0xC2, 0x0B  ], X, WITH_VEXL | VEX_OP | PREF_66 | IMM_OP, AVX;
    b"yoyowo"     , [0x01, 0xC2, 0x0B  ], X, VEX_OP | IMM_OP | PREF_66, AVX;
]
"vcmpfalseps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x0B  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpfalsesd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x0B  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x0B  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
]
"vcmpfalsess" = [
    b"yoyomq"     , [0x01, 0xC2, 0x0B  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x0B  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
]
"vcmpge_oqpd" = [
    b"yhyhwh"     , [0x01, 0xC2, 0x1D  ], X, WITH_VEXL | VEX_OP | PREF_66 | IMM_OP, AVX;
    b"yoyowo"     , [0x01, 0xC2, 0x1D  ], X, VEX_OP | IMM_OP | PREF_66, AVX;
]
"vcmpge_oqps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x1D  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpge_oqsd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x1D  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x1D  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
]
"vcmpge_oqss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x1D  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x1D  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
]
"vcmpge_ospd" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x0D  ], X, VEX_OP | AUTO_VEXL | PREF_66 | IMM_OP, AVX;
]
"vcmpge_osps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x0D  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpge_ossd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x0D  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x0D  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
]
"vcmpge_osss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x0D  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x0D  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
]
"vcmpgepd" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x0D  ], X, VEX_OP | AUTO_VEXL | IMM_OP | PREF_66, AVX;
]
"vcmpgeps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x0D  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpgesd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x0D  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x0D  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
]
"vcmpgess" = [
    b"yoyomq"     , [0x01, 0xC2, 0x0D  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x0D  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
]
"vcmpgt_oqpd" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x1E  ], X, VEX_OP | AUTO_VEXL | IMM_OP | PREF_66, AVX;
]
"vcmpgt_oqps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x1E  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpgt_oqsd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x1E  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x1E  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
]
"vcmpgt_oqss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x1E  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x1E  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
]
"vcmpgt_ospd" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x0E  ], X, VEX_OP | AUTO_VEXL | IMM_OP | PREF_66, AVX;
]
"vcmpgt_osps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x0E  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpgt_ossd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x0E  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x0E  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
]
"vcmpgt_osss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x0E  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x0E  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
]
"vcmpgtpd" = [
    b"yhyhwh"     , [0x01, 0xC2, 0x0E  ], X, WITH_VEXL | VEX_OP | PREF_66 | IMM_OP, AVX;
    b"yoyowo"     , [0x01, 0xC2, 0x0E  ], X, VEX_OP | IMM_OP | PREF_66, AVX;
]
"vcmpgtps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x0E  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpgtsd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x0E  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x0E  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
]
"vcmpgtss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x0E  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x0E  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
]
"vcmple_oqpd" = [
    b"yhyhwh"     , [0x01, 0xC2, 0x12  ], X, WITH_VEXL | VEX_OP | IMM_OP | PREF_66, AVX;
    b"yoyowo"     , [0x01, 0xC2, 0x12  ], X, VEX_OP | PREF_66 | IMM_OP, AVX;
]
"vcmple_oqps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x12  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmple_oqsd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x12  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x12  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
]
"vcmple_oqss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x12  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x12  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
]
"vcmple_ospd" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x02  ], X, VEX_OP | AUTO_VEXL | IMM_OP | PREF_66, AVX;
]
"vcmple_osps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x02  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmple_ossd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x02  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x02  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
]
"vcmple_osss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x02  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x02  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
]
"vcmplepd" = [
    b"yhyhwh"     , [0x01, 0xC2, 0x02  ], X, WITH_VEXL | VEX_OP | IMM_OP | PREF_66, AVX;
    b"yoyowo"     , [0x01, 0xC2, 0x02  ], X, VEX_OP | PREF_66 | IMM_OP, AVX;
]
"vcmpleps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x02  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmplesd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x02  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x02  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
]
"vcmpless" = [
    b"yoyomq"     , [0x01, 0xC2, 0x02  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x02  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
]
"vcmplt_oqpd" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x11  ], X, VEX_OP | AUTO_VEXL | PREF_66 | IMM_OP, AVX;
]
"vcmplt_oqps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x11  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmplt_oqsd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x11  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x11  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
]
"vcmplt_oqss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x11  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x11  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
]
"vcmplt_ospd" = [
    b"yhyhwh"     , [0x01, 0xC2, 0x01  ], X, WITH_VEXL | VEX_OP | IMM_OP | PREF_66, AVX;
    b"yoyowo"     , [0x01, 0xC2, 0x01  ], X, VEX_OP | PREF_66 | IMM_OP, AVX;
]
"vcmplt_osps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x01  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmplt_ossd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x01  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x01  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
]
"vcmplt_osss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x01  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x01  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
]
"vcmpltpd" = [
    b"yhyhwh"     , [0x01, 0xC2, 0x01  ], X, WITH_VEXL | VEX_OP | IMM_OP | PREF_66, AVX;
    b"yoyowo"     , [0x01, 0xC2, 0x01  ], X, VEX_OP | PREF_66 | IMM_OP, AVX;
]
"vcmpltps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x01  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpltsd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x01  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x01  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
]
"vcmpltss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x01  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x01  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
]
"vcmpneq_oqpd" = [
    b"yhyhwh"     , [0x01, 0xC2, 0x0C  ], X, WITH_VEXL | VEX_OP | IMM_OP | PREF_66, AVX;
    b"yoyowo"     , [0x01, 0xC2, 0x0C  ], X, VEX_OP | PREF_66 | IMM_OP, AVX;
]
"vcmpneq_oqps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x0C  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpneq_oqsd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x0C  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x0C  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
]
"vcmpneq_oqss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x0C  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x0C  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
]
"vcmpneq_ospd" = [
    b"yhyhwh"     , [0x01, 0xC2, 0x1C  ], X, WITH_VEXL | VEX_OP | PREF_66 | IMM_OP, AVX;
    b"yoyowo"     , [0x01, 0xC2, 0x1C  ], X, VEX_OP | IMM_OP | PREF_66, AVX;
]
"vcmpneq_osps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x1C  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpneq_ossd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x1C  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x1C  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
]
"vcmpneq_osss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x1C  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x1C  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
]
"vcmpneq_uqpd" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x04  ], X, VEX_OP | AUTO_VEXL | PREF_66 | IMM_OP, AVX;
]
"vcmpneq_uqps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x04  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpneq_uqsd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x04  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x04  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
]
"vcmpneq_uqss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x04  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x04  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
]
"vcmpneq_uspd" = [
    b"yhyhwh"     , [0x01, 0xC2, 0x14  ], X, WITH_VEXL | VEX_OP | IMM_OP | PREF_66, AVX;
    b"yoyowo"     , [0x01, 0xC2, 0x14  ], X, VEX_OP | PREF_66 | IMM_OP, AVX;
]
"vcmpneq_usps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x14  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpneq_ussd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x14  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x14  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
]
"vcmpneq_usss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x14  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x14  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
]
"vcmpneqpd" = [
    b"yhyhwh"     , [0x01, 0xC2, 0x04  ], X, WITH_VEXL | VEX_OP | PREF_66 | IMM_OP, AVX;
    b"yoyowo"     , [0x01, 0xC2, 0x04  ], X, VEX_OP | IMM_OP | PREF_66, AVX;
]
"vcmpneqps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x04  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpneqsd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x04  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x04  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
]
"vcmpneqss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x04  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x04  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
]
"vcmpnge_uqpd" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x19  ], X, VEX_OP | AUTO_VEXL | PREF_66 | IMM_OP, AVX;
]
"vcmpnge_uqps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x19  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpnge_uqsd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x19  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x19  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
]
"vcmpnge_uqss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x19  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x19  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
]
"vcmpnge_uspd" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x09  ], X, VEX_OP | AUTO_VEXL | IMM_OP | PREF_66, AVX;
]
"vcmpnge_usps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x09  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpnge_ussd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x09  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x09  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
]
"vcmpnge_usss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x09  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x09  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
]
"vcmpngepd" = [
    b"yhyhwh"     , [0x01, 0xC2, 0x09  ], X, WITH_VEXL | VEX_OP | PREF_66 | IMM_OP, AVX;
    b"yoyowo"     , [0x01, 0xC2, 0x09  ], X, VEX_OP | IMM_OP | PREF_66, AVX;
]
"vcmpngeps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x09  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpngesd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x09  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x09  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
]
"vcmpngess" = [
    b"yoyomq"     , [0x01, 0xC2, 0x09  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x09  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
]
"vcmpngt_uqpd" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x1A  ], X, VEX_OP | AUTO_VEXL | PREF_66 | IMM_OP, AVX;
]
"vcmpngt_uqps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x1A  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpngt_uqsd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x1A  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x1A  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
]
"vcmpngt_uqss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x1A  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x1A  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
]
"vcmpngt_uspd" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x0A  ], X, VEX_OP | AUTO_VEXL | PREF_66 | IMM_OP, AVX;
]
"vcmpngt_usps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x0A  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpngt_ussd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x0A  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x0A  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
]
"vcmpngt_usss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x0A  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x0A  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
]
"vcmpngtpd" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x0A  ], X, VEX_OP | AUTO_VEXL | IMM_OP | PREF_66, AVX;
]
"vcmpngtps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x0A  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpngtsd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x0A  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x0A  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
]
"vcmpngtss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x0A  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x0A  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
]
"vcmpnle_uqpd" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x16  ], X, VEX_OP | AUTO_VEXL | PREF_66 | IMM_OP, AVX;
]
"vcmpnle_uqps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x16  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpnle_uqsd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x16  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x16  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
]
"vcmpnle_uqss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x16  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x16  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
]
"vcmpnle_uspd" = [
    b"yhyhwh"     , [0x01, 0xC2, 0x06  ], X, WITH_VEXL | VEX_OP | IMM_OP | PREF_66, AVX;
    b"yoyowo"     , [0x01, 0xC2, 0x06  ], X, VEX_OP | PREF_66 | IMM_OP, AVX;
]
"vcmpnle_usps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x06  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpnle_ussd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x06  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x06  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
]
"vcmpnle_usss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x06  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x06  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
]
"vcmpnlepd" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x06  ], X, VEX_OP | AUTO_VEXL | IMM_OP | PREF_66, AVX;
]
"vcmpnleps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x06  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpnlesd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x06  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x06  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
]
"vcmpnless" = [
    b"yoyomq"     , [0x01, 0xC2, 0x06  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x06  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
]
"vcmpnlt_uqpd" = [
    b"yhyhwh"     , [0x01, 0xC2, 0x15  ], X, WITH_VEXL | VEX_OP | IMM_OP | PREF_66, AVX;
    b"yoyowo"     , [0x01, 0xC2, 0x15  ], X, VEX_OP | PREF_66 | IMM_OP, AVX;
]
"vcmpnlt_uqps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x15  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpnlt_uqsd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x15  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x15  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
]
"vcmpnlt_uqss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x15  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x15  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
]
"vcmpnlt_uspd" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x05  ], X, VEX_OP | AUTO_VEXL | IMM_OP | PREF_66, AVX;
]
"vcmpnlt_usps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x05  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpnlt_ussd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x05  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x05  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
]
"vcmpnlt_usss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x05  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x05  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
]
"vcmpnltpd" = [
    b"yhyhwh"     , [0x01, 0xC2, 0x05  ], X, WITH_VEXL | VEX_OP | PREF_66 | IMM_OP, AVX;
    b"yoyowo"     , [0x01, 0xC2, 0x05  ], X, VEX_OP | IMM_OP | PREF_66, AVX;
]
"vcmpnltps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x05  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpnltsd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x05  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x05  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
]
"vcmpnltss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x05  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x05  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
]
"vcmpord_qpd" = [
    b"yhyhwh"     , [0x01, 0xC2, 0x07  ], X, WITH_VEXL | VEX_OP | PREF_66 | IMM_OP, AVX;
    b"yoyowo"     , [0x01, 0xC2, 0x07  ], X, VEX_OP | IMM_OP | PREF_66, AVX;
]
"vcmpord_qps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x07  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpord_qsd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x07  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x07  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
]
"vcmpord_qss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x07  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x07  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
]
"vcmpord_spd" = [
    b"yhyhwh"     , [0x01, 0xC2, 0x17  ], X, WITH_VEXL | VEX_OP | PREF_66 | IMM_OP, AVX;
    b"yoyowo"     , [0x01, 0xC2, 0x17  ], X, VEX_OP | IMM_OP | PREF_66, AVX;
]
"vcmpord_sps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x17  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpord_ssd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x17  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x17  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
]
"vcmpord_sss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x17  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x17  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
]
"vcmpordpd" = [
    b"yhyhwh"     , [0x01, 0xC2, 0x07  ], X, WITH_VEXL | VEX_OP | IMM_OP | PREF_66, AVX;
    b"yoyowo"     , [0x01, 0xC2, 0x07  ], X, VEX_OP | PREF_66 | IMM_OP, AVX;
]
"vcmpordps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x07  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpordsd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x07  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x07  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
]
"vcmpordss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x07  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x07  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
]
"vcmppd" = [
    b"y*y*w*ib"   , [0x01, 0xC2        ], X, VEX_OP | AUTO_VEXL | ENC_MR | PREF_66, AVX;
]
"vcmpps" = [
    b"y*y*w*ib"   , [0x01, 0xC2        ], X, VEX_OP | AUTO_VEXL | ENC_MR, AVX;
]
"vcmpsd" = [
    b"yoyomqib"   , [0x01, 0xC2        ], X, VEX_OP | PREF_F2, AVX;
    b"yoyoyoib"   , [0x01, 0xC2        ], X, VEX_OP | PREF_F2, AVX;
]
"vcmpss" = [
    b"yoyomqib"   , [0x01, 0xC2        ], X, VEX_OP | PREF_F3, AVX;
    b"yoyoyoib"   , [0x01, 0xC2        ], X, VEX_OP | PREF_F3, AVX;
]
"vcmptrue_uqpd" = [
    b"yhyhwh"     , [0x01, 0xC2, 0x0F  ], X, WITH_VEXL | VEX_OP | IMM_OP | PREF_66, AVX;
    b"yoyowo"     , [0x01, 0xC2, 0x0F  ], X, VEX_OP | PREF_66 | IMM_OP, AVX;
]
"vcmptrue_uqps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x0F  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmptrue_uqsd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x0F  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x0F  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
]
"vcmptrue_uqss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x0F  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x0F  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
]
"vcmptrue_uspd" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x1F  ], X, VEX_OP | AUTO_VEXL | PREF_66 | IMM_OP, AVX;
]
"vcmptrue_usps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x1F  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmptrue_ussd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x1F  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x1F  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
]
"vcmptrue_usss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x1F  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x1F  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
]
"vcmptruepd" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x0F  ], X, VEX_OP | AUTO_VEXL | PREF_66 | IMM_OP, AVX;
]
"vcmptrueps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x0F  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmptruesd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x0F  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x0F  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
]
"vcmptruess" = [
    b"yoyomq"     , [0x01, 0xC2, 0x0F  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x0F  ], X, VEX_OP | PREF_F3 | IMM_OP, AVX;
]
"vcmpunord_qpd" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x03  ], X, VEX_OP | AUTO_VEXL | PREF_66 | IMM_OP, AVX;
]
"vcmpunord_qps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x03  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpunord_qsd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x03  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x03  ], X, VEX_OP | IMM_OP | PREF_F2, AVX;
]
"vcmpunord_qss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x03  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x03  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
]
"vcmpunord_spd" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x13  ], X, VEX_OP | AUTO_VEXL | IMM_OP | PREF_66, AVX;
]
"vcmpunord_sps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x13  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpunord_ssd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x13  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x13  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
]
"vcmpunord_sss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x13  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x13  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
]
"vcmpunordpd" = [
    b"yhyhwh"     , [0x01, 0xC2, 0x03  ], X, WITH_VEXL | VEX_OP | PREF_66 | IMM_OP, AVX;
    b"yoyowo"     , [0x01, 0xC2, 0x03  ], X, VEX_OP | IMM_OP | PREF_66, AVX;
]
"vcmpunordps" = [
    b"y*y*w*"     , [0x01, 0xC2, 0x03  ], X, VEX_OP | AUTO_VEXL | IMM_OP, AVX;
]
"vcmpunordsd" = [
    b"yoyomq"     , [0x01, 0xC2, 0x03  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x03  ], X, VEX_OP | PREF_F2 | IMM_OP, AVX;
]
"vcmpunordss" = [
    b"yoyomq"     , [0x01, 0xC2, 0x03  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0xC2, 0x03  ], X, VEX_OP | IMM_OP | PREF_F3, AVX;
]
"vcomisd" = [
    b"yomq"       , [0x01, 0x2F        ], X, VEX_OP | PREF_66, AVX;
    b"yoyo"       , [0x01, 0x2F        ], X, VEX_OP | PREF_66, AVX;
]
"vcomiss" = [
    b"yomd"       , [0x01, 0x2F        ], X, VEX_OP, AVX;
    b"yoyo"       , [0x01, 0x2F        ], X, VEX_OP, AVX;
]
"vcvtdq2pd" = [
    b"yomq"       , [0x01, 0xE6        ], X, VEX_OP | PREF_F3, AVX;
    b"y*wo"       , [0x01, 0xE6        ], X, VEX_OP | AUTO_VEXL | PREF_F3, AVX;
]
"vcvtdq2ps" = [
    b"y*w*"       , [0x01, 0x5B        ], X, VEX_OP | AUTO_VEXL, AVX;
]
"vcvtpd2dq" = [
    b"yom*"       , [0x01, 0xE6        ], X, VEX_OP | AUTO_VEXL | PREF_F2, AVX;
    b"yoy*"       , [0x01, 0xE6        ], X, VEX_OP | AUTO_VEXL | PREF_F2, AVX;
]
"vcvtpd2ps" = [
    b"yom*"       , [0x01, 0x5A        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
    b"yoy*"       , [0x01, 0x5A        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vcvtph2ps" = [
    b"yomq"       , [0x02, 0x13        ], X, VEX_OP | PREF_66, AVX;
    b"y*wo"       , [0x02, 0x13        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vcvtps2dq" = [
    b"y*w*"       , [0x01, 0x5B        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vcvtps2pd" = [
    b"yomq"       , [0x01, 0x5A        ], X, VEX_OP, AVX;
    b"y*wo"       , [0x01, 0x5A        ], X, VEX_OP | AUTO_VEXL, AVX;
]
"vcvtps2ph" = [
    b"mqyoib"     , [0x03, 0x1D        ], X, VEX_OP | ENC_MR | PREF_66, AVX;
    b"woy*ib"     , [0x03, 0x1D        ], X, VEX_OP | AUTO_VEXL | ENC_MR | PREF_66, AVX;
]
"vcvtsd2si" = [
    b"r*mq"       , [0x01, 0x2D        ], X, VEX_OP | AUTO_REXW | PREF_F2, AVX;
    b"r*yo"       , [0x01, 0x2D        ], X, VEX_OP | AUTO_REXW | PREF_F2, AVX;
]
"vcvtsd2ss" = [
    b"yoyomq"     , [0x01, 0x5A        ], X, VEX_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0x5A        ], X, VEX_OP | PREF_F2, AVX;
]
"vcvtsi2sd" = [
    b"yoyov*"     , [0x01, 0x2A        ], X, VEX_OP | AUTO_REXW | PREF_F2, AVX;
]
"vcvtsi2ss" = [
    b"yoyov*"     , [0x01, 0x2A        ], X, VEX_OP | AUTO_REXW | PREF_F3, AVX;
]
"vcvtss2sd" = [
    b"yoyomd"     , [0x01, 0x5A        ], X, VEX_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0x5A        ], X, VEX_OP | PREF_F3, AVX;
]
"vcvtss2si" = [
    b"r*md"       , [0x01, 0x2D        ], X, VEX_OP | AUTO_REXW | PREF_F3, AVX;
    b"r*yo"       , [0x01, 0x2D        ], X, VEX_OP | AUTO_REXW | PREF_F3, AVX;
]
"vcvttpd2dq" = [
    b"yom*"       , [0x01, 0xE6        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
    b"yoy*"       , [0x01, 0xE6        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vcvttps2dq" = [
    b"y*w*"       , [0x01, 0x5B        ], X, VEX_OP | AUTO_VEXL | PREF_F3, AVX;
]
"vcvttsd2si" = [
    b"r*mq"       , [0x01, 0x2C        ], X, VEX_OP | AUTO_REXW | PREF_F2, AVX;
    b"r*yo"       , [0x01, 0x2C        ], X, VEX_OP | AUTO_REXW | PREF_F2, AVX;
]
"vcvttss2si" = [
    b"r*md"       , [0x01, 0x2C        ], X, VEX_OP | AUTO_REXW | PREF_F3, AVX;
    b"r*yo"       , [0x01, 0x2C        ], X, VEX_OP | AUTO_REXW | PREF_F3, AVX;
]
"vdivpd" = [
    b"y*y*w*"     , [0x01, 0x5E        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vdivps" = [
    b"y*y*w*"     , [0x01, 0x5E        ], X, VEX_OP | AUTO_VEXL, AVX;
]
"vdivsd" = [
    b"yoyomq"     , [0x01, 0x5E        ], X, VEX_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0x5E        ], X, VEX_OP | PREF_F2, AVX;
]
"vdivss" = [
    b"yoyomd"     , [0x01, 0x5E        ], X, VEX_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0x5E        ], X, VEX_OP | PREF_F3, AVX;
]
"vdppd" = [
    b"yoyowoib"   , [0x03, 0x41        ], X, VEX_OP | PREF_66, AVX;
]
"vdpps" = [
    b"y*y*w*ib"   , [0x03, 0x40        ], X, VEX_OP | AUTO_VEXL | ENC_MR | PREF_66, AVX;
]
"verr" = [
    b"m!"         , [0x0F, 0x00        ], 4;
    b"rw"         , [0x0F, 0x00        ], 4;
]
"verw" = [
    b"m!"         , [0x0F, 0x00        ], 5;
    b"rw"         , [0x0F, 0x00        ], 5;
]
"vextractf128" = [
    b"woyhib"     , [0x03, 0x19        ], X, WITH_VEXL | VEX_OP | ENC_MR | PREF_66, AVX;
]
"vextracti128" = [
    b"woyhib"     , [0x03, 0x39        ], X, WITH_VEXL | VEX_OP | ENC_MR | PREF_66, AVX2;
]
"vextractps" = [
    b"vdyoib"     , [0x03, 0x17        ], X, VEX_OP | ENC_MR | PREF_66, AVX;
]
"vfmadd123pd" = [
    b"y*y*w*"     , [0x02, 0xA8        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmadd123ps" = [
    b"y*y*w*"     , [0x02, 0xA8        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmadd123sd" = [
    b"yoyomq"     , [0x02, 0xA9        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xA9        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
]
"vfmadd123ss" = [
    b"yoyomd"     , [0x02, 0xA9        ], X, VEX_OP | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xA9        ], X, VEX_OP | PREF_66, FMA;
]
"vfmadd132pd" = [
    b"y*y*w*"     , [0x02, 0x98        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmadd132ps" = [
    b"y*y*w*"     , [0x02, 0x98        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmadd132sd" = [
    b"yoyomq"     , [0x02, 0x99        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0x99        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
]
"vfmadd132ss" = [
    b"yoyomd"     , [0x02, 0x99        ], X, VEX_OP | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0x99        ], X, VEX_OP | PREF_66, FMA;
]
"vfmadd213pd" = [
    b"y*y*w*"     , [0x02, 0xA8        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmadd213ps" = [
    b"y*y*w*"     , [0x02, 0xA8        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmadd213sd" = [
    b"yoyomq"     , [0x02, 0xA9        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xA9        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
]
"vfmadd213ss" = [
    b"yoyomd"     , [0x02, 0xA9        ], X, VEX_OP | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xA9        ], X, VEX_OP | PREF_66, FMA;
]
"vfmadd231pd" = [
    b"y*y*w*"     , [0x02, 0xB8        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmadd231ps" = [
    b"y*y*w*"     , [0x02, 0xB8        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmadd231sd" = [
    b"yoyomq"     , [0x02, 0xB9        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xB9        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
]
"vfmadd231ss" = [
    b"yoyomd"     , [0x02, 0xB9        ], X, VEX_OP | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xB9        ], X, VEX_OP | PREF_66, FMA;
]
"vfmadd312pd" = [
    b"y*y*w*"     , [0x02, 0x98        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmadd312ps" = [
    b"y*y*w*"     , [0x02, 0x98        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmadd312sd" = [
    b"yoyomq"     , [0x02, 0x99        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0x99        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
]
"vfmadd312ss" = [
    b"yoyomd"     , [0x02, 0x99        ], X, VEX_OP | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0x99        ], X, VEX_OP | PREF_66, FMA;
]
"vfmadd321pd" = [
    b"y*y*w*"     , [0x02, 0xB8        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmadd321ps" = [
    b"y*y*w*"     , [0x02, 0xB8        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmadd321sd" = [
    b"yoyomq"     , [0x02, 0xB9        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xB9        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
]
"vfmadd321ss" = [
    b"yoyomd"     , [0x02, 0xB9        ], X, VEX_OP | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xB9        ], X, VEX_OP | PREF_66, FMA;
]
"vfmaddpd" = [
    b"y*y*y*w*"   , [0x03, 0x69        ], X, VEX_OP | AUTO_VEXL | PREF_66, AMD | SSE5;
    b"y*y*w*y*"   , [0x03, 0x69        ], X, VEX_OP | AUTO_VEXL | PREF_66, SSE5 | AMD;
]
"vfmaddps" = [
    b"y*y*y*w*"   , [0x03, 0x68        ], X, VEX_OP | AUTO_VEXL | PREF_66, AMD | SSE5;
    b"y*y*w*y*"   , [0x03, 0x68        ], X, VEX_OP | AUTO_VEXL | PREF_66, SSE5 | AMD;
]
"vfmaddsd" = [
    b"yoyomqyo"   , [0x03, 0x6B        ], X, VEX_OP | PREF_66, AMD | SSE5;
    b"yoyoyomq"   , [0x03, 0x6B        ], X, VEX_OP | WITH_REXW | PREF_66, AMD | SSE5;
    b"yoyoyoyo"   , [0x03, 0x6B        ], X, VEX_OP | WITH_REXW | PREF_66, AMD | SSE5;
]
"vfmaddss" = [
    b"yoyomdyo"   , [0x03, 0x6A        ], X, VEX_OP | PREF_66, SSE5 | AMD;
    b"yoyoyomd"   , [0x03, 0x6A        ], X, VEX_OP | WITH_REXW | PREF_66, SSE5 | AMD;
    b"yoyoyoyo"   , [0x03, 0x6A        ], X, VEX_OP | WITH_REXW | PREF_66, SSE5 | AMD;
]
"vfmaddsub123pd" = [
    b"y*y*w*"     , [0x02, 0xA6        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmaddsub123ps" = [
    b"y*y*w*"     , [0x02, 0xA6        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmaddsub132pd" = [
    b"y*y*w*"     , [0x02, 0x96        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmaddsub132ps" = [
    b"y*y*w*"     , [0x02, 0x96        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmaddsub213pd" = [
    b"y*y*w*"     , [0x02, 0xA6        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmaddsub213ps" = [
    b"y*y*w*"     , [0x02, 0xA6        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmaddsub231pd" = [
    b"y*y*w*"     , [0x02, 0xB6        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmaddsub231ps" = [
    b"y*y*w*"     , [0x02, 0xB6        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmaddsub312pd" = [
    b"y*y*w*"     , [0x02, 0x96        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmaddsub312ps" = [
    b"y*y*w*"     , [0x02, 0x96        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmaddsub321pd" = [
    b"y*y*w*"     , [0x02, 0xB6        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmaddsub321ps" = [
    b"y*y*w*"     , [0x02, 0xB6        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmaddsubpd" = [
    b"y*y*y*w*"   , [0x03, 0x5D        ], X, VEX_OP | AUTO_VEXL | PREF_66, SSE5 | AMD;
    b"y*y*w*y*"   , [0x03, 0x5D        ], X, VEX_OP | AUTO_VEXL | PREF_66, AMD | SSE5;
]
"vfmaddsubps" = [
    b"y*y*y*w*"   , [0x03, 0x5C        ], X, VEX_OP | AUTO_VEXL | PREF_66, SSE5 | AMD;
    b"y*y*w*y*"   , [0x03, 0x5C        ], X, VEX_OP | AUTO_VEXL | PREF_66, SSE5 | AMD;
]
"vfmsub123pd" = [
    b"y*y*w*"     , [0x02, 0xAA        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmsub123ps" = [
    b"y*y*w*"     , [0x02, 0xAA        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmsub123sd" = [
    b"yoyomq"     , [0x02, 0xAB        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xAB        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
]
"vfmsub123ss" = [
    b"yoyomd"     , [0x02, 0xAB        ], X, VEX_OP | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xAB        ], X, VEX_OP | PREF_66, FMA;
]
"vfmsub132pd" = [
    b"y*y*w*"     , [0x02, 0x9A        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmsub132ps" = [
    b"y*y*w*"     , [0x02, 0x9A        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmsub132sd" = [
    b"yoyomq"     , [0x02, 0x9B        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0x9B        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
]
"vfmsub132ss" = [
    b"yoyomd"     , [0x02, 0x9B        ], X, VEX_OP | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0x9B        ], X, VEX_OP | PREF_66, FMA;
]
"vfmsub213pd" = [
    b"y*y*w*"     , [0x02, 0xAA        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmsub213ps" = [
    b"y*y*w*"     , [0x02, 0xAA        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmsub213sd" = [
    b"yoyomq"     , [0x02, 0xAB        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xAB        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
]
"vfmsub213ss" = [
    b"yoyomd"     , [0x02, 0xAB        ], X, VEX_OP | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xAB        ], X, VEX_OP | PREF_66, FMA;
]
"vfmsub231pd" = [
    b"y*y*w*"     , [0x02, 0xBA        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmsub231ps" = [
    b"y*y*w*"     , [0x02, 0xBA        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmsub231sd" = [
    b"yoyomq"     , [0x02, 0xBB        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xBB        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
]
"vfmsub231ss" = [
    b"yoyomd"     , [0x02, 0xBB        ], X, VEX_OP | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xBB        ], X, VEX_OP | PREF_66, FMA;
]
"vfmsub312pd" = [
    b"y*y*w*"     , [0x02, 0x9A        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmsub312ps" = [
    b"y*y*w*"     , [0x02, 0x9A        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmsub312sd" = [
    b"yoyomq"     , [0x02, 0x9B        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0x9B        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
]
"vfmsub312ss" = [
    b"yoyomd"     , [0x02, 0x9B        ], X, VEX_OP | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0x9B        ], X, VEX_OP | PREF_66, FMA;
]
"vfmsub321pd" = [
    b"y*y*w*"     , [0x02, 0xBA        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmsub321ps" = [
    b"y*y*w*"     , [0x02, 0xBA        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmsub321sd" = [
    b"yoyomq"     , [0x02, 0xBB        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xBB        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
]
"vfmsub321ss" = [
    b"yoyomd"     , [0x02, 0xBB        ], X, VEX_OP | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xBB        ], X, VEX_OP | PREF_66, FMA;
]
"vfmsubadd123pd" = [
    b"y*y*w*"     , [0x02, 0xA7        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmsubadd123ps" = [
    b"y*y*w*"     , [0x02, 0xA7        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmsubadd132pd" = [
    b"y*y*w*"     , [0x02, 0x97        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmsubadd132ps" = [
    b"y*y*w*"     , [0x02, 0x97        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmsubadd213pd" = [
    b"y*y*w*"     , [0x02, 0xA7        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmsubadd213ps" = [
    b"y*y*w*"     , [0x02, 0xA7        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmsubadd231pd" = [
    b"y*y*w*"     , [0x02, 0xB7        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmsubadd231ps" = [
    b"y*y*w*"     , [0x02, 0xB7        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmsubadd312pd" = [
    b"y*y*w*"     , [0x02, 0x97        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmsubadd312ps" = [
    b"y*y*w*"     , [0x02, 0x97        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmsubadd321pd" = [
    b"y*y*w*"     , [0x02, 0xB7        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmsubadd321ps" = [
    b"y*y*w*"     , [0x02, 0xB7        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfmsubaddpd" = [
    b"y*y*y*w*"   , [0x03, 0x5F        ], X, VEX_OP | AUTO_VEXL | PREF_66, AMD | SSE5;
    b"y*y*w*y*"   , [0x03, 0x5F        ], X, VEX_OP | AUTO_VEXL | PREF_66, AMD | SSE5;
]
"vfmsubaddps" = [
    b"y*y*y*w*"   , [0x03, 0x5E        ], X, VEX_OP | AUTO_VEXL | PREF_66, AMD | SSE5;
    b"y*y*w*y*"   , [0x03, 0x5E        ], X, VEX_OP | AUTO_VEXL | PREF_66, AMD | SSE5;
]
"vfmsubpd" = [
    b"y*y*y*w*"   , [0x03, 0x6D        ], X, VEX_OP | AUTO_VEXL | PREF_66, AMD | SSE5;
    b"y*y*w*y*"   , [0x03, 0x6D        ], X, VEX_OP | AUTO_VEXL | PREF_66, AMD | SSE5;
]
"vfmsubps" = [
    b"y*y*y*w*"   , [0x03, 0x6C        ], X, VEX_OP | AUTO_VEXL | PREF_66, SSE5 | AMD;
    b"y*y*w*y*"   , [0x03, 0x6C        ], X, VEX_OP | AUTO_VEXL | PREF_66, SSE5 | AMD;
]
"vfmsubsd" = [
    b"yoyomqyo"   , [0x03, 0x6F        ], X, VEX_OP | PREF_66, AMD | SSE5;
    b"yoyoyomq"   , [0x03, 0x6F        ], X, VEX_OP | WITH_REXW | PREF_66, SSE5 | AMD;
    b"yoyoyoyo"   , [0x03, 0x6F        ], X, VEX_OP | WITH_REXW | PREF_66, SSE5 | AMD;
]
"vfmsubss" = [
    b"yoyomdyo"   , [0x03, 0x6E        ], X, VEX_OP | PREF_66, AMD | SSE5;
    b"yoyoyomd"   , [0x03, 0x6E        ], X, VEX_OP | WITH_REXW | PREF_66, AMD | SSE5;
    b"yoyoyoyo"   , [0x03, 0x6E        ], X, VEX_OP | WITH_REXW | PREF_66, AMD | SSE5;
]
"vfnmadd123pd" = [
    b"y*y*w*"     , [0x02, 0xAC        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfnmadd123ps" = [
    b"y*y*w*"     , [0x02, 0xAC        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfnmadd123sd" = [
    b"yoyomq"     , [0x02, 0xAD        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xAD        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
]
"vfnmadd123ss" = [
    b"yoyomd"     , [0x02, 0xAD        ], X, VEX_OP | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xAD        ], X, VEX_OP | PREF_66, FMA;
]
"vfnmadd132pd" = [
    b"y*y*w*"     , [0x02, 0x9C        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfnmadd132ps" = [
    b"y*y*w*"     , [0x02, 0x9C        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfnmadd132sd" = [
    b"yoyomq"     , [0x02, 0x9D        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0x9D        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
]
"vfnmadd132ss" = [
    b"yoyomd"     , [0x02, 0x9D        ], X, VEX_OP | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0x9D        ], X, VEX_OP | PREF_66, FMA;
]
"vfnmadd213pd" = [
    b"y*y*w*"     , [0x02, 0xAC        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfnmadd213ps" = [
    b"y*y*w*"     , [0x02, 0xAC        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfnmadd213sd" = [
    b"yoyomq"     , [0x02, 0xAD        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xAD        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
]
"vfnmadd213ss" = [
    b"yoyomd"     , [0x02, 0xAD        ], X, VEX_OP | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xAD        ], X, VEX_OP | PREF_66, FMA;
]
"vfnmadd231pd" = [
    b"y*y*w*"     , [0x02, 0xBC        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfnmadd231ps" = [
    b"y*y*w*"     , [0x02, 0xBC        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfnmadd231sd" = [
    b"yoyomq"     , [0x02, 0xBD        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xBD        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
]
"vfnmadd231ss" = [
    b"yoyomd"     , [0x02, 0xBD        ], X, VEX_OP | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xBD        ], X, VEX_OP | PREF_66, FMA;
]
"vfnmadd312pd" = [
    b"y*y*w*"     , [0x02, 0x9C        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfnmadd312ps" = [
    b"y*y*w*"     , [0x02, 0x9C        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfnmadd312sd" = [
    b"yoyomq"     , [0x02, 0x9D        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0x9D        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
]
"vfnmadd312ss" = [
    b"yoyomd"     , [0x02, 0x9D        ], X, VEX_OP | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0x9D        ], X, VEX_OP | PREF_66, FMA;
]
"vfnmadd321pd" = [
    b"y*y*w*"     , [0x02, 0xBC        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfnmadd321ps" = [
    b"y*y*w*"     , [0x02, 0xBC        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfnmadd321sd" = [
    b"yoyomq"     , [0x02, 0xBD        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xBD        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
]
"vfnmadd321ss" = [
    b"yoyomd"     , [0x02, 0xBD        ], X, VEX_OP | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xBD        ], X, VEX_OP | PREF_66, FMA;
]
"vfnmaddpd" = [
    b"y*y*y*w*"   , [0x03, 0x79        ], X, VEX_OP | AUTO_VEXL | PREF_66, SSE5 | AMD;
    b"y*y*w*y*"   , [0x03, 0x79        ], X, VEX_OP | AUTO_VEXL | PREF_66, AMD | SSE5;
]
"vfnmaddps" = [
    b"y*y*y*w*"   , [0x03, 0x78        ], X, VEX_OP | AUTO_VEXL | PREF_66, AMD | SSE5;
    b"y*y*w*y*"   , [0x03, 0x78        ], X, VEX_OP | AUTO_VEXL | PREF_66, SSE5 | AMD;
]
"vfnmaddsd" = [
    b"yoyomqyo"   , [0x03, 0x7B        ], X, VEX_OP | PREF_66, SSE5 | AMD;
    b"yoyoyomq"   , [0x03, 0x7B        ], X, VEX_OP | WITH_REXW | PREF_66, AMD | SSE5;
    b"yoyoyoyo"   , [0x03, 0x7B        ], X, VEX_OP | WITH_REXW | PREF_66, AMD | SSE5;
]
"vfnmaddss" = [
    b"yoyomdyo"   , [0x03, 0x7A        ], X, VEX_OP | PREF_66, SSE5 | AMD;
    b"yoyoyomd"   , [0x03, 0x7A        ], X, VEX_OP | WITH_REXW | PREF_66, AMD | SSE5;
    b"yoyoyoyo"   , [0x03, 0x7A        ], X, VEX_OP | WITH_REXW | PREF_66, AMD | SSE5;
]
"vfnmsub123pd" = [
    b"y*y*w*"     , [0x02, 0xAE        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfnmsub123ps" = [
    b"y*y*w*"     , [0x02, 0xAE        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfnmsub123sd" = [
    b"yoyomq"     , [0x02, 0xAF        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xAF        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
]
"vfnmsub123ss" = [
    b"yoyomd"     , [0x02, 0xAF        ], X, VEX_OP | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xAF        ], X, VEX_OP | PREF_66, FMA;
]
"vfnmsub132pd" = [
    b"y*y*w*"     , [0x02, 0x9E        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfnmsub132ps" = [
    b"y*y*w*"     , [0x02, 0x9E        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfnmsub132sd" = [
    b"yoyomq"     , [0x02, 0x9F        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0x9F        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
]
"vfnmsub132ss" = [
    b"yoyomd"     , [0x02, 0x9F        ], X, VEX_OP | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0x9F        ], X, VEX_OP | PREF_66, FMA;
]
"vfnmsub213pd" = [
    b"y*y*w*"     , [0x02, 0xAE        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfnmsub213ps" = [
    b"y*y*w*"     , [0x02, 0xAE        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfnmsub213sd" = [
    b"yoyomq"     , [0x02, 0xAF        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xAF        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
]
"vfnmsub213ss" = [
    b"yoyomd"     , [0x02, 0xAF        ], X, VEX_OP | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xAF        ], X, VEX_OP | PREF_66, FMA;
]
"vfnmsub231pd" = [
    b"y*y*w*"     , [0x02, 0xBE        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfnmsub231ps" = [
    b"y*y*w*"     , [0x02, 0xBE        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfnmsub231sd" = [
    b"yoyomq"     , [0x02, 0xBF        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xBF        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
]
"vfnmsub231ss" = [
    b"yoyomd"     , [0x02, 0xBF        ], X, VEX_OP | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xBF        ], X, VEX_OP | PREF_66, FMA;
]
"vfnmsub312pd" = [
    b"y*y*w*"     , [0x02, 0x9E        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfnmsub312ps" = [
    b"y*y*w*"     , [0x02, 0x9E        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfnmsub312sd" = [
    b"yoyomq"     , [0x02, 0x9F        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0x9F        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
]
"vfnmsub312ss" = [
    b"yoyomd"     , [0x02, 0x9F        ], X, VEX_OP | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0x9F        ], X, VEX_OP | PREF_66, FMA;
]
"vfnmsub321pd" = [
    b"y*y*w*"     , [0x02, 0xBE        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfnmsub321ps" = [
    b"y*y*w*"     , [0x02, 0xBE        ], X, VEX_OP | AUTO_VEXL | PREF_66, FMA;
]
"vfnmsub321sd" = [
    b"yoyomq"     , [0x02, 0xBF        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xBF        ], X, VEX_OP | WITH_REXW | PREF_66, FMA;
]
"vfnmsub321ss" = [
    b"yoyomd"     , [0x02, 0xBF        ], X, VEX_OP | PREF_66, FMA;
    b"yoyoyo"     , [0x02, 0xBF        ], X, VEX_OP | PREF_66, FMA;
]
"vfnmsubpd" = [
    b"y*y*y*w*"   , [0x03, 0x7D        ], X, VEX_OP | AUTO_VEXL | PREF_66, AMD | SSE5;
    b"y*y*w*y*"   , [0x03, 0x7D        ], X, VEX_OP | AUTO_VEXL | PREF_66, AMD | SSE5;
]
"vfnmsubps" = [
    b"y*y*y*w*"   , [0x03, 0x7C        ], X, VEX_OP | AUTO_VEXL | PREF_66, SSE5 | AMD;
    b"y*y*w*y*"   , [0x03, 0x7C        ], X, VEX_OP | AUTO_VEXL | PREF_66, AMD | SSE5;
]
"vfnmsubsd" = [
    b"yoyomqyo"   , [0x03, 0x7F        ], X, VEX_OP | PREF_66, SSE5 | AMD;
    b"yoyoyomq"   , [0x03, 0x7F        ], X, VEX_OP | WITH_REXW | PREF_66, AMD | SSE5;
    b"yoyoyoyo"   , [0x03, 0x7F        ], X, VEX_OP | WITH_REXW | PREF_66, AMD | SSE5;
]
"vfnmsubss" = [
    b"yoyomdyo"   , [0x03, 0x7E        ], X, VEX_OP | PREF_66, SSE5 | AMD;
    b"yoyoyomd"   , [0x03, 0x7E        ], X, VEX_OP | WITH_REXW | PREF_66, SSE5 | AMD;
    b"yoyoyoyo"   , [0x03, 0x7E        ], X, VEX_OP | WITH_REXW | PREF_66, SSE5 | AMD;
]
"vfrczpd" = [
    b"y*w*"       , [0x09, 0x81        ], X, XOP_OP | AUTO_VEXL, SSE5 | AMD;
]
"vfrczps" = [
    b"y*w*"       , [0x09, 0x80        ], X, XOP_OP | AUTO_VEXL, AMD | SSE5;
]
"vfrczsd" = [
    b"yomq"       , [0x09, 0x83        ], X, XOP_OP, SSE5 | AMD;
    b"yoyo"       , [0x09, 0x83        ], X, XOP_OP, SSE5 | AMD;
]
"vfrczss" = [
    b"yomd"       , [0x09, 0x82        ], X, XOP_OP, AMD | SSE5;
    b"yoyo"       , [0x09, 0x82        ], X, XOP_OP, AMD | SSE5;
]
"vgatherdpd" = [
    b"y*loy*"     , [0x02, 0x92        ], X, VEX_OP | AUTO_VEXL | ENC_MR | PREF_66, AVX2;
]
"vgatherdps" = [
    b"y*k*y*"     , [0x02, 0x92        ], X, VEX_OP | AUTO_VEXL | ENC_MR | PREF_66, AVX2;
]
"vgatherqpd" = [
    b"y*l*y*"     , [0x02, 0x93        ], X, VEX_OP | AUTO_VEXL | ENC_MR | PREF_66, AVX2;
]
"vgatherqps" = [
    b"yok*yo"     , [0x02, 0x93        ], X, VEX_OP | AUTO_VEXL | ENC_MR | PREF_66, AVX2;
]
"vhaddpd" = [
    b"y*y*w*"     , [0x01, 0x7C        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vhaddps" = [
    b"y*y*w*"     , [0x01, 0x7C        ], X, VEX_OP | AUTO_VEXL | PREF_F2, AVX;
]
"vhsubpd" = [
    b"y*y*w*"     , [0x01, 0x7D        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vhsubps" = [
    b"y*y*w*"     , [0x01, 0x7D        ], X, VEX_OP | AUTO_VEXL | PREF_F2, AVX;
]
"vinsertf128" = [
    b"yhyhwoib"   , [0x03, 0x18        ], X, WITH_VEXL | VEX_OP | PREF_66, AVX;
]
"vinserti128" = [
    b"yhyhwoib"   , [0x03, 0x38        ], X, WITH_VEXL | VEX_OP | PREF_66, AVX2;
]
"vinsertps" = [
    b"yoyomdib"   , [0x03, 0x21        ], X, VEX_OP | PREF_66, AVX;
    b"yoyoyoib"   , [0x03, 0x21        ], X, VEX_OP | PREF_66, AVX;
]
"vlddqu" = [
    b"y*m*"       , [0x01, 0xF0        ], X, VEX_OP | AUTO_VEXL | PREF_F2, AVX;
]
"vldmxcsr" = [
    b"md"         , [0x01, 0xAE        ], 2, VEX_OP, AVX;
]
"vldqqu" = [
    b"yhmh"       , [0x01, 0xF0        ], X, WITH_VEXL | VEX_OP | PREF_F2, AVX;
]
"vmaskmovdqu" = [
    b"yoyo"       , [0x01, 0xF7        ], X, VEX_OP | PREF_66, AVX;
]
"vmaskmovpd" = [
    b"m*y*y*"     , [0x02, 0x2F        ], X, VEX_OP | AUTO_VEXL | ENC_VM | PREF_66, AVX;
    b"y*y*m*"     , [0x02, 0x2D        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vmaskmovps" = [
    b"m*y*y*"     , [0x02, 0x2E        ], X, VEX_OP | AUTO_VEXL | ENC_VM | PREF_66, AVX;
    b"y*y*m*"     , [0x02, 0x2C        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vmaxpd" = [
    b"y*y*w*"     , [0x01, 0x5F        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vmaxps" = [
    b"y*y*w*"     , [0x01, 0x5F        ], X, VEX_OP | AUTO_VEXL, AVX;
]
"vmaxsd" = [
    b"yoyomq"     , [0x01, 0x5F        ], X, VEX_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0x5F        ], X, VEX_OP | PREF_F2, AVX;
]
"vmaxss" = [
    b"yoyomd"     , [0x01, 0x5F        ], X, VEX_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0x5F        ], X, VEX_OP | PREF_F3, AVX;
]
"vmcall" = [
    b""           , [0x0F, 0x01, 0xC1  ], X, DEFAULT, VMX;
]
"vmclear" = [
    b"m!"         , [0x0F, 0xC7        ], 6, PREF_66, VMX;
]
"vmfunc" = [
    b""           , [0x0F, 0x01, 0xD4  ], X, DEFAULT, VMX;
]
"vminpd" = [
    b"y*y*w*"     , [0x01, 0x5D        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vminps" = [
    b"y*y*w*"     , [0x01, 0x5D        ], X, VEX_OP | AUTO_VEXL, AVX;
]
"vminsd" = [
    b"yoyomq"     , [0x01, 0x5D        ], X, VEX_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0x5D        ], X, VEX_OP | PREF_F2, AVX;
]
"vminss" = [
    b"yoyomd"     , [0x01, 0x5D        ], X, VEX_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0x5D        ], X, VEX_OP | PREF_F3, AVX;
]
"vmlaunch" = [
    b""           , [0x0F, 0x01, 0xC2  ], X, DEFAULT, VMX;
]
"vmload" = [
    b""           , [0x0F, 0x01, 0xDA  ], X, DEFAULT, AMD | VMX;
]
"vmmcall" = [
    b""           , [0x0F, 0x01, 0xD9  ], X, DEFAULT, AMD | VMX;
]
"vmovapd" = [
    b"y*w*"       , [0x01, 0x28        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
    b"whyh"       , [0x01, 0x29        ], X, VEX_OP | WITH_VEXL | ENC_MR | PREF_66, AVX;
    b"woyo"       , [0x01, 0x29        ], X, VEX_OP | ENC_MR | PREF_66, AVX;
]
"vmovaps" = [
    b"y*w*"       , [0x01, 0x28        ], X, VEX_OP | AUTO_VEXL, AVX;
    b"whyh"       , [0x01, 0x29        ], X, VEX_OP | WITH_VEXL | ENC_MR, AVX;
    b"woyo"       , [0x01, 0x29        ], X, VEX_OP | ENC_MR, AVX;
]
"vmovd" = [
    b"yovd"       , [0x01, 0x6E        ], X, VEX_OP | PREF_66, AVX;
    b"vdyo"       , [0x01, 0x7E        ], X, VEX_OP | ENC_MR | PREF_66, AVX;
]
"vmovddup" = [
    b"y*w*"       , [0x01, 0x12        ], X, VEX_OP | AUTO_VEXL | PREF_F2, AVX;
    b"yomq"       , [0x01, 0x12        ], X, VEX_OP | PREF_F2, AVX;
]
"vmovdqa" = [
    b"y*w*"       , [0x01, 0x6F        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
    b"whyh"       , [0x01, 0x7F        ], X, VEX_OP | WITH_VEXL | ENC_MR | PREF_66, AVX;
    b"woyo"       , [0x01, 0x7F        ], X, VEX_OP | ENC_MR | PREF_66, AVX;
]
"vmovdqu" = [
    b"y*w*"       , [0x01, 0x6F        ], X, VEX_OP | AUTO_VEXL | PREF_F3, AVX;
    b"whyh"       , [0x01, 0x7F        ], X, VEX_OP | WITH_VEXL | ENC_MR | PREF_F3, AVX;
    b"woyo"       , [0x01, 0x7F        ], X, VEX_OP | ENC_MR | PREF_F3, AVX;
]
"vmovhlps" = [
    b"yoyoyo"     , [0x01, 0x12        ], X, VEX_OP, AVX;
]
"vmovhpd" = [
    b"mqyo"       , [0x01, 0x17        ], X, VEX_OP | ENC_MR | PREF_66, AVX;
    b"yoyomq"     , [0x01, 0x16        ], X, VEX_OP | PREF_66, AVX;
]
"vmovhps" = [
    b"mqyo"       , [0x01, 0x17        ], X, VEX_OP | ENC_MR, AVX;
    b"yoyomq"     , [0x01, 0x16        ], X, VEX_OP, AVX;
]
"vmovlhps" = [
    b"yoyoyo"     , [0x01, 0x16        ], X, VEX_OP, AVX;
]
"vmovlpd" = [
    b"mqyo"       , [0x01, 0x13        ], X, VEX_OP | ENC_MR | PREF_66, AVX;
    b"yoyomq"     , [0x01, 0x12        ], X, VEX_OP | PREF_66, AVX;
]
"vmovlps" = [
    b"mqyo"       , [0x01, 0x13        ], X, VEX_OP | ENC_MR, AVX;
    b"yoyomq"     , [0x01, 0x12        ], X, VEX_OP, AVX;
]
"vmovmskpd" = [
    b"r*y*"       , [0x01, 0x50        ], X, VEX_OP | PREF_66, AVX;
]
"vmovmskps" = [
    b"r*y*"       , [0x01, 0x50        ], X, VEX_OP, AVX;
]
"vmovntdq" = [
    b"m*y*"       , [0x01, 0xE7        ], X, VEX_OP | AUTO_VEXL | ENC_MR | PREF_66, AVX;
]
"vmovntdqa" = [
    b"y*m*"       , [0x02, 0x2A        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vmovntpd" = [
    b"m*y*"       , [0x01, 0x2B        ], X, VEX_OP | AUTO_VEXL | ENC_MR | PREF_66, AVX;
]
"vmovntps" = [
    b"m*y*"       , [0x01, 0x2B        ], X, VEX_OP | AUTO_VEXL | ENC_MR, AVX;
]
"vmovntqq" = [
    b"mhyh"       , [0x01, 0xE7        ], X, WITH_VEXL | VEX_OP | ENC_MR | PREF_66, AVX;
]
"vmovq" = [
    b"mqyo"       , [0x01, 0xD6        ], X, VEX_OP | ENC_MR | PREF_66, AVX;
    b"yomq"       , [0x01, 0x7E        ], X, VEX_OP | PREF_F3, AVX;
    b"yovq"       , [0x01, 0x6E        ], X, WITH_REXW | VEX_OP | PREF_66, AVX;
    b"yoyo"       , [0x01, 0x7E        ], X, VEX_OP | PREF_F3, AVX;
    b"yoyo"       , [0x01, 0xD6        ], X, VEX_OP | ENC_MR | PREF_66, AVX;
    b"vqyo"       , [0x01, 0x7E        ], X, WITH_REXW | VEX_OP | ENC_MR | PREF_66, AVX;
]
"vmovqqa" = [
    b"yhwh"       , [0x01, 0x6F        ], X, WITH_VEXL | VEX_OP | PREF_66, AVX;
    b"whyh"       , [0x01, 0x7F        ], X, WITH_VEXL | VEX_OP | ENC_MR | PREF_66, AVX;
]
"vmovqqu" = [
    b"yhwh"       , [0x01, 0x6F        ], X, WITH_VEXL | VEX_OP | PREF_F3, AVX;
    b"whyh"       , [0x01, 0x7F        ], X, WITH_VEXL | VEX_OP | ENC_MR | PREF_F3, AVX;
]
"vmovsd" = [
    b"mqyo"       , [0x01, 0x11        ], X, VEX_OP | ENC_MR | PREF_F2, AVX;
    b"yomq"       , [0x01, 0x10        ], X, VEX_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0x10        ], X, VEX_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0x11        ], X, VEX_OP | ENC_VM | PREF_F2, AVX;
]
"vmovshdup" = [
    b"y*w*"       , [0x01, 0x16        ], X, VEX_OP | AUTO_VEXL | PREF_F3, AVX;
]
"vmovsldup" = [
    b"y*w*"       , [0x01, 0x12        ], X, VEX_OP | AUTO_VEXL | PREF_F3, AVX;
]
"vmovss" = [
    b"mdyo"       , [0x01, 0x11        ], X, VEX_OP | ENC_MR | PREF_F3, AVX;
    b"yomd"       , [0x01, 0x10        ], X, VEX_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0x10        ], X, VEX_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0x11        ], X, VEX_OP | ENC_VM | PREF_F3, AVX;
]
"vmovupd" = [
    b"y*w*"       , [0x01, 0x10        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
    b"whyh"       , [0x01, 0x11        ], X, VEX_OP | WITH_VEXL | ENC_MR | PREF_66, AVX;
    b"woyo"       , [0x01, 0x11        ], X, VEX_OP | ENC_MR | PREF_66, AVX;
]
"vmovups" = [
    b"y*w*"       , [0x01, 0x10        ], X, VEX_OP | AUTO_VEXL, AVX;
    b"whyh"       , [0x01, 0x11        ], X, VEX_OP | WITH_VEXL | ENC_MR, AVX;
    b"woyo"       , [0x01, 0x11        ], X, VEX_OP | ENC_MR, AVX;
]
"vmpsadbw" = [
    b"y*y*w*ib"   , [0x03, 0x42        ], X, VEX_OP | AUTO_VEXL | ENC_MR | PREF_66, AVX;
]
"vmptrld" = [
    b"m!"         , [0x0F, 0xC7        ], 6, DEFAULT, VMX;
]
"vmptrst" = [
    b"m!"         , [0x0F, 0xC7        ], 7, DEFAULT, VMX;
]
"vmread" = [
    b"vqrq"       , [0x0F, 0x78        ], X, ENC_MR, VMX;
]
"vmresume" = [
    b""           , [0x0F, 0x01, 0xC3  ], X, DEFAULT, VMX;
]
"vmrun" = [
    b""           , [0x0F, 0x01, 0xD8  ], X, DEFAULT, AMD | VMX;
]
"vmsave" = [
    b""           , [0x0F, 0x01, 0xDB  ], X, DEFAULT, VMX | AMD;
]
"vmulpd" = [
    b"y*y*w*"     , [0x01, 0x59        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vmulps" = [
    b"y*y*w*"     , [0x01, 0x59        ], X, VEX_OP | AUTO_VEXL, AVX;
]
"vmulsd" = [
    b"yoyomq"     , [0x01, 0x59        ], X, VEX_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0x59        ], X, VEX_OP | PREF_F2, AVX;
]
"vmulss" = [
    b"yoyomd"     , [0x01, 0x59        ], X, VEX_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0x59        ], X, VEX_OP | PREF_F3, AVX;
]
"vmwrite" = [
    b"rqvq"       , [0x0F, 0x79        ], X, DEFAULT, VMX;
]
"vmxoff" = [
    b""           , [0x0F, 0x01, 0xC4  ], X, DEFAULT, VMX;
]
"vmxon" = [
    b"m!"         , [0x0F, 0xC7        ], 6, PREF_F3, VMX;
]
"vorpd" = [
    b"y*y*w*"     , [0x01, 0x56        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vorps" = [
    b"y*y*w*"     , [0x01, 0x56        ], X, VEX_OP | AUTO_VEXL, AVX;
]
"vpabsb" = [
    b"y*w*"       , [0x02, 0x1C        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpabsd" = [
    b"y*w*"       , [0x02, 0x1E        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpabsw" = [
    b"y*w*"       , [0x02, 0x1D        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpackssdw" = [
    b"y*y*w*"     , [0x01, 0x6B        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpacksswb" = [
    b"y*y*w*"     , [0x01, 0x63        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpackusdw" = [
    b"y*y*w*"     , [0x02, 0x2B        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpackuswb" = [
    b"y*y*w*"     , [0x01, 0x67        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpaddb" = [
    b"y*y*w*"     , [0x01, 0xFC        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpaddd" = [
    b"y*y*w*"     , [0x01, 0xFE        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpaddq" = [
    b"y*y*w*"     , [0x01, 0xD4        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpaddsb" = [
    b"y*y*w*"     , [0x01, 0xEC        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpaddsw" = [
    b"y*y*w*"     , [0x01, 0xED        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpaddusb" = [
    b"y*y*w*"     , [0x01, 0xDC        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpaddusw" = [
    b"y*y*w*"     , [0x01, 0xDD        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpaddw" = [
    b"y*y*w*"     , [0x01, 0xFD        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpalignr" = [
    b"y*y*w*ib"   , [0x03, 0x0F        ], X, VEX_OP | AUTO_VEXL | ENC_MR | PREF_66, AVX;
]
"vpand" = [
    b"y*y*w*"     , [0x01, 0xDB        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpandn" = [
    b"y*y*w*"     , [0x01, 0xDF        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpavgb" = [
    b"y*y*w*"     , [0x01, 0xE0        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpavgw" = [
    b"y*y*w*"     , [0x01, 0xE3        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpblendd" = [
    b"y*y*w*ib"   , [0x03, 0x02        ], X, VEX_OP | AUTO_VEXL | ENC_MR | PREF_66, AVX2;
]
"vpblendvb" = [
    b"y*y*w*y*"   , [0x03, 0x4C        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpblendw" = [
    b"y*y*w*ib"   , [0x03, 0x0E        ], X, VEX_OP | AUTO_VEXL | ENC_MR | PREF_66, AVX;
]
"vpbroadcastb" = [
    b"y*mb"       , [0x02, 0x78        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX2;
    b"y*yo"       , [0x02, 0x78        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX2;
]
"vpbroadcastd" = [
    b"y*md"       , [0x02, 0x58        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX2;
    b"y*yo"       , [0x02, 0x58        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX2;
]
"vpbroadcastq" = [
    b"yhmq"       , [0x02, 0x59        ], X, VEX_OP | WITH_VEXL | PREF_66, AVX2;
    b"yomq"       , [0x02, 0x59        ], X, VEX_OP | PREF_66, AVX2;
    b"y*yo"       , [0x02, 0x59        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX2;
]
"vpbroadcastw" = [
    b"y*mw"       , [0x02, 0x79        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX2;
    b"y*yo"       , [0x02, 0x79        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX2;
]
"vpclmulhqhqdq" = [
    b"yoyowo"     , [0x03, 0x44, 0x11  ], X, VEX_OP | PREF_66 | IMM_OP, AVX;
]
"vpclmulhqlqdq" = [
    b"yoyowo"     , [0x03, 0x44, 0x01  ], X, VEX_OP | IMM_OP | PREF_66, AVX;
]
"vpclmullqhqdq" = [
    b"yoyowo"     , [0x03, 0x44, 0x10  ], X, VEX_OP | IMM_OP | PREF_66, AVX;
]
"vpclmullqlqdq" = [
    b"yoyowo"     , [0x03, 0x44, 0x00  ], X, VEX_OP | IMM_OP | PREF_66, AVX;
]
"vpclmulqdq" = [
    b"yoyowoib"   , [0x03, 0x44        ], X, VEX_OP | PREF_66, AVX;
]
"vpcmov" = [
    b"y*y*w*y*"   , [0x08, 0xA2        ], X, XOP_OP | AUTO_VEXL, SSE5 | AMD;
    b"y*y*y*w*"   , [0x08, 0xA2        ], X, XOP_OP | AUTO_VEXL, AMD | SSE5;
]
"vpcmpeqb" = [
    b"y*y*w*"     , [0x01, 0x74        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpcmpeqd" = [
    b"y*y*w*"     , [0x01, 0x76        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpcmpeqq" = [
    b"y*y*w*"     , [0x02, 0x29        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpcmpeqw" = [
    b"y*y*w*"     , [0x01, 0x75        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpcmpestri" = [
    b"yowoib"     , [0x03, 0x61        ], X, VEX_OP | PREF_66, AVX;
]
"vpcmpestrm" = [
    b"yowoib"     , [0x03, 0x60        ], X, VEX_OP | PREF_66, AVX;
]
"vpcmpgtb" = [
    b"y*y*w*"     , [0x01, 0x64        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpcmpgtd" = [
    b"y*y*w*"     , [0x01, 0x66        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpcmpgtq" = [
    b"y*y*w*"     , [0x02, 0x37        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpcmpgtw" = [
    b"y*y*w*"     , [0x01, 0x65        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpcmpistri" = [
    b"yowoib"     , [0x03, 0x63        ], X, VEX_OP | PREF_66, AVX;
]
"vpcmpistrm" = [
    b"yowoib"     , [0x03, 0x62        ], X, VEX_OP | PREF_66, AVX;
]
"vpcomb" = [
    b"yoyowoib"   , [0x08, 0xCC        ], X, XOP_OP, AMD | SSE5;
]
"vpcomd" = [
    b"yoyowoib"   , [0x08, 0xCE        ], X, XOP_OP, AMD | SSE5;
]
"vpcomq" = [
    b"yoyowoib"   , [0x08, 0xCF        ], X, XOP_OP, SSE5 | AMD;
]
"vpcomub" = [
    b"yoyowoib"   , [0x08, 0xEC        ], X, XOP_OP, AMD | SSE5;
]
"vpcomud" = [
    b"yoyowoib"   , [0x08, 0xEE        ], X, XOP_OP, SSE5 | AMD;
]
"vpcomuq" = [
    b"yoyowoib"   , [0x08, 0xEF        ], X, XOP_OP, AMD | SSE5;
]
"vpcomuw" = [
    b"yoyowoib"   , [0x08, 0xED        ], X, XOP_OP, SSE5 | AMD;
]
"vpcomw" = [
    b"yoyowoib"   , [0x08, 0xCD        ], X, XOP_OP, AMD | SSE5;
]
"vperm2f128" = [
    b"yhyhwhib"   , [0x03, 0x06        ], X, WITH_VEXL | VEX_OP | PREF_66, AVX;
]
"vperm2i128" = [
    b"yhyhwhib"   , [0x03, 0x46        ], X, WITH_VEXL | VEX_OP | PREF_66, AVX2;
]
"vpermd" = [
    b"yhyhwh"     , [0x02, 0x36        ], X, WITH_VEXL | VEX_OP | PREF_66, AVX2;
]
"vpermilpd" = [
    b"y*y*w*"     , [0x02, 0x0D        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
    b"y*w*ib"     , [0x03, 0x05        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpermilps" = [
    b"y*y*w*"     , [0x02, 0x0C        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
    b"y*w*ib"     , [0x03, 0x04        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpermpd" = [
    b"yhwhib"     , [0x03, 0x01        ], X, WITH_VEXL | WITH_REXW | VEX_OP | PREF_66, AVX2;
]
"vpermps" = [
    b"yhyhwh"     , [0x02, 0x16        ], X, WITH_VEXL | VEX_OP | PREF_66, AVX2;
]
"vpermq" = [
    b"yhwhib"     , [0x03, 0x00        ], X, WITH_VEXL | WITH_REXW | VEX_OP | PREF_66, AVX2;
]
"vpextrb" = [
    b"mbyoib"     , [0x03, 0x14        ], X, VEX_OP | ENC_MR | PREF_66, AVX;
    b"rdyoib"     , [0x03, 0x14        ], X, VEX_OP | ENC_MR | PREF_66, AVX;
    b"rqyoib"     , [0x03, 0x14        ], X, VEX_OP | ENC_MR | PREF_66, AVX;
]
"vpextrd" = [
    b"rqyoib"     , [0x03, 0x16        ], X, VEX_OP | ENC_MR | PREF_66, AVX;
    b"vdyoib"     , [0x03, 0x16        ], X, VEX_OP | ENC_MR | PREF_66, AVX;
]
"vpextrq" = [
    b"vqyoib"     , [0x03, 0x16        ], X, WITH_REXW | VEX_OP | ENC_MR | PREF_66, AVX;
]
"vpextrw" = [
    b"mwyoib"     , [0x03, 0x15        ], X, VEX_OP | ENC_MR | PREF_66, AVX;
    b"rdyoib"     , [0x01, 0xC5        ], X, VEX_OP | PREF_66, AVX;
    b"rdyoib"     , [0x03, 0x15        ], X, VEX_OP | ENC_MR | PREF_66, AVX;
    b"rqyoib"     , [0x01, 0xC5        ], X, VEX_OP | PREF_66, AVX;
    b"rqyoib"     , [0x03, 0x15        ], X, VEX_OP | ENC_MR | PREF_66, AVX;
]
"vpgatherdd" = [
    b"y*k*y*"     , [0x02, 0x90        ], X, VEX_OP | AUTO_VEXL | ENC_MR | PREF_66, AVX2;
]
"vpgatherdq" = [
    b"y*loy*"     , [0x02, 0x90        ], X, VEX_OP | AUTO_VEXL | ENC_MR | PREF_66, AVX2;
]
"vpgatherqd" = [
    b"yok*yo"     , [0x02, 0x91        ], X, VEX_OP | AUTO_VEXL | ENC_MR | PREF_66, AVX2;
]
"vpgatherqq" = [
    b"y*l*y*"     , [0x02, 0x91        ], X, VEX_OP | AUTO_VEXL | ENC_MR | PREF_66, AVX2;
]
"vphaddbd" = [
    b"yowo"       , [0x09, 0xC2        ], X, XOP_OP, SSE5 | AMD;
]
"vphaddbq" = [
    b"yowo"       , [0x09, 0xC3        ], X, XOP_OP, SSE5 | AMD;
]
"vphaddbw" = [
    b"yowo"       , [0x09, 0xC1        ], X, XOP_OP, SSE5 | AMD;
]
"vphaddd" = [
    b"y*y*w*"     , [0x02, 0x02        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vphadddq" = [
    b"yowo"       , [0x09, 0xCB        ], X, XOP_OP, SSE5 | AMD;
]
"vphaddsw" = [
    b"y*y*w*"     , [0x02, 0x03        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vphaddubd" = [
    b"yowo"       , [0x09, 0xD2        ], X, XOP_OP, SSE5 | AMD;
]
"vphaddubq" = [
    b"yowo"       , [0x09, 0xD3        ], X, XOP_OP, AMD | SSE5;
]
"vphaddubw" = [
    b"yowo"       , [0x09, 0xD1        ], X, XOP_OP, SSE5 | AMD;
]
"vphaddudq" = [
    b"yowo"       , [0x09, 0xDB        ], X, XOP_OP, AMD | SSE5;
]
"vphadduwd" = [
    b"yowo"       , [0x09, 0xD6        ], X, XOP_OP, AMD | SSE5;
]
"vphadduwq" = [
    b"yowo"       , [0x09, 0xD7        ], X, XOP_OP, AMD | SSE5;
]
"vphaddw" = [
    b"y*y*w*"     , [0x02, 0x01        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vphaddwd" = [
    b"yowo"       , [0x09, 0xC6        ], X, XOP_OP, AMD | SSE5;
]
"vphaddwq" = [
    b"yowo"       , [0x09, 0xC7        ], X, XOP_OP, AMD | SSE5;
]
"vphminposuw" = [
    b"yowo"       , [0x02, 0x41        ], X, VEX_OP | PREF_66, AVX;
]
"vphsubbw" = [
    b"yowo"       , [0x09, 0xE1        ], X, XOP_OP, SSE5 | AMD;
]
"vphsubd" = [
    b"y*y*w*"     , [0x02, 0x06        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vphsubdq" = [
    b"yowo"       , [0x09, 0xE3        ], X, XOP_OP, SSE5 | AMD;
]
"vphsubsw" = [
    b"y*y*w*"     , [0x02, 0x07        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vphsubw" = [
    b"y*y*w*"     , [0x02, 0x05        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vphsubwd" = [
    b"yowo"       , [0x09, 0xE2        ], X, XOP_OP, SSE5 | AMD;
]
"vpinsrb" = [
    b"yoyordib"   , [0x03, 0x20        ], X, VEX_OP | PREF_66, AVX;
    b"yoyovbib"   , [0x03, 0x20        ], X, VEX_OP | PREF_66, AVX;
]
"vpinsrd" = [
    b"yoyovdib"   , [0x03, 0x22        ], X, VEX_OP | PREF_66, AVX;
]
"vpinsrq" = [
    b"yoyovqib"   , [0x03, 0x22        ], X, VEX_OP | WITH_REXW | PREF_66, AVX;
]
"vpinsrw" = [
    b"yoyordib"   , [0x01, 0xC4        ], X, VEX_OP | PREF_66, AVX;
    b"yoyovwib"   , [0x01, 0xC4        ], X, VEX_OP | PREF_66, AVX;
]
"vpmacsdd" = [
    b"yoyowoyo"   , [0x08, 0x9E        ], X, XOP_OP, AMD | SSE5;
]
"vpmacsdqh" = [
    b"yoyowoyo"   , [0x08, 0x9F        ], X, XOP_OP, SSE5 | AMD;
]
"vpmacsdql" = [
    b"yoyowoyo"   , [0x08, 0x97        ], X, XOP_OP, AMD | SSE5;
]
"vpmacssdd" = [
    b"yoyowoyo"   , [0x08, 0x8E        ], X, XOP_OP, SSE5 | AMD;
]
"vpmacssdqh" = [
    b"yoyowoyo"   , [0x08, 0x8F        ], X, XOP_OP, AMD | SSE5;
]
"vpmacssdql" = [
    b"yoyowoyo"   , [0x08, 0x87        ], X, XOP_OP, SSE5 | AMD;
]
"vpmacsswd" = [
    b"yoyowoyo"   , [0x08, 0x86        ], X, XOP_OP, AMD | SSE5;
]
"vpmacssww" = [
    b"yoyowoyo"   , [0x08, 0x85        ], X, XOP_OP, AMD | SSE5;
]
"vpmacswd" = [
    b"yoyowoyo"   , [0x08, 0x96        ], X, XOP_OP, SSE5 | AMD;
]
"vpmacsww" = [
    b"yoyowoyo"   , [0x08, 0x95        ], X, XOP_OP, AMD | SSE5;
]
"vpmadcsswd" = [
    b"yoyowoyo"   , [0x08, 0xA6        ], X, XOP_OP, SSE5 | AMD;
]
"vpmadcswd" = [
    b"yoyowoyo"   , [0x08, 0xB6        ], X, XOP_OP, AMD | SSE5;
]
"vpmaddubsw" = [
    b"y*y*w*"     , [0x02, 0x04        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpmaddwd" = [
    b"y*y*w*"     , [0x01, 0xF5        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpmaskmovd" = [
    b"m*y*y*"     , [0x02, 0x8E        ], X, VEX_OP | AUTO_VEXL | ENC_VM | PREF_66, AVX2;
    b"y*y*m*"     , [0x02, 0x8C        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX2;
]
"vpmaskmovq" = [
    b"m*y*y*"     , [0x02, 0x8E        ], X, VEX_OP | AUTO_VEXL | ENC_VM | PREF_66, AVX2;
    b"y*y*m*"     , [0x02, 0x8C        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX2;
]
"vpmaxsb" = [
    b"y*y*w*"     , [0x02, 0x3C        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpmaxsd" = [
    b"y*y*w*"     , [0x02, 0x3D        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpmaxsw" = [
    b"y*y*w*"     , [0x01, 0xEE        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpmaxub" = [
    b"y*y*w*"     , [0x01, 0xDE        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpmaxud" = [
    b"y*y*w*"     , [0x02, 0x3F        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpmaxuw" = [
    b"y*y*w*"     , [0x02, 0x3E        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpminsb" = [
    b"y*y*w*"     , [0x02, 0x38        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpminsd" = [
    b"y*y*w*"     , [0x02, 0x39        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpminsw" = [
    b"y*y*w*"     , [0x01, 0xEA        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpminub" = [
    b"y*y*w*"     , [0x01, 0xDA        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpminud" = [
    b"y*y*w*"     , [0x02, 0x3B        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpminuw" = [
    b"y*y*w*"     , [0x02, 0x3A        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpmovmskb" = [
    b"r*y*"       , [0x01, 0xD7        ], X, VEX_OP | PREF_66, AVX;
]
"vpmovsxbd" = [
    b"yomd"       , [0x02, 0x21        ], X, VEX_OP | PREF_66, AVX;
    b"y*yo"       , [0x02, 0x21        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpmovsxbq" = [
    b"y*mw"       , [0x02, 0x22        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
    b"y*yo"       , [0x02, 0x22        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpmovsxbw" = [
    b"yomq"       , [0x02, 0x20        ], X, VEX_OP | PREF_66, AVX;
    b"y*wo"       , [0x02, 0x20        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpmovsxdq" = [
    b"yomq"       , [0x02, 0x25        ], X, VEX_OP | PREF_66, AVX;
    b"y*wo"       , [0x02, 0x25        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpmovsxwd" = [
    b"yomq"       , [0x02, 0x23        ], X, VEX_OP | PREF_66, AVX;
    b"y*wo"       , [0x02, 0x23        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpmovsxwq" = [
    b"yomd"       , [0x02, 0x24        ], X, VEX_OP | PREF_66, AVX;
    b"y*yo"       , [0x02, 0x24        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpmovzxbd" = [
    b"yomd"       , [0x02, 0x31        ], X, VEX_OP | PREF_66, AVX;
    b"y*yo"       , [0x02, 0x31        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpmovzxbq" = [
    b"y*mw"       , [0x02, 0x32        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
    b"y*yo"       , [0x02, 0x32        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpmovzxbw" = [
    b"yomq"       , [0x02, 0x30        ], X, VEX_OP | PREF_66, AVX;
    b"y*wo"       , [0x02, 0x30        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpmovzxdq" = [
    b"yomq"       , [0x02, 0x35        ], X, VEX_OP | PREF_66, AVX;
    b"y*wo"       , [0x02, 0x35        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpmovzxwd" = [
    b"yomq"       , [0x02, 0x33        ], X, VEX_OP | PREF_66, AVX;
    b"y*wo"       , [0x02, 0x33        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpmovzxwq" = [
    b"yomd"       , [0x02, 0x34        ], X, VEX_OP | PREF_66, AVX;
    b"y*yo"       , [0x02, 0x34        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpmuldq" = [
    b"y*y*w*"     , [0x02, 0x28        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpmulhrsw" = [
    b"y*y*w*"     , [0x02, 0x0B        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpmulhuw" = [
    b"y*y*w*"     , [0x01, 0xE4        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpmulhw" = [
    b"y*y*w*"     , [0x01, 0xE5        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpmulld" = [
    b"y*y*w*"     , [0x02, 0x40        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpmullw" = [
    b"y*y*w*"     , [0x01, 0xD5        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpmuludq" = [
    b"y*y*w*"     , [0x01, 0xF4        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpor" = [
    b"y*y*w*"     , [0x01, 0xEB        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpperm" = [
    b"yoyowoyo"   , [0x08, 0xA3        ], X, XOP_OP, AMD | SSE5;
    b"yoyoyowo"   , [0x08, 0xA3        ], X, WITH_REXW | XOP_OP, SSE5 | AMD;
]
"vprotb" = [
    b"yowoib"     , [0x08, 0xC0        ], X, XOP_OP, SSE5 | AMD;
    b"yowoyo"     , [0x09, 0x90        ], X, XOP_OP | ENC_MR, AMD | SSE5;
    b"yoyowo"     , [0x09, 0x90        ], X, WITH_REXW | XOP_OP, SSE5 | AMD;
]
"vprotd" = [
    b"yowoib"     , [0x08, 0xC2        ], X, XOP_OP, AMD | SSE5;
    b"yowoyo"     , [0x09, 0x92        ], X, XOP_OP | ENC_MR, AMD | SSE5;
    b"yoyowo"     , [0x09, 0x92        ], X, WITH_REXW | XOP_OP, SSE5 | AMD;
]
"vprotq" = [
    b"yowoib"     , [0x08, 0xC3        ], X, XOP_OP, AMD | SSE5;
    b"yowoyo"     , [0x09, 0x93        ], X, XOP_OP | ENC_MR, AMD | SSE5;
    b"yoyowo"     , [0x09, 0x93        ], X, WITH_REXW | XOP_OP, SSE5 | AMD;
]
"vprotw" = [
    b"yowoib"     , [0x08, 0xC1        ], X, XOP_OP, AMD | SSE5;
    b"yowoyo"     , [0x09, 0x91        ], X, XOP_OP | ENC_MR, SSE5 | AMD;
    b"yoyowo"     , [0x09, 0x91        ], X, WITH_REXW | XOP_OP, AMD | SSE5;
]
"vpsadbw" = [
    b"y*y*w*"     , [0x01, 0xF6        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpshab" = [
    b"yowoyo"     , [0x09, 0x98        ], X, XOP_OP | ENC_MR, AMD | SSE5;
    b"yoyowo"     , [0x09, 0x98        ], X, WITH_REXW | XOP_OP, SSE5 | AMD;
]
"vpshad" = [
    b"yowoyo"     , [0x09, 0x9A        ], X, XOP_OP | ENC_MR, SSE5 | AMD;
    b"yoyowo"     , [0x09, 0x9A        ], X, WITH_REXW | XOP_OP, AMD | SSE5;
]
"vpshaq" = [
    b"yowoyo"     , [0x09, 0x9B        ], X, XOP_OP | ENC_MR, SSE5 | AMD;
    b"yoyowo"     , [0x09, 0x9B        ], X, WITH_REXW | XOP_OP, SSE5 | AMD;
]
"vpshaw" = [
    b"yowoyo"     , [0x09, 0x99        ], X, XOP_OP | ENC_MR, AMD | SSE5;
    b"yoyowo"     , [0x09, 0x99        ], X, WITH_REXW | XOP_OP, SSE5 | AMD;
]
"vpshlb" = [
    b"yowoyo"     , [0x09, 0x94        ], X, XOP_OP | ENC_MR, AMD | SSE5;
    b"yoyowo"     , [0x09, 0x94        ], X, WITH_REXW | XOP_OP, AMD | SSE5;
]
"vpshld" = [
    b"yowoyo"     , [0x09, 0x96        ], X, XOP_OP | ENC_MR, SSE5 | AMD;
    b"yoyowo"     , [0x09, 0x96        ], X, WITH_REXW | XOP_OP, SSE5 | AMD;
]
"vpshlq" = [
    b"yowoyo"     , [0x09, 0x97        ], X, XOP_OP | ENC_MR, AMD | SSE5;
    b"yoyowo"     , [0x09, 0x97        ], X, WITH_REXW | XOP_OP, AMD | SSE5;
]
"vpshlw" = [
    b"yowoyo"     , [0x09, 0x95        ], X, XOP_OP | ENC_MR, AMD | SSE5;
    b"yoyowo"     , [0x09, 0x95        ], X, WITH_REXW | XOP_OP, SSE5 | AMD;
]
"vpshufb" = [
    b"y*y*w*"     , [0x02, 0x00        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpshufd" = [
    b"y*w*ib"     , [0x01, 0x70        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpshufhw" = [
    b"y*w*ib"     , [0x01, 0x70        ], X, VEX_OP | AUTO_VEXL | PREF_F3, AVX;
]
"vpshuflw" = [
    b"y*w*ib"     , [0x01, 0x70        ], X, VEX_OP | AUTO_VEXL | PREF_F2, AVX;
]
"vpsignb" = [
    b"y*y*w*"     , [0x02, 0x08        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpsignd" = [
    b"y*y*w*"     , [0x02, 0x0A        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpsignw" = [
    b"y*y*w*"     , [0x02, 0x09        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpslld" = [
    b"y*y*ib"     , [0x01, 0x72        ], 6, VEX_OP | AUTO_VEXL | ENC_VM | PREF_66, AVX;
    b"y*y*wo"     , [0x01, 0xF2        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpslldq" = [
    b"y*y*ib"     , [0x01, 0x73        ], 7, VEX_OP | AUTO_VEXL | ENC_VM | PREF_66, AVX;
]
"vpsllq" = [
    b"y*y*ib"     , [0x01, 0x73        ], 6, VEX_OP | AUTO_VEXL | ENC_VM | PREF_66, AVX;
    b"y*y*wo"     , [0x01, 0xF3        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpsllvd" = [
    b"y*y*w*"     , [0x02, 0x47        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX2;
]
"vpsllvq" = [
    b"y*y*w*"     , [0x02, 0x47        ], X, VEX_OP | AUTO_VEXL | WITH_REXW | PREF_66, AVX2;
]
"vpsllw" = [
    b"y*y*ib"     , [0x01, 0x71        ], 6, VEX_OP | AUTO_VEXL | ENC_VM | PREF_66, AVX;
    b"y*y*wo"     , [0x01, 0xF1        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpsrad" = [
    b"y*y*ib"     , [0x01, 0x72        ], 4, VEX_OP | AUTO_VEXL | ENC_VM | PREF_66, AVX;
    b"y*y*wo"     , [0x01, 0xE2        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpsravd" = [
    b"y*y*w*"     , [0x02, 0x46        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX2;
]
"vpsraw" = [
    b"y*y*ib"     , [0x01, 0x71        ], 4, VEX_OP | AUTO_VEXL | ENC_VM | PREF_66, AVX;
    b"y*y*wo"     , [0x01, 0xE1        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpsrld" = [
    b"y*y*ib"     , [0x01, 0x72        ], 2, VEX_OP | AUTO_VEXL | ENC_VM | PREF_66, AVX;
    b"y*y*wo"     , [0x01, 0xD2        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpsrldq" = [
    b"y*y*ib"     , [0x01, 0x73        ], 3, VEX_OP | AUTO_VEXL | ENC_VM | PREF_66, AVX;
]
"vpsrlq" = [
    b"y*y*ib"     , [0x01, 0x73        ], 2, VEX_OP | ENC_VM | PREF_66, AVX;
    b"y*y*wo"     , [0x01, 0xD3        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpsrlvd" = [
    b"y*y*w*"     , [0x02, 0x45        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX2;
]
"vpsrlvq" = [
    b"y*y*w*"     , [0x02, 0x45        ], X, VEX_OP | AUTO_VEXL | WITH_REXW | PREF_66, AVX2;
]
"vpsrlw" = [
    b"y*y*ib"     , [0x01, 0x71        ], 2, VEX_OP | AUTO_VEXL | ENC_VM | PREF_66, AVX;
    b"y*y*wo"     , [0x01, 0xD1        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpsubb" = [
    b"y*y*w*"     , [0x01, 0xF8        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpsubd" = [
    b"y*y*w*"     , [0x01, 0xFA        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpsubq" = [
    b"y*y*w*"     , [0x01, 0xFB        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpsubsb" = [
    b"y*y*w*"     , [0x01, 0xE8        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpsubsw" = [
    b"y*y*w*"     , [0x01, 0xE9        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpsubusb" = [
    b"y*y*w*"     , [0x01, 0xD8        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpsubusw" = [
    b"y*y*w*"     , [0x01, 0xD9        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpsubw" = [
    b"y*y*w*"     , [0x01, 0xF9        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vptest" = [
    b"y*w*"       , [0x02, 0x17        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpunpckhbw" = [
    b"y*y*w*"     , [0x01, 0x68        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpunpckhdq" = [
    b"y*y*w*"     , [0x01, 0x6A        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpunpckhqdq" = [
    b"y*y*w*"     , [0x01, 0x6D        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpunpckhwd" = [
    b"y*y*w*"     , [0x01, 0x69        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpunpcklbw" = [
    b"y*y*w*"     , [0x01, 0x60        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpunpckldq" = [
    b"y*y*w*"     , [0x01, 0x62        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpunpcklqdq" = [
    b"y*y*w*"     , [0x01, 0x6C        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpunpcklwd" = [
    b"y*y*w*"     , [0x01, 0x61        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vpxor" = [
    b"y*y*w*"     , [0x01, 0xEF        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vrcpps" = [
    b"y*w*"       , [0x01, 0x53        ], X, VEX_OP | AUTO_VEXL, AVX;
]
"vrcpss" = [
    b"yoyomd"     , [0x01, 0x53        ], X, VEX_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0x53        ], X, VEX_OP | PREF_F3, AVX;
]
"vroundpd" = [
    b"y*w*ib"     , [0x03, 0x09        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vroundps" = [
    b"y*w*ib"     , [0x03, 0x08        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vroundsd" = [
    b"yoyomqib"   , [0x03, 0x0B        ], X, VEX_OP | PREF_66, AVX;
    b"yoyoyoib"   , [0x03, 0x0B        ], X, VEX_OP | PREF_66, AVX;
]
"vroundss" = [
    b"yoyomdib"   , [0x03, 0x0A        ], X, VEX_OP | PREF_66, AVX;
    b"yoyoyoib"   , [0x03, 0x0A        ], X, VEX_OP | PREF_66, AVX;
]
"vrsqrtps" = [
    b"y*w*"       , [0x01, 0x52        ], X, VEX_OP | AUTO_VEXL, AVX;
]
"vrsqrtss" = [
    b"yoyomd"     , [0x01, 0x52        ], X, VEX_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0x52        ], X, VEX_OP | PREF_F3, AVX;
]
"vshufpd" = [
    b"y*y*w*ib"   , [0x01, 0xC6        ], X, VEX_OP | AUTO_VEXL | ENC_MR | PREF_66, AVX;
]
"vshufps" = [
    b"y*y*w*ib"   , [0x01, 0xC6        ], X, VEX_OP | AUTO_VEXL | ENC_MR, AVX;
]
"vsqrtpd" = [
    b"y*w*"       , [0x01, 0x51        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vsqrtps" = [
    b"y*w*"       , [0x01, 0x51        ], X, VEX_OP | AUTO_VEXL, AVX;
]
"vsqrtsd" = [
    b"yoyomq"     , [0x01, 0x51        ], X, VEX_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0x51        ], X, VEX_OP | PREF_F2, AVX;
]
"vsqrtss" = [
    b"yoyomd"     , [0x01, 0x51        ], X, VEX_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0x51        ], X, VEX_OP | PREF_F3, AVX;
]
"vstmxcsr" = [
    b"md"         , [0x01, 0xAE        ], 3, VEX_OP, AVX;
]
"vsubpd" = [
    b"y*y*w*"     , [0x01, 0x5C        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vsubps" = [
    b"y*y*w*"     , [0x01, 0x5C        ], X, VEX_OP | AUTO_VEXL, AVX;
]
"vsubsd" = [
    b"yoyomq"     , [0x01, 0x5C        ], X, VEX_OP | PREF_F2, AVX;
    b"yoyoyo"     , [0x01, 0x5C        ], X, VEX_OP | PREF_F2, AVX;
]
"vsubss" = [
    b"yoyomd"     , [0x01, 0x5C        ], X, VEX_OP | PREF_F3, AVX;
    b"yoyoyo"     , [0x01, 0x5C        ], X, VEX_OP | PREF_F3, AVX;
]
"vtestpd" = [
    b"y*w*"       , [0x02, 0x0F        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vtestps" = [
    b"y*w*"       , [0x02, 0x0E        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vucomisd" = [
    b"yomq"       , [0x01, 0x2E        ], X, VEX_OP | PREF_66, AVX;
    b"yoyo"       , [0x01, 0x2E        ], X, VEX_OP | PREF_66, AVX;
]
"vucomiss" = [
    b"yomd"       , [0x01, 0x2E        ], X, VEX_OP, AVX;
    b"yoyo"       , [0x01, 0x2E        ], X, VEX_OP, AVX;
]
"vunpckhpd" = [
    b"y*y*w*"     , [0x01, 0x15        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vunpckhps" = [
    b"y*y*w*"     , [0x01, 0x15        ], X, VEX_OP | AUTO_VEXL, AVX;
]
"vunpcklpd" = [
    b"y*y*w*"     , [0x01, 0x14        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vunpcklps" = [
    b"y*y*w*"     , [0x01, 0x14        ], X, VEX_OP | AUTO_VEXL, AVX;
]
"vxorpd" = [
    b"y*y*w*"     , [0x01, 0x57        ], X, VEX_OP | AUTO_VEXL | PREF_66, AVX;
]
"vxorps" = [
    b"y*y*w*"     , [0x01, 0x57        ], X, VEX_OP | AUTO_VEXL, AVX;
]
"vzeroall" = [
    b""           , [0x01, 0x77        ], X, WITH_VEXL | VEX_OP, AVX;
]
"vzeroupper" = [
    b""           , [0x01, 0x77        ], X, VEX_OP, AVX;
]
"wbinvd" = [
    b""           , [0x0F, 0x09        ], X;
]
"wrfsbase" = [
    b"rd"         , [0x0F, 0xAE        ], 2, PREF_F3;
    b"rq"         , [0x0F, 0xAE        ], 2, WITH_REXW | PREF_F3;
]
"wrgsbase" = [
    b"rd"         , [0x0F, 0xAE        ], 3, PREF_F3;
    b"rq"         , [0x0F, 0xAE        ], 3, WITH_REXW | PREF_F3;
]
"wrmsr" = [
    b""           , [0x0F, 0x30        ], X;
]
"wrpkru" = [
    b""           , [0x0F, 0x01, 0xEF  ], X;
]
"wrshr" = [
    b"vd"         , [0x0F, 0x37        ], 0, DEFAULT, CYRIX;
]
"xabort" = [
    b"ib"         , [0xC6, 0xF8        ], X, DEFAULT, RTM;
]
"xadd" = [
    b"mbrb"       , [0x0F, 0xC0        ], X, LOCK | ENC_MR;
    b"rbrb"       , [0x0F, 0xC0        ], X, ENC_MR;
    b"m*r*"       , [0x0F, 0xC1        ], X, AUTO_SIZE | LOCK | ENC_MR;
    b"r*r*"       , [0x0F, 0xC1        ], X, AUTO_SIZE | ENC_MR;
]
"xbegin" = [
    b"od"         , [0xC7, 0xF8        ], X, DEFAULT, RTM;
]
"xchg" = [
    b"mbrb"       , [0x86              ], X, LOCK | ENC_MR;
    b"rbmb"       , [0x86              ], X, LOCK;
    b"rbrb"       , [0x86              ], X;
    b"rbrb"       , [0x86              ], X, ENC_MR;
    b"A*r*"       , [0x90              ], X, AUTO_SIZE | SHORT_ARG;
    b"m*r*"       , [0x87              ], X, AUTO_SIZE | ENC_MR;
    b"r*A*"       , [0x90              ], X, AUTO_SIZE | SHORT_ARG;
    b"r*m*"       , [0x87              ], X, AUTO_SIZE;
    b"r*r*"       , [0x87              ], X, AUTO_SIZE;
    b"r*r*"       , [0x87              ], X, AUTO_SIZE | ENC_MR;
]
"xcryptcbc" = [
    b""           , [0x0F, 0xA7, 0xD0  ], X, PREF_F3, CYRIX;
]
"xcryptcfb" = [
    b""           , [0x0F, 0xA7, 0xE0  ], X, PREF_F3, CYRIX;
]
"xcryptctr" = [
    b""           , [0x0F, 0xA7, 0xD8  ], X, PREF_F3, CYRIX;
]
"xcryptecb" = [
    b""           , [0x0F, 0xA7, 0xC8  ], X, PREF_F3, CYRIX;
]
"xcryptofb" = [
    b""           , [0x0F, 0xA7, 0xE8  ], X, PREF_F3, CYRIX;
]
"xend" = [
    b""           , [0x0F, 0x01, 0xD5  ], X, DEFAULT, RTM;
]
"xgetbv" = [
    b""           , [0x0F, 0x01, 0xD0  ], X;
]
"xlat" = [
    b""           , [0xD7              ], X;
]
"xlatb" = [
    b""           , [0xD7              ], X;
]
"xor" = [
    b"Abib"       , [0x34              ], X;
    b"mbib"       , [0x80              ], 6, LOCK;
    b"mbrb"       , [0x30              ], X, LOCK | ENC_MR;
    b"rbib"       , [0x80              ], 6;
    b"rbrb"       , [0x30              ], X, ENC_MR;
    b"rbvb"       , [0x32              ], X;
    b"r*ib"       , [0x83              ], 6, AUTO_SIZE  | EXACT_SIZE;
    b"A*i*"       , [0x35              ], X, AUTO_SIZE;
    b"m*i*"       , [0x81              ], 6, AUTO_SIZE | LOCK;
    b"m*ib"       , [0x83              ], 6, AUTO_SIZE | LOCK;
    b"m*r*"       , [0x31              ], X, AUTO_SIZE | LOCK | ENC_MR;
    b"r*i*"       , [0x81              ], 6, AUTO_SIZE ;
    b"r*r*"       , [0x31              ], X, AUTO_SIZE | ENC_MR;
    b"r*v*"       , [0x33              ], X, AUTO_SIZE;
]
"xorpd" = [
    b"yowo"       , [0x0F, 0x57        ], X, PREF_66, SSE2;
]
"xorps" = [
    b"yowo"       , [0x0F, 0x57        ], X, DEFAULT, SSE;
]
"xrstor" = [
    b"m!"         , [0x0F, 0xAE        ], 5;
]
"xrstor64" = [
    b"m!"         , [0x0F, 0xAE        ], 5, WITH_REXW;
]
"xrstors64" = [
    b"m!"         , [0x0F, 0xC7        ], 3, WITH_REXW;
]
"xsave" = [
    b"m!"         , [0x0F, 0xAE        ], 4;
]
"xsave64" = [
    b"m!"         , [0x0F, 0xAE        ], 4, WITH_REXW;
]
"xsavec64" = [
    b"m!"         , [0x0F, 0xC7        ], 4, WITH_REXW;
]
"xsaveopt64" = [
    b"m!"         , [0x0F, 0xAE        ], 6, WITH_REXW;
]
"xsaves64" = [
    b"m!"         , [0x0F, 0xC7        ], 5, WITH_REXW;
]
"xsetbv" = [
    b""           , [0x0F, 0x01, 0xD1  ], X;
]
"xsha1" = [
    b""           , [0x0F, 0xA6, 0xC8  ], X, PREF_F3, CYRIX;
]
"xsha256" = [
    b""           , [0x0F, 0xA6, 0xD0  ], X, PREF_F3, CYRIX;
]
"xstore" = [
    b""           , [0x0F, 0xA7, 0xC0  ], X, DEFAULT, CYRIX;
]
"xtest" = [
    b""           , [0x0F, 0x01, 0xD6  ], X, DEFAULT, RTM;
]

"call"  = [
    b"iwiw"       , [0x9A              ], X, X86_ONLY | WORD_SIZE | EXACT_SIZE;
    b"idiw"       , [0x9A              ], X, X86_ONLY;
    b"mf"         , [0xFF              ], 3, X86_ONLY | EXACT_SIZE;
    b"od"         , [0xE8              ], X;
    b"v*"         , [0xFF              ], 2, AUTO_NO32;
]
"callf" = [
    b"iwiw"       , [0x9A              ], X, X86_ONLY | WORD_SIZE | EXACT_SIZE;
    b"idiw"       , [0x9A              ], X, X86_ONLY;
    b"md"         , [0xFF              ], 3, X86_ONLY | WORD_SIZE | EXACT_SIZE;
    b"mf"         , [0xFF              ], 3, X86_ONLY;
]
"jmp"   = [
    b"iwiw"       , [0x9A              ], X, X86_ONLY | WORD_SIZE | EXACT_SIZE;
    b"idiw"       , [0xEA              ], X, X86_ONLY;
    b"mf"         , [0xFF              ], 5, X86_ONLY | EXACT_SIZE;
    b"ob"         , [0xEB              ], X, EXACT_SIZE;
    b"od"         , [0xE9              ], X;
    b"v*"         , [0xFF              ], 4, AUTO_NO32 ;
]
"jmpf" = [
    b"iwiw"       , [0x9A              ], X, X86_ONLY | WORD_SIZE | EXACT_SIZE;
    b"idiw"       , [0xEA              ], X, X86_ONLY;
    b"md"         , [0xFF              ], 5, X86_ONLY | WORD_SIZE | EXACT_SIZE;
    b"mf"         , [0xFF              ], 5, X86_ONLY;
]
"mov"   = [
    b"v*r*"       , [0x89              ], X, AUTO_SIZE;
    b"vbrb"       , [0x88              ], X;
    b"r*v*"       , [0x8B              ], X, AUTO_SIZE;
    b"rbvb"       , [0x8A              ], X;
    b"r*sw"       , [0x8C              ], X, AUTO_SIZE;
    b"mwsw"       , [0x8C              ], X;
    b"swmw"       , [0x8C              ], X;
    b"swrw"       , [0x8C              ], X;
    b"rbib"       , [0xB0              ], X,             SHORT_ARG;
    b"rwiw"       , [0xB8              ], X, WORD_SIZE | SHORT_ARG;
    b"rdid"       , [0xB8              ], X,             SHORT_ARG;
    b"v*i*"       , [0xC7              ], 0, AUTO_SIZE;
    b"vbib"       , [0xC6              ], 0;
    b"rqiq"       , [0xB8              ], X, WITH_REXW | SHORT_ARG;
    b"cdrd"       , [0x0F, 0x22        ], X; // can only match in 32 bit mode due to "cd"
    b"cqrq"       , [0x0F, 0x22        ], X; // doesn't need a prefix to be encoded, as it's 64 bit natural in 64 bit mode
    b"rdcd"       , [0x0F, 0x20        ], X;
    b"rqcq"       , [0x0F, 0x20        ], X;
    b"Wdrd"       , [0x0F, 0x22        ], 0, PREF_F0; // note: technically CR8 should actually be encoded, but the encoding is 0.
    b"Wqrq"       , [0x0F, 0x22        ], 0, PREF_F0;
    b"rdWd"       , [0x0F, 0x22        ], 0, PREF_F0;
    b"rqWq"       , [0x0F, 0x22        ], 0, PREF_F0;
    b"ddrd"       , [0x0F, 0x23        ], X; // 32 bit mode only
    b"dqrq"       , [0x0F, 0x23        ], X;
    b"rddd"       , [0x0F, 0x21        ], X;
    b"rqdq"       , [0x0F, 0x21        ], X;
]
"movabs"  = [
    b"Abiq"       , [0xA0              ], X; // special syntax for 64-bit disp only mov
    b"Awiq"       , [0xA1              ], X, WORD_SIZE;
    b"Adiq"       , [0xA1              ], X;
    b"Aqiq"       , [0xA1              ], X, WITH_REXW;
    b"iqAb"       , [0xA2              ], X;
    b"iqAw"       , [0xA3              ], X, WORD_SIZE;
    b"iqAd"       , [0xA3              ], X;
    b"iqAq"       , [0xA3              ], X, WITH_REXW;
]
"jo"     = [
    b"ob"         , [0x70            ], X, EXACT_SIZE;
    b"od"         , [0x0F, 0x80      ], X;
]
"jno"    = [
    b"ob"         , [0x71            ], X, EXACT_SIZE;
    b"od"         , [0x0F, 0x81      ], X;
]
"jb"     = [
    b"ob",       [0x72            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x82      ], X;
]
"jc"     = [
    b"ob",       [0x72            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x82      ], X;
]
"jnae"   = [
    b"ob",       [0x72            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x82      ], X;
]
"jnb"    = [
    b"ob",       [0x73            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x83      ], X;
]
"jnc"    = [
    b"ob",       [0x73            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x83      ], X;
]
"jae"    = [
    b"ob",       [0x73            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x83      ], X;
]
"jz"     = [
    b"ob",       [0x74            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x84      ], X;
]
"je"     = [
    b"ob",       [0x74            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x84      ], X;
]
"jnz"    = [
    b"ob",       [0x75            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x85      ], X;
]
"jne"    = [
    b"ob",       [0x75            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x85      ], X;
]
"jbe"    = [
    b"ob",       [0x76            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x86      ], X;
]
"jna"    = [
    b"ob",       [0x76            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x86      ], X;
]
"jnbe"   = [
    b"ob",       [0x77            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x87      ], X;
]
"ja"     = [
    b"ob",       [0x77            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x87      ], X;
]
"js"     = [
    b"ob",       [0x78            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x88      ], X;
]
"jns"    = [
    b"ob",       [0x79            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x89      ], X;
]
"jp"     = [
    b"ob",       [0x7A            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x8A      ], X;
]
"jpe"    = [
    b"ob",       [0x7A            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x8A      ], X;
]
"jnp"    = [
    b"ob",       [0x7B            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x8B      ], X;
]
"jpo"    = [
    b"ob",       [0x7B            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x8B      ], X;
]
"jl"     = [
    b"ob",       [0x7C            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x8C      ], X;
]
"jnge"   = [
    b"ob",       [0x7C            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x8C      ], X;
]
"jnl"    = [
    b"ob",       [0x7D            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x8D      ], X;
]
"jge"    = [
    b"ob",       [0x7D            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x8D      ], X;
]
"jle"    = [
    b"ob",       [0x7E            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x8E      ], X;
]
"jng"    = [
    b"ob",       [0x7E            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x8E      ], X;
]
"jnle"   = [
    b"ob",       [0x7F            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x8F      ], X;
]
"jg"     = [
    b"ob",       [0x7F            ], X, EXACT_SIZE;
    b"od",       [0x0F, 0x8F      ], X;
]

"cmovo"    = [
    b"r*v*",     [0x0F, 0x40      ], X, AUTO_SIZE;
]
"cmovno"   = [
    b"r*v*",     [0x0F, 0x41      ], X, AUTO_SIZE;
]
"cmovb"    = [
    b"r*v*",     [0x0F, 0x42      ], X, AUTO_SIZE;
]
"cmovc"    = [
    b"r*v*",     [0x0F, 0x42      ], X, AUTO_SIZE;
]
"cmovnae"  = [
    b"r*v*",     [0x0F, 0x42      ], X, AUTO_SIZE;
]
"cmovnb"      = [
    b"r*v*",     [0x0F, 0x43      ], X, AUTO_SIZE;
]
"cmovnc"      = [
    b"r*v*",     [0x0F, 0x43      ], X, AUTO_SIZE;
]
"cmovae"      = [
    b"r*v*",     [0x0F, 0x43      ], X, AUTO_SIZE;
]
"cmovz"       = [
    b"r*v*",     [0x0F, 0x44      ], X, AUTO_SIZE;
]
"cmove"       = [
    b"r*v*",     [0x0F, 0x44      ], X, AUTO_SIZE;
]
"cmovnz"      = [
    b"r*v*",     [0x0F, 0x45      ], X, AUTO_SIZE;
]
"cmovne"      = [
    b"r*v*",     [0x0F, 0x45      ], X, AUTO_SIZE;
]
"cmovbe"      = [
    b"r*v*",     [0x0F, 0x46      ], X, AUTO_SIZE;
]
"cmovna"      = [
    b"r*v*",     [0x0F, 0x46      ], X, AUTO_SIZE;
]
"cmovnbe"     = [
    b"r*v*",     [0x0F, 0x47      ], X, AUTO_SIZE;
]
"cmova"       = [
    b"r*v*",     [0x0F, 0x47      ], X, AUTO_SIZE;
]
"cmovs"       = [
    b"r*v*",     [0x0F, 0x48      ], X, AUTO_SIZE;
]
"cmovns"      = [
    b"r*v*",     [0x0F, 0x49      ], X, AUTO_SIZE;
]
"cmovp"       = [
    b"r*v*",     [0x0F, 0x4A      ], X, AUTO_SIZE;
]
"cmovpe"      = [
    b"r*v*",     [0x0F, 0x4A      ], X, AUTO_SIZE;
]
"cmovnp"      = [
    b"r*v*",     [0x0F, 0x4B      ], X, AUTO_SIZE;
]
"cmovpo"      = [
    b"r*v*",     [0x0F, 0x4B      ], X, AUTO_SIZE;
]
"cmovl"       = [
    b"r*v*",     [0x0F, 0x4C      ], X, AUTO_SIZE;
]
"cmovnge"     = [
    b"r*v*",     [0x0F, 0x4C      ], X, AUTO_SIZE;
]
"cmovnl"      = [
    b"r*v*",     [0x0F, 0x4D      ], X, AUTO_SIZE;
]
"cmovge"      = [
    b"r*v*",     [0x0F, 0x4D      ], X, AUTO_SIZE;
]
"cmovle"      = [
    b"r*v*",     [0x0F, 0x4E      ], X, AUTO_SIZE;
]
"cmovng"      = [
    b"r*v*",     [0x0F, 0x4E      ], X, AUTO_SIZE;
]
"cmovnle"     = [
    b"r*v*",     [0x0F, 0x4F      ], X, AUTO_SIZE;
]
"cmovg"       = [
    b"r*v*",     [0x0F, 0x4F      ], X, AUTO_SIZE;
]

"seto"        = [
    b"vb",       [0x0F, 0x90      ], 0;
]
"setno"       = [
    b"vb",       [0x0F, 0x91      ], 0;
]
"setb"        = [
    b"vb",       [0x0F, 0x92      ], 0;
]
"setc"        = [
    b"vb",       [0x0F, 0x92      ], 0;
]
"setnae"      = [
    b"vb",       [0x0F, 0x92      ], 0;
]
"setnb"       = [
    b"vb",       [0x0F, 0x93      ], 0;
]
"setnc"       = [
    b"vb",       [0x0F, 0x93      ], 0;
]
"setae"       = [
    b"vb",       [0x0F, 0x93      ], 0;
]
"setz"        = [
    b"vb",       [0x0F, 0x94      ], 0;
]
"sete"        = [
    b"vb",       [0x0F, 0x94      ], 0;
]
"setnz"       = [
    b"vb",       [0x0F, 0x95      ], 0;
]
"setne"       = [
    b"vb",       [0x0F, 0x95      ], 0;
]
"setbe"       = [
    b"vb",       [0x0F, 0x96      ], 0;
]
"setna"       = [
    b"vb",       [0x0F, 0x96      ], 0;
]
"setnbe"      = [
    b"vb",       [0x0F, 0x97      ], 0;
]
"seta"        = [
    b"vb",       [0x0F, 0x97      ], 0;
]
"sets"        = [
    b"vb",       [0x0F, 0x98      ], 0;
]
"setns"       = [
    b"vb",       [0x0F, 0x99      ], 0;
]
"setp"        = [
    b"vb",       [0x0F, 0x9A      ], 0;
]
"setpe"       = [
    b"vb",       [0x0F, 0x9A      ], 0;
]
"setnp"       = [
    b"vb",       [0x0F, 0x9B      ], 0;
]
"setpo"       = [
    b"vb",       [0x0F, 0x9B      ], 0;
]
"setl"        = [
    b"vb",       [0x0F, 0x9C      ], 0;
]
"setnge"      = [
    b"vb",       [0x0F, 0x9C      ], 0;
]
"setnl"       = [
    b"vb",       [0x0F, 0x9D      ], 0;
]
"setge"       = [
    b"vb",       [0x0F, 0x9D      ], 0;
]
"setle"       = [
    b"vb",       [0x0F, 0x9E      ], 0;
]
"setng"       = [
    b"vb",       [0x0F, 0x9E      ], 0;
]
"setnle"      = [
    b"vb",       [0x0F, 0x9F      ], 0;
]
"setg"        = [
    b"vb",       [0x0F, 0x9F      ], 0;
]


"in"    = [
    b"Abib"       , [0xE4            ], X;
    b"Awib"       , [0xE5            ], X, WORD_SIZE;
    b"Adib"       , [0xE5            ], X;
    b"AbCw"       , [0xEC            ], X;
    b"AwCw"       , [0xED            ], X, WORD_SIZE;
    b"AdCw"       , [0xED            ], X;
]

"out"   = [
    b"ibAb"       , [0xE6            ], X;
    b"ibAw"       , [0xE7            ], X;
    b"ibAd"       , [0xE7            ], X;
    b"CwAb"       , [0xEE            ], X;
    b"CwAw"       , [0xEF            ], X, WORD_SIZE;
    b"CwAd"       , [0xEF            ], X;
]

"crc32"  = [
    b"r*vb"       , [0x0F, 0x38, 0xF0], X, AUTO_REXW | PREF_F2 | EXACT_SIZE; // unique size encoding scheme
    b"rdvw"       , [0x0F, 0x38, 0xF1], X, WORD_SIZE | PREF_F2 | EXACT_SIZE;
    b"r*v*"       , [0x0F, 0x38, 0xF1], X, AUTO_REXW | PREF_F2 | EXACT_SIZE;
]

"imul"   = [
    b"v*"         , [0xF7            ], 5, AUTO_SIZE;
    b"vb"         , [0xF6            ], 5;
    b"r*v*"       , [0x0F, 0xAF      ], X, AUTO_SIZE;
    b"r*v*ib"     , [0x6B            ], X, AUTO_SIZE | EXACT_SIZE;
    b"r*v*i*"     , [0x69            ], X, AUTO_SIZE;
]

)
