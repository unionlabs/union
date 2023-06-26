/*
 * Copyright Supranational LLC
 * Licensed under the Apache License, Version 2.0, see LICENSE for details.
 * SPDX-License-Identifier: Apache-2.0
 */

#include "point.h"
#include "fields.h"

/*
 * y^2 = x^3 + A'*x + B', isogenous one
 */
static const vec384 Aprime_E1 = {
    /* (0x00144698a3b8e9433d693a02c96d4982b0ea985383ee66a8
          d8e8981aefd881ac98936f8da0e0f97f5cf428082d584c1d << 384) % P */
    TO_LIMB_T(0x2f65aa0e9af5aa51), TO_LIMB_T(0x86464c2d1e8416c3),
    TO_LIMB_T(0xb85ce591b7bd31e2), TO_LIMB_T(0x27e11c91b5f24e7c),
    TO_LIMB_T(0x28376eda6bfc1835), TO_LIMB_T(0x155455c3e5071d85)
};
static const vec384 Bprime_E1 = {
    /* (0x12e2908d11688030018b12e8753eee3b2016c1f0f24f4070
          a0b9c14fcef35ef55a23215a316ceaa5d1cc48e98e172be0 << 384) % P */
    TO_LIMB_T(0xfb996971fe22a1e0), TO_LIMB_T(0x9aa93eb35b742d6f),
    TO_LIMB_T(0x8c476013de99c5c4), TO_LIMB_T(0x873e27c3a221e571),
    TO_LIMB_T(0xca72b5e45a52d888), TO_LIMB_T(0x06824061418a386b)
};

static void map_fp_times_Zz(vec384 map[], const vec384 isogeny_map[],
                            const vec384 Zz_powers[], size_t n)
{
    while (n--)
        mul_fp(map[n], isogeny_map[n], Zz_powers[n]);
}

static void map_fp(vec384 acc, const vec384 x, const vec384 map[], size_t n)
{
    while (n--) {
        mul_fp(acc, acc, x);
        add_fp(acc, acc, map[n]);
    }
}

static void isogeny_map_to_E1(POINTonE1 *out, const POINTonE1 *p)
{
    /*
     * x = x_num / x_den, where
     * x_num = k_(1,11) * x'^11 + k_(1,10) * x'^10 + k_(1,9) * x'^9 +
     *         ... + k_(1,0)
     * ...
     */
    static const vec384 isogeny_map_x_num[] = { /*  (k_(1,*)<<384) % P  */
      { TO_LIMB_T(0x4d18b6f3af00131c), TO_LIMB_T(0x19fa219793fee28c),
        TO_LIMB_T(0x3f2885f1467f19ae), TO_LIMB_T(0x23dcea34f2ffb304),
        TO_LIMB_T(0xd15b58d2ffc00054), TO_LIMB_T(0x0913be200a20bef4)  },
      { TO_LIMB_T(0x898985385cdbbd8b), TO_LIMB_T(0x3c79e43cc7d966aa),
        TO_LIMB_T(0x1597e193f4cd233a), TO_LIMB_T(0x8637ef1e4d6623ad),
        TO_LIMB_T(0x11b22deed20d827b), TO_LIMB_T(0x07097bc5998784ad)  },
      { TO_LIMB_T(0xa542583a480b664b), TO_LIMB_T(0xfc7169c026e568c6),
        TO_LIMB_T(0x5ba2ef314ed8b5a6), TO_LIMB_T(0x5b5491c05102f0e7),
        TO_LIMB_T(0xdf6e99707d2a0079), TO_LIMB_T(0x0784151ed7605524)  },
      { TO_LIMB_T(0x494e212870f72741), TO_LIMB_T(0xab9be52fbda43021),
        TO_LIMB_T(0x26f5577994e34c3d), TO_LIMB_T(0x049dfee82aefbd60),
        TO_LIMB_T(0x65dadd7828505289), TO_LIMB_T(0x0e93d431ea011aeb)  },
      { TO_LIMB_T(0x90ee774bd6a74d45), TO_LIMB_T(0x7ada1c8a41bfb185),
        TO_LIMB_T(0x0f1a8953b325f464), TO_LIMB_T(0x104c24211be4805c),
        TO_LIMB_T(0x169139d319ea7a8f), TO_LIMB_T(0x09f20ead8e532bf6)  },
      { TO_LIMB_T(0x6ddd93e2f43626b7), TO_LIMB_T(0xa5482c9aa1ccd7bd),
        TO_LIMB_T(0x143245631883f4bd), TO_LIMB_T(0x2e0a94ccf77ec0db),
        TO_LIMB_T(0xb0282d480e56489f), TO_LIMB_T(0x18f4bfcbb4368929)  },
      { TO_LIMB_T(0x23c5f0c953402dfd), TO_LIMB_T(0x7a43ff6958ce4fe9),
        TO_LIMB_T(0x2c390d3d2da5df63), TO_LIMB_T(0xd0df5c98e1f9d70f),
        TO_LIMB_T(0xffd89869a572b297), TO_LIMB_T(0x1277ffc72f25e8fe)  },
      { TO_LIMB_T(0x79f4f0490f06a8a6), TO_LIMB_T(0x85f894a88030fd81),
        TO_LIMB_T(0x12da3054b18b6410), TO_LIMB_T(0xe2a57f6505880d65),
        TO_LIMB_T(0xbba074f260e400f1), TO_LIMB_T(0x08b76279f621d028)  },
      { TO_LIMB_T(0xe67245ba78d5b00b), TO_LIMB_T(0x8456ba9a1f186475),
        TO_LIMB_T(0x7888bff6e6b33bb4), TO_LIMB_T(0xe21585b9a30f86cb),
        TO_LIMB_T(0x05a69cdcef55feee), TO_LIMB_T(0x09e699dd9adfa5ac)  },
      { TO_LIMB_T(0x0de5c357bff57107), TO_LIMB_T(0x0a0db4ae6b1a10b2),
        TO_LIMB_T(0xe256bb67b3b3cd8d), TO_LIMB_T(0x8ad456574e9db24f),
        TO_LIMB_T(0x0443915f50fd4179), TO_LIMB_T(0x098c4bf7de8b6375)  },
      { TO_LIMB_T(0xe6b0617e7dd929c7), TO_LIMB_T(0xfe6e37d442537375),
        TO_LIMB_T(0x1dafdeda137a489e), TO_LIMB_T(0xe4efd1ad3f767ceb),
        TO_LIMB_T(0x4a51d8667f0fe1cf), TO_LIMB_T(0x054fdf4bbf1d821c)  },
      { TO_LIMB_T(0x72db2a50658d767b), TO_LIMB_T(0x8abf91faa257b3d5),
        TO_LIMB_T(0xe969d6833764ab47), TO_LIMB_T(0x464170142a1009eb),
        TO_LIMB_T(0xb14f01aadb30be2f), TO_LIMB_T(0x18ae6a856f40715d)  }
    };
    /* ...
     * x_den = x'^10 + k_(2,9) * x'^9 + k_(2,8) * x'^8 + ... + k_(2,0)
     */
    static const vec384 isogeny_map_x_den[] = { /*  (k_(2,*)<<384) % P  */
      { TO_LIMB_T(0xb962a077fdb0f945), TO_LIMB_T(0xa6a9740fefda13a0),
        TO_LIMB_T(0xc14d568c3ed6c544), TO_LIMB_T(0xb43fc37b908b133e),
        TO_LIMB_T(0x9c0b3ac929599016), TO_LIMB_T(0x0165aa6c93ad115f)  },
      { TO_LIMB_T(0x23279a3ba506c1d9), TO_LIMB_T(0x92cfca0a9465176a),
        TO_LIMB_T(0x3b294ab13755f0ff), TO_LIMB_T(0x116dda1c5070ae93),
        TO_LIMB_T(0xed4530924cec2045), TO_LIMB_T(0x083383d6ed81f1ce)  },
      { TO_LIMB_T(0x9885c2a6449fecfc), TO_LIMB_T(0x4a2b54ccd37733f0),
        TO_LIMB_T(0x17da9ffd8738c142), TO_LIMB_T(0xa0fba72732b3fafd),
        TO_LIMB_T(0xff364f36e54b6812), TO_LIMB_T(0x0f29c13c660523e2)  },
      { TO_LIMB_T(0xe349cc118278f041), TO_LIMB_T(0xd487228f2f3204fb),
        TO_LIMB_T(0xc9d325849ade5150), TO_LIMB_T(0x43a92bd69c15c2df),
        TO_LIMB_T(0x1c2c7844bc417be4), TO_LIMB_T(0x12025184f407440c)  },
      { TO_LIMB_T(0x587f65ae6acb057b), TO_LIMB_T(0x1444ef325140201f),
        TO_LIMB_T(0xfbf995e71270da49), TO_LIMB_T(0xccda066072436a42),
        TO_LIMB_T(0x7408904f0f186bb2), TO_LIMB_T(0x13b93c63edf6c015)  },
      { TO_LIMB_T(0xfb918622cd141920), TO_LIMB_T(0x4a4c64423ecaddb4),
        TO_LIMB_T(0x0beb232927f7fb26), TO_LIMB_T(0x30f94df6f83a3dc2),
        TO_LIMB_T(0xaeedd424d780f388), TO_LIMB_T(0x06cc402dd594bbeb)  },
      { TO_LIMB_T(0xd41f761151b23f8f), TO_LIMB_T(0x32a92465435719b3),
        TO_LIMB_T(0x64f436e888c62cb9), TO_LIMB_T(0xdf70a9a1f757c6e4),
        TO_LIMB_T(0x6933a38d5b594c81), TO_LIMB_T(0x0c6f7f7237b46606)  },
      { TO_LIMB_T(0x693c08747876c8f7), TO_LIMB_T(0x22c9850bf9cf80f0),
        TO_LIMB_T(0x8e9071dab950c124), TO_LIMB_T(0x89bc62d61c7baf23),
        TO_LIMB_T(0xbc6be2d8dad57c23), TO_LIMB_T(0x17916987aa14a122)  },
      { TO_LIMB_T(0x1be3ff439c1316fd), TO_LIMB_T(0x9965243a7571dfa7),
        TO_LIMB_T(0xc7f7f62962f5cd81), TO_LIMB_T(0x32c6aa9af394361c),
        TO_LIMB_T(0xbbc2ee18e1c227f4), TO_LIMB_T(0x0c102cbac531bb34)  },
      { TO_LIMB_T(0x997614c97bacbf07), TO_LIMB_T(0x61f86372b99192c0),
        TO_LIMB_T(0x5b8c95fc14353fc3), TO_LIMB_T(0xca2b066c2a87492f),
        TO_LIMB_T(0x16178f5bbf698711), TO_LIMB_T(0x12a6dcd7f0f4e0e8)  }
    };
    /*
     * y = y' * y_num / y_den, where
     * y_num = k_(3,15) * x'^15 + k_(3,14) * x'^14 + k_(3,13) * x'^13 +
     *         ... + k_(3,0)
     * ...
     */
    static const vec384 isogeny_map_y_num[] = { /*  (k_(3,*)<<384) % P  */
      { TO_LIMB_T(0x2b567ff3e2837267), TO_LIMB_T(0x1d4d9e57b958a767),
        TO_LIMB_T(0xce028fea04bd7373), TO_LIMB_T(0xcc31a30a0b6cd3df),
        TO_LIMB_T(0x7d7b18a682692693), TO_LIMB_T(0x0d300744d42a0310)  },
      { TO_LIMB_T(0x99c2555fa542493f), TO_LIMB_T(0xfe7f53cc4874f878),
        TO_LIMB_T(0x5df0608b8f97608a), TO_LIMB_T(0x14e03832052b49c8),
        TO_LIMB_T(0x706326a6957dd5a4), TO_LIMB_T(0x0a8dadd9c2414555)  },
      { TO_LIMB_T(0x13d942922a5cf63a), TO_LIMB_T(0x357e33e36e261e7d),
        TO_LIMB_T(0xcf05a27c8456088d), TO_LIMB_T(0x0000bd1de7ba50f0),
        TO_LIMB_T(0x83d0c7532f8c1fde), TO_LIMB_T(0x13f70bf38bbf2905)  },
      { TO_LIMB_T(0x5c57fd95bfafbdbb), TO_LIMB_T(0x28a359a65e541707),
        TO_LIMB_T(0x3983ceb4f6360b6d), TO_LIMB_T(0xafe19ff6f97e6d53),
        TO_LIMB_T(0xb3468f4550192bf7), TO_LIMB_T(0x0bb6cde49d8ba257)  },
      { TO_LIMB_T(0x590b62c7ff8a513f), TO_LIMB_T(0x314b4ce372cacefd),
        TO_LIMB_T(0x6bef32ce94b8a800), TO_LIMB_T(0x6ddf84a095713d5f),
        TO_LIMB_T(0x64eace4cb0982191), TO_LIMB_T(0x0386213c651b888d)  },
      { TO_LIMB_T(0xa5310a31111bbcdd), TO_LIMB_T(0xa14ac0f5da148982),
        TO_LIMB_T(0xf9ad9cc95423d2e9), TO_LIMB_T(0xaa6ec095283ee4a7),
        TO_LIMB_T(0xcf5b1f022e1c9107), TO_LIMB_T(0x01fddf5aed881793)  },
      { TO_LIMB_T(0x65a572b0d7a7d950), TO_LIMB_T(0xe25c2d8183473a19),
        TO_LIMB_T(0xc2fcebe7cb877dbd), TO_LIMB_T(0x05b2d36c769a89b0),
        TO_LIMB_T(0xba12961be86e9efb), TO_LIMB_T(0x07eb1b29c1dfde1f)  },
      { TO_LIMB_T(0x93e09572f7c4cd24), TO_LIMB_T(0x364e929076795091),
        TO_LIMB_T(0x8569467e68af51b5), TO_LIMB_T(0xa47da89439f5340f),
        TO_LIMB_T(0xf4fa918082e44d64), TO_LIMB_T(0x0ad52ba3e6695a79)  },
      { TO_LIMB_T(0x911429844e0d5f54), TO_LIMB_T(0xd03f51a3516bb233),
        TO_LIMB_T(0x3d587e5640536e66), TO_LIMB_T(0xfa86d2a3a9a73482),
        TO_LIMB_T(0xa90ed5adf1ed5537), TO_LIMB_T(0x149c9c326a5e7393)  },
      { TO_LIMB_T(0x462bbeb03c12921a), TO_LIMB_T(0xdc9af5fa0a274a17),
        TO_LIMB_T(0x9a558ebde836ebed), TO_LIMB_T(0x649ef8f11a4fae46),
        TO_LIMB_T(0x8100e1652b3cdc62), TO_LIMB_T(0x1862bd62c291dacb)  },
      { TO_LIMB_T(0x05c9b8ca89f12c26), TO_LIMB_T(0x0194160fa9b9ac4f),
        TO_LIMB_T(0x6a643d5a6879fa2c), TO_LIMB_T(0x14665bdd8846e19d),
        TO_LIMB_T(0xbb1d0d53af3ff6bf), TO_LIMB_T(0x12c7e1c3b28962e5)  },
      { TO_LIMB_T(0xb55ebf900b8a3e17), TO_LIMB_T(0xfedc77ec1a9201c4),
        TO_LIMB_T(0x1f07db10ea1a4df4), TO_LIMB_T(0x0dfbd15dc41a594d),
        TO_LIMB_T(0x389547f2334a5391), TO_LIMB_T(0x02419f98165871a4)  },
      { TO_LIMB_T(0xb416af000745fc20), TO_LIMB_T(0x8e563e9d1ea6d0f5),
        TO_LIMB_T(0x7c763e17763a0652), TO_LIMB_T(0x01458ef0159ebbef),
        TO_LIMB_T(0x8346fe421f96bb13), TO_LIMB_T(0x0d2d7b829ce324d2)  },
      { TO_LIMB_T(0x93096bb538d64615), TO_LIMB_T(0x6f2a2619951d823a),
        TO_LIMB_T(0x8f66b3ea59514fa4), TO_LIMB_T(0xf563e63704f7092f),
        TO_LIMB_T(0x724b136c4cf2d9fa), TO_LIMB_T(0x046959cfcfd0bf49)  },
      { TO_LIMB_T(0xea748d4b6e405346), TO_LIMB_T(0x91e9079c2c02d58f),
        TO_LIMB_T(0x41064965946d9b59), TO_LIMB_T(0xa06731f1d2bbe1ee),
        TO_LIMB_T(0x07f897e267a33f1b), TO_LIMB_T(0x1017290919210e5f)  },
      { TO_LIMB_T(0x872aa6c17d985097), TO_LIMB_T(0xeecc53161264562a),
        TO_LIMB_T(0x07afe37afff55002), TO_LIMB_T(0x54759078e5be6838),
        TO_LIMB_T(0xc4b92d15db8acca8), TO_LIMB_T(0x106d87d1b51d13b9)  }
    };
    /* ...
     * y_den = x'^15 + k_(4,14) * x'^14 + k_(4,13) * x'^13 + ... + k_(4,0)
     */
    static const vec384 isogeny_map_y_den[] = { /*  (k_(4,*)<<384) % P  */
      { TO_LIMB_T(0xeb6c359d47e52b1c), TO_LIMB_T(0x18ef5f8a10634d60),
        TO_LIMB_T(0xddfa71a0889d5b7e), TO_LIMB_T(0x723e71dcc5fc1323),
        TO_LIMB_T(0x52f45700b70d5c69), TO_LIMB_T(0x0a8b981ee47691f1)  },
      { TO_LIMB_T(0x616a3c4f5535b9fb), TO_LIMB_T(0x6f5f037395dbd911),
        TO_LIMB_T(0xf25f4cc5e35c65da), TO_LIMB_T(0x3e50dffea3c62658),
        TO_LIMB_T(0x6a33dca523560776), TO_LIMB_T(0x0fadeff77b6bfe3e)  },
      { TO_LIMB_T(0x2be9b66df470059c), TO_LIMB_T(0x24a2c159a3d36742),
        TO_LIMB_T(0x115dbe7ad10c2a37), TO_LIMB_T(0xb6634a652ee5884d),
        TO_LIMB_T(0x04fe8bb2b8d81af4), TO_LIMB_T(0x01c2a7a256fe9c41)  },
      { TO_LIMB_T(0xf27bf8ef3b75a386), TO_LIMB_T(0x898b367476c9073f),
        TO_LIMB_T(0x24482e6b8c2f4e5f), TO_LIMB_T(0xc8e0bbd6fe110806),
        TO_LIMB_T(0x59b0c17f7631448a), TO_LIMB_T(0x11037cd58b3dbfbd)  },
      { TO_LIMB_T(0x31c7912ea267eec6), TO_LIMB_T(0x1dbf6f1c5fcdb700),
        TO_LIMB_T(0xd30d4fe3ba86fdb1), TO_LIMB_T(0x3cae528fbee9a2a4),
        TO_LIMB_T(0xb1cce69b6aa9ad9a), TO_LIMB_T(0x044393bb632d94fb)  },
      { TO_LIMB_T(0xc66ef6efeeb5c7e8), TO_LIMB_T(0x9824c289dd72bb55),
        TO_LIMB_T(0x71b1a4d2f119981d), TO_LIMB_T(0x104fc1aafb0919cc),
        TO_LIMB_T(0x0e49df01d942a628), TO_LIMB_T(0x096c3a09773272d4)  },
      { TO_LIMB_T(0x9abc11eb5fadeff4), TO_LIMB_T(0x32dca50a885728f0),
        TO_LIMB_T(0xfb1fa3721569734c), TO_LIMB_T(0xc4b76271ea6506b3),
        TO_LIMB_T(0xd466a75599ce728e), TO_LIMB_T(0x0c81d4645f4cb6ed)  },
      { TO_LIMB_T(0x4199f10e5b8be45b), TO_LIMB_T(0xda64e495b1e87930),
        TO_LIMB_T(0xcb353efe9b33e4ff), TO_LIMB_T(0x9e9efb24aa6424c6),
        TO_LIMB_T(0xf08d33680a237465), TO_LIMB_T(0x0d3378023e4c7406)  },
      { TO_LIMB_T(0x7eb4ae92ec74d3a5), TO_LIMB_T(0xc341b4aa9fac3497),
        TO_LIMB_T(0x5be603899e907687), TO_LIMB_T(0x03bfd9cca75cbdeb),
        TO_LIMB_T(0x564c2935a96bfa93), TO_LIMB_T(0x0ef3c33371e2fdb5)  },
      { TO_LIMB_T(0x7ee91fd449f6ac2e), TO_LIMB_T(0xe5d5bd5cb9357a30),
        TO_LIMB_T(0x773a8ca5196b1380), TO_LIMB_T(0xd0fda172174ed023),
        TO_LIMB_T(0x6cb95e0fa776aead), TO_LIMB_T(0x0d22d5a40cec7cff)  },
      { TO_LIMB_T(0xf727e09285fd8519), TO_LIMB_T(0xdc9d55a83017897b),
        TO_LIMB_T(0x7549d8bd057894ae), TO_LIMB_T(0x178419613d90d8f8),
        TO_LIMB_T(0xfce95ebdeb5b490a), TO_LIMB_T(0x0467ffaef23fc49e)  },
      { TO_LIMB_T(0xc1769e6a7c385f1b), TO_LIMB_T(0x79bc930deac01c03),
        TO_LIMB_T(0x5461c75a23ede3b5), TO_LIMB_T(0x6e20829e5c230c45),
        TO_LIMB_T(0x828e0f1e772a53cd), TO_LIMB_T(0x116aefa749127bff)  },
      { TO_LIMB_T(0x101c10bf2744c10a), TO_LIMB_T(0xbbf18d053a6a3154),
        TO_LIMB_T(0xa0ecf39ef026f602), TO_LIMB_T(0xfc009d4996dc5153),
        TO_LIMB_T(0xb9000209d5bd08d3), TO_LIMB_T(0x189e5fe4470cd73c)  },
      { TO_LIMB_T(0x7ebd546ca1575ed2), TO_LIMB_T(0xe47d5a981d081b55),
        TO_LIMB_T(0x57b2b625b6d4ca21), TO_LIMB_T(0xb0a1ba04228520cc),
        TO_LIMB_T(0x98738983c2107ff3), TO_LIMB_T(0x13dddbc4799d81d6)  },
      { TO_LIMB_T(0x09319f2e39834935), TO_LIMB_T(0x039e952cbdb05c21),
        TO_LIMB_T(0x55ba77a9a2f76493), TO_LIMB_T(0xfd04e3dfc6086467),
        TO_LIMB_T(0xfb95832e7d78742e), TO_LIMB_T(0x0ef9c24eccaf5e0e)  }
    };
    vec384 Zz_powers[15], map[15], xn, xd, yn, yd;

    /* lay down Z^2 powers in descending order                          */
    sqr_fp(Zz_powers[14], p->Z);                        /* ZZ^1         */
#ifdef __OPTIMIZE_SIZE__
    for (size_t i = 14; i > 0; i--)
        mul_fp(Zz_powers[i-1], Zz_powers[i], Zz_powers[14]);
#else
    sqr_fp(Zz_powers[13], Zz_powers[14]);               /* ZZ^2  1+1    */
    mul_fp(Zz_powers[12], Zz_powers[14], Zz_powers[13]);/* ZZ^3  2+1    */
    sqr_fp(Zz_powers[11], Zz_powers[13]);               /* ZZ^4  2+2    */
    mul_fp(Zz_powers[10], Zz_powers[13], Zz_powers[12]);/* ZZ^5  2+3    */
    sqr_fp(Zz_powers[9],  Zz_powers[12]);               /* ZZ^6  3+3    */
    mul_fp(Zz_powers[8],  Zz_powers[12], Zz_powers[11]);/* ZZ^7  3+4    */
    sqr_fp(Zz_powers[7],  Zz_powers[11]);               /* ZZ^8  4+4    */
    mul_fp(Zz_powers[6],  Zz_powers[11], Zz_powers[10]);/* ZZ^9  4+5    */
    sqr_fp(Zz_powers[5],  Zz_powers[10]);               /* ZZ^10 5+5    */
    mul_fp(Zz_powers[4],  Zz_powers[10], Zz_powers[9]); /* ZZ^11 5+6    */
    sqr_fp(Zz_powers[3],  Zz_powers[9]);                /* ZZ^12 6+6    */
    mul_fp(Zz_powers[2],  Zz_powers[9],  Zz_powers[8]); /* ZZ^13 6+7    */
    sqr_fp(Zz_powers[1],  Zz_powers[8]);                /* ZZ^14 7+7    */
    mul_fp(Zz_powers[0],  Zz_powers[8],  Zz_powers[7]); /* ZZ^15 7+8    */
#endif

    map_fp_times_Zz(map, isogeny_map_x_num, Zz_powers + 4, 11);
    mul_fp(xn, p->X, isogeny_map_x_num[11]);
    add_fp(xn, xn, map[10]);
    map_fp(xn, p->X, map, 10);

    map_fp_times_Zz(map, isogeny_map_x_den, Zz_powers + 5, 10);
    add_fp(xd, p->X, map[9]);
    map_fp(xd, p->X, map, 9);
    mul_fp(xd, xd, Zz_powers[14]);      /* xd *= Z^2                    */

    map_fp_times_Zz(map, isogeny_map_y_num, Zz_powers, 15);
    mul_fp(yn, p->X, isogeny_map_y_num[15]);
    add_fp(yn, yn, map[14]);
    map_fp(yn, p->X, map, 14);
    mul_fp(yn, yn, p->Y);               /* yn *= Y                      */

    map_fp_times_Zz(map, isogeny_map_y_den, Zz_powers, 15);
    add_fp(yd, p->X, map[14]);
    map_fp(yd, p->X, map, 14);
    mul_fp(Zz_powers[14], Zz_powers[14], p->Z);
    mul_fp(yd, yd, Zz_powers[14]);      /* yd *= Z^3                    */

    /* convert (xn, xd, yn, yd) to Jacobian coordinates                 */
    mul_fp(out->Z, xd, yd);             /* Z = xd * yd                  */
    mul_fp(out->X, xn, yd);
    mul_fp(out->X, out->X, out->Z);     /* X = xn * xd * yd^2           */
    sqr_fp(out->Y, out->Z);
    mul_fp(out->Y, out->Y, xd);
    mul_fp(out->Y, out->Y, yn);         /* Y = yn * xd^3 * yd^2         */
}

static void map_to_isogenous_E1(POINTonE1 *p, const vec384 u)
{
    static const vec384 minus_A = { /* P - A */
        TO_LIMB_T(0x8a9955f1650a005a), TO_LIMB_T(0x9865b3d192cfe93c),
        TO_LIMB_T(0xaed3ed0f3ef3c441), TO_LIMB_T(0x3c962ef33d92c442),
        TO_LIMB_T(0x22e438dbd74f94a2), TO_LIMB_T(0x04acbc265478c915)
    };
    static const vec384 Z = {       /* (11<<384) % P */
        TO_LIMB_T(0x886c00000023ffdc), TO_LIMB_T(0x0f70008d3090001d),
        TO_LIMB_T(0x77672417ed5828c3), TO_LIMB_T(0x9dac23e943dc1740),
        TO_LIMB_T(0x50553f1b9c131521), TO_LIMB_T(0x078c712fbe0ab6e8)
    };
    static const vec384 sqrt_minus_ZZZ = {
        TO_LIMB_T(0x43b571cad3215f1f), TO_LIMB_T(0xccb460ef1c702dc2),
        TO_LIMB_T(0x742d884f4f97100b), TO_LIMB_T(0xdb2c3e3238a3382b),
        TO_LIMB_T(0xe40f3fa13fce8f88), TO_LIMB_T(0x0073a2af9892a2ff)
    };
    static const vec384 ZxA = {
        TO_LIMB_T(0x7f674ea0a8915178), TO_LIMB_T(0xb0f945fc13b8fa65),
        TO_LIMB_T(0x4b46759a38e87d76), TO_LIMB_T(0x2e7a929641bbb6a1),
        TO_LIMB_T(0x1668ddfa462bf6b6), TO_LIMB_T(0x00960e2ed1cf294c)
    };
    vec384 uu, tv2, x2n, gx1, gxd, y2;
#if 0
    vec384 xn, x1n, xd, y, y1, Zuu, tv4;
#else
# define xn     p->X
# define y      p->Y
# define xd     p->Z
# define x1n    xn
# define y1     y
# define Zuu    x2n
# define tv4    y1
#endif
#define sgn0_fp(a) (sgn0_pty_mont_384((a), BLS12_381_P, p0) & 1)
    bool_t e1, e2;

    /*
     * as per map_to_curve() from poc/sswu_opt.sage at
     * https://github.com/cfrg/draft-irtf-cfrg-hash-to-curve
     */
    /* x numerator variants                                             */
    sqr_fp(uu, u);                      /* uu = u^2                     */
    mul_fp(Zuu, Z, uu);                 /* Zuu = Z * uu                 */
    sqr_fp(tv2, Zuu);                   /* tv2 = Zuu^2                  */
    add_fp(tv2, tv2, Zuu);              /* tv2 = tv2 + Zuu              */
    add_fp(x1n, tv2, BLS12_381_Rx.p);   /* x1n = tv2 + 1                */
    mul_fp(x1n, x1n, Bprime_E1);        /* x1n = x1n * B                */
    mul_fp(x2n, Zuu, x1n);              /* x2n = Zuu * x1n              */

    /* x denumenator                                                    */
    mul_fp(xd, minus_A, tv2);           /* xd = -A * tv2                */
    e1 = vec_is_zero(xd, sizeof(xd));   /* e1 = xd == 0                 */
    vec_select(xd, ZxA, xd, sizeof(xd), e1);    /*              # If xd == 0, set xd = Z*A */

    /* y numerators variants                                            */
    sqr_fp(tv2, xd);                    /* tv2 = xd^2                   */
    mul_fp(gxd, xd, tv2);               /* gxd = xd^3                   */
    mul_fp(tv2, Aprime_E1, tv2);        /* tv2 = A * tv2                */
    sqr_fp(gx1, x1n);                   /* gx1 = x1n^2                  */
    add_fp(gx1, gx1, tv2);              /* gx1 = gx1 + tv2      # x1n^2 + A*xd^2 */
    mul_fp(gx1, gx1, x1n);              /* gx1 = gx1 * x1n      # x1n^3 + A*x1n*xd^2 */
    mul_fp(tv2, Bprime_E1, gxd);        /* tv2 = B * gxd                */
    add_fp(gx1, gx1, tv2);              /* gx1 = gx1 + tv2      # x1^3 + A*x1*xd^2 + B*xd^3 */
    sqr_fp(tv4, gxd);                   /* tv4 = gxd^2                  */
    mul_fp(tv2, gx1, gxd);              /* tv2 = gx1 * gxd              */
    mul_fp(tv4, tv4, tv2);              /* tv4 = tv4 * tv2      # gx1*gxd^3 */
    e2 = recip_sqrt_fp(y1, tv4);        /* y1 = tv4^c1          # (gx1*gxd^3)^((p-3)/4) */
    mul_fp(y1, y1, tv2);                /* y1 = y1 * tv2        # gx1*gxd*y1 */
    mul_fp(y2, y1, sqrt_minus_ZZZ);     /* y2 = y1 * c2         # y2 = y1*sqrt(-Z^3) */
    mul_fp(y2, y2, uu);                 /* y2 = y2 * uu                 */
    mul_fp(y2, y2, u);                  /* y2 = y2 * u                  */

    /* choose numerators                                                */
    vec_select(xn, x1n, x2n, sizeof(xn), e2);   /* xn = e2 ? x1n : x2n  */
    vec_select(y, y1, y2, sizeof(y), e2);       /* y  = e2 ? y1 : y2    */

    e1 = sgn0_fp(u);
    e2 = sgn0_fp(y);
    cneg_fp(y, y, e1^e2);               /* fix sign of y                */
                                        /* return (xn, xd, y, 1)        */

    /* convert (xn, xd, y, 1) to Jacobian projective coordinates        */
    mul_fp(p->X, xn, xd);               /* X = xn * xd                  */
    mul_fp(p->Y, y, gxd);               /* Y = y * xd^3                 */
#ifndef xd
    vec_copy(p->Z, xd, sizeof(xd));     /* Z = xd                       */
#else
# undef xn
# undef y
# undef xd
# undef x1n
# undef y1
# undef Zuu
# undef tv4
#endif
#undef sgn0_fp
}

static void POINTonE1_add_n_dbl(POINTonE1 *out, const POINTonE1 *p, size_t n)
{
    POINTonE1_dadd(out, out, p, NULL);
    while(n--)
        POINTonE1_double(out, out);
}

static void POINTonE1_times_minus_z(POINTonE1 *out, const POINTonE1 *in)
{
    POINTonE1_double(out, in);          /*      1: 0x2                  */
    POINTonE1_add_n_dbl(out, in, 2);    /*   2..4: 0x3..0xc             */
    POINTonE1_add_n_dbl(out, in, 3);    /*   5..8: 0xd..0x68            */
    POINTonE1_add_n_dbl(out, in, 9);    /*  9..18: 0x69..0xd200         */
    POINTonE1_add_n_dbl(out, in, 32);   /* 19..51: ..0xd20100000000     */
    POINTonE1_add_n_dbl(out, in, 16);   /* 52..68: ..0xd201000000010000 */
}

/*
 * |u|, |v| are expected to be in Montgomery representation
 */
static void map_to_g1(POINTonE1 *out, const vec384 u, const vec384 v)
{
    POINTonE1 p;

    map_to_isogenous_E1(&p, u);

    if (v != NULL) {
        map_to_isogenous_E1(out, v);    /* borrow |out|                 */
        POINTonE1_dadd(&p, &p, out, Aprime_E1);
    }

    isogeny_map_to_E1(&p, &p);          /* sprinkle isogenous powder    */

    /* clear the cofactor by multiplying |p| by 1-z, 0xd201000000010001 */
    POINTonE1_times_minus_z(out, &p);
    POINTonE1_dadd(out, out, &p, NULL);
}

void blst_map_to_g1(POINTonE1 *out, const vec384 u, const vec384 v)
{   map_to_g1(out, u, v);   }

static void Encode_to_G1(POINTonE1 *p, const unsigned char *msg, size_t msg_len,
                                       const unsigned char *DST, size_t DST_len,
                                       const unsigned char *aug, size_t aug_len)
{
    vec384 u[1];

    hash_to_field(u, 1, aug, aug_len, msg, msg_len, DST, DST_len);
    map_to_g1(p, u[0], NULL);
}

void blst_encode_to_g1(POINTonE1 *p, const unsigned char *msg, size_t msg_len,
                                     const unsigned char *DST, size_t DST_len,
                                     const unsigned char *aug, size_t aug_len)
{   Encode_to_G1(p, msg, msg_len, DST, DST_len, aug, aug_len);   }

static void Hash_to_G1(POINTonE1 *p, const unsigned char *msg, size_t msg_len,
                                     const unsigned char *DST, size_t DST_len,
                                     const unsigned char *aug, size_t aug_len)
{
    vec384 u[2];

    hash_to_field(u, 2, aug, aug_len, msg, msg_len, DST, DST_len);
    map_to_g1(p, u[0], u[1]);
}

void blst_hash_to_g1(POINTonE1 *p, const unsigned char *msg, size_t msg_len,
                                   const unsigned char *DST, size_t DST_len,
                                   const unsigned char *aug, size_t aug_len)
{   Hash_to_G1(p, msg, msg_len, DST, DST_len, aug, aug_len);   }

static void sigma(POINTonE1 *out, const POINTonE1 *in);

#if 0
#ifdef __OPTIMIZE_SIZE__
static void POINTonE1_times_zz_minus_1_div_by_3(POINTonE1 *out,
                                                const POINTonE1 *in)
{
    static const byte zz_minus_1_div_by_3[] = {
        TO_BYTES(0x0000000055555555ULL), TO_BYTES(0x396c8c005555e156)
    };
    size_t n = 126-1;
    const POINTonE1 *dblin = in;

    while(n--) {
        POINTonE1_double(out, dblin);   dblin = out;
        if (is_bit_set(zz_minus_1_div_by_3, n))
            POINTonE1_dadd(out, out, in, NULL);
    }
}
#else
static void POINTonE1_dbl_n_add(POINTonE1 *out, size_t n, const POINTonE1 *p)
{
    while(n--)
        POINTonE1_double(out, out);
    POINTonE1_dadd(out, out, p, NULL);
}

static void POINTonE1_times_zz_minus_1_div_by_3(POINTonE1 *out,
                                                const POINTonE1 *in)
{
    POINTonE1 t3, t5, t7, t11, t85;

    POINTonE1_double(&t7, in);              /* 2P */
    POINTonE1_dadd(&t3, &t7, in, NULL);     /* 3P */
    POINTonE1_dadd(&t5, &t3, &t7, NULL);    /* 5P */
    POINTonE1_dadd(&t7, &t5, &t7, NULL);    /* 7P */
    POINTonE1_double(&t85, &t5);            /* 10P */
    POINTonE1_dadd(&t11, &t85, in, NULL);   /* 11P */
    POINTonE1_dbl_n_add(&t85, 3, &t5);      /* 0x55P */
                                            /* (-0xd201000000010000^2 - 1) / 3 */
    POINTonE1_double(out, &t7);             /* 0xe */
    POINTonE1_dbl_n_add(out, 5,  &t11);     /* 0x1cb */
    POINTonE1_dbl_n_add(out, 3,  &t3);      /* 0xe5b */
    POINTonE1_dbl_n_add(out, 3,  in);       /* 0x72d9 */
    POINTonE1_dbl_n_add(out, 5,  &t3);      /* 0xe5b23 */
    POINTonE1_dbl_n_add(out, 18, &t85);     /* 0x396c8c0055 */
    POINTonE1_dbl_n_add(out, 8,  &t85);     /* 0x396c8c005555 */
    POINTonE1_dbl_n_add(out, 3,  &t7);      /* 0x1cb646002aaaf */
    POINTonE1_dbl_n_add(out, 7,  &t5);      /* 0xe5b23001555785 */
    POINTonE1_dbl_n_add(out, 5,  &t11);     /* 0x1cb646002aaaf0ab */
    POINTonE1_dbl_n_add(out, 41, &t85);     /* 0x396c8c005555e1560000000055 */
    POINTonE1_dbl_n_add(out, 8,  &t85);     /* 0x396c8c005555e156000000005555 */
    POINTonE1_dbl_n_add(out, 8,  &t85);     /* 0x396c8c005555e15600000000555555 */
    POINTonE1_dbl_n_add(out, 8,  &t85);     /* 0x396c8c005555e1560000000055555555 */
}
#endif

static bool_t POINTonE1_in_G1(const POINTonE1 *P)
{
    POINTonE1 t0, t1, t2;

    /* Bowe, S., "Faster subgroup checks for BLS12-381"                   */
    sigma(&t0, P);                        /* σ(P)                         */
    sigma(&t1, &t0);                      /* σ²(P)                        */

    POINTonE1_double(&t0, &t0);           /* 2σ(P)                        */
    POINTonE1_dadd(&t2, &t1, P, NULL);    /* P +  σ²(P)                   */
    POINTonE1_cneg(&t2, 1);               /* - P - σ²(P)                  */
    POINTonE1_dadd(&t2, &t2, &t0, NULL);  /* 2σ(P) - P - σ²(P)            */
    POINTonE1_times_zz_minus_1_div_by_3(  &t0, &t2);
    POINTonE1_cneg(&t1, 1);
    POINTonE1_dadd(&t0, &t0, &t1, NULL);  /* [(z²-1)/3](2σ(P) - P - σ²(P)) */
                                          /* - σ²(P) */
    return vec_is_zero(t0.Z, sizeof(t0.Z));
}
#else
static bool_t POINTonE1_in_G1(const POINTonE1 *P)
{
    POINTonE1 t0, t1;

    /* Scott, M., https://eprint.iacr.org/2021/1130 */
    POINTonE1_times_minus_z(&t0, P);
    POINTonE1_times_minus_z(&t1, &t0);
    POINTonE1_cneg(&t1, 1);             /* [-z²]P   */

    sigma(&t0, P);                      /* σ(P)     */
    sigma(&t0, &t0);                    /* σ²(P)    */

    return POINTonE1_is_equal(&t0, &t1);
}
#endif

int blst_p1_in_g1(const POINTonE1 *p)
{   return (int)POINTonE1_in_G1(p);   }

int blst_p1_affine_in_g1(const POINTonE1_affine *p)
{
    POINTonE1 P;

    vec_copy(P.X, p->X, 2*sizeof(P.X));
    vec_select(P.Z, p->X, BLS12_381_Rx.p, sizeof(P.Z),
                     vec_is_zero(p, sizeof(*p)));

    return (int)POINTonE1_in_G1(&P);
}
