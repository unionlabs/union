// Copyright 2020 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package bls12381

// isogenyMapG1 applies 11-isogeny map for BLS12-381 G1 defined at draft-irtf-cfrg-hash-to-curve-06.
func isogenyMapG1(x, y *fe) {
	// https://tools.ietf.org/html/draft-irtf-cfrg-hash-to-curve-06#appendix-C.2
	params := isogenyConstantsG1
	degree := 15
	xNum, xDen, yNum, yDen := new(fe), new(fe), new(fe), new(fe)
	xNum.set(params[0][degree])
	xDen.set(params[1][degree])
	yNum.set(params[2][degree])
	yDen.set(params[3][degree])
	for i := degree - 1; i >= 0; i-- {
		mul(xNum, xNum, x)
		mul(xDen, xDen, x)
		mul(yNum, yNum, x)
		mul(yDen, yDen, x)
		add(xNum, xNum, params[0][i])
		add(xDen, xDen, params[1][i])
		add(yNum, yNum, params[2][i])
		add(yDen, yDen, params[3][i])
	}
	inverse(xDen, xDen)
	inverse(yDen, yDen)
	mul(xNum, xNum, xDen)
	mul(yNum, yNum, yDen)
	mul(yNum, yNum, y)
	x.set(xNum)
	y.set(yNum)
}

// isogenyMapG2 applies 11-isogeny map for BLS12-381 G1 defined at draft-irtf-cfrg-hash-to-curve-06.
func isogenyMapG2(e *fp2, x, y *fe2) {
	if e == nil {
		e = newFp2()
	}
	// https://tools.ietf.org/html/draft-irtf-cfrg-hash-to-curve-06#appendix-C.2
	params := isogenyConstantsG2
	degree := 3
	xNum := new(fe2).set(params[0][degree])
	xDen := new(fe2).set(params[1][degree])
	yNum := new(fe2).set(params[2][degree])
	yDen := new(fe2).set(params[3][degree])
	for i := degree - 1; i >= 0; i-- {
		e.mul(xNum, xNum, x)
		e.mul(xDen, xDen, x)
		e.mul(yNum, yNum, x)
		e.mul(yDen, yDen, x)
		e.add(xNum, xNum, params[0][i])
		e.add(xDen, xDen, params[1][i])
		e.add(yNum, yNum, params[2][i])
		e.add(yDen, yDen, params[3][i])
	}
	e.inverse(xDen, xDen)
	e.inverse(yDen, yDen)
	e.mul(xNum, xNum, xDen)
	e.mul(yNum, yNum, yDen)
	e.mul(yNum, yNum, y)
	x.set(xNum)
	y.set(yNum)
}

var isogenyConstantsG1 = [4][16]*fe{
	{
		{0x4d18b6f3af00131c, 0x19fa219793fee28c, 0x3f2885f1467f19ae, 0x23dcea34f2ffb304, 0xd15b58d2ffc00054, 0x0913be200a20bef4},
		{0x898985385cdbbd8b, 0x3c79e43cc7d966aa, 0x1597e193f4cd233a, 0x8637ef1e4d6623ad, 0x11b22deed20d827b, 0x07097bc5998784ad},
		{0xa542583a480b664b, 0xfc7169c026e568c6, 0x5ba2ef314ed8b5a6, 0x5b5491c05102f0e7, 0xdf6e99707d2a0079, 0x0784151ed7605524},
		{0x494e212870f72741, 0xab9be52fbda43021, 0x26f5577994e34c3d, 0x049dfee82aefbd60, 0x65dadd7828505289, 0x0e93d431ea011aeb},
		{0x90ee774bd6a74d45, 0x7ada1c8a41bfb185, 0x0f1a8953b325f464, 0x104c24211be4805c, 0x169139d319ea7a8f, 0x09f20ead8e532bf6},
		{0x6ddd93e2f43626b7, 0xa5482c9aa1ccd7bd, 0x143245631883f4bd, 0x2e0a94ccf77ec0db, 0xb0282d480e56489f, 0x18f4bfcbb4368929},
		{0x23c5f0c953402dfd, 0x7a43ff6958ce4fe9, 0x2c390d3d2da5df63, 0xd0df5c98e1f9d70f, 0xffd89869a572b297, 0x1277ffc72f25e8fe},
		{0x79f4f0490f06a8a6, 0x85f894a88030fd81, 0x12da3054b18b6410, 0xe2a57f6505880d65, 0xbba074f260e400f1, 0x08b76279f621d028},
		{0xe67245ba78d5b00b, 0x8456ba9a1f186475, 0x7888bff6e6b33bb4, 0xe21585b9a30f86cb, 0x05a69cdcef55feee, 0x09e699dd9adfa5ac},
		{0x0de5c357bff57107, 0x0a0db4ae6b1a10b2, 0xe256bb67b3b3cd8d, 0x8ad456574e9db24f, 0x0443915f50fd4179, 0x098c4bf7de8b6375},
		{0xe6b0617e7dd929c7, 0xfe6e37d442537375, 0x1dafdeda137a489e, 0xe4efd1ad3f767ceb, 0x4a51d8667f0fe1cf, 0x054fdf4bbf1d821c},
		{0x72db2a50658d767b, 0x8abf91faa257b3d5, 0xe969d6833764ab47, 0x464170142a1009eb, 0xb14f01aadb30be2f, 0x18ae6a856f40715d},
		{0, 0, 0, 0, 0, 0},
		{0, 0, 0, 0, 0, 0},
		{0, 0, 0, 0, 0, 0},
		{0, 0, 0, 0, 0, 0},
	},
	{
		{0xb962a077fdb0f945, 0xa6a9740fefda13a0, 0xc14d568c3ed6c544, 0xb43fc37b908b133e, 0x9c0b3ac929599016, 0x0165aa6c93ad115f},
		{0x23279a3ba506c1d9, 0x92cfca0a9465176a, 0x3b294ab13755f0ff, 0x116dda1c5070ae93, 0xed4530924cec2045, 0x083383d6ed81f1ce},
		{0x9885c2a6449fecfc, 0x4a2b54ccd37733f0, 0x17da9ffd8738c142, 0xa0fba72732b3fafd, 0xff364f36e54b6812, 0x0f29c13c660523e2},
		{0xe349cc118278f041, 0xd487228f2f3204fb, 0xc9d325849ade5150, 0x43a92bd69c15c2df, 0x1c2c7844bc417be4, 0x12025184f407440c},
		{0x587f65ae6acb057b, 0x1444ef325140201f, 0xfbf995e71270da49, 0xccda066072436a42, 0x7408904f0f186bb2, 0x13b93c63edf6c015},
		{0xfb918622cd141920, 0x4a4c64423ecaddb4, 0x0beb232927f7fb26, 0x30f94df6f83a3dc2, 0xaeedd424d780f388, 0x06cc402dd594bbeb},
		{0xd41f761151b23f8f, 0x32a92465435719b3, 0x64f436e888c62cb9, 0xdf70a9a1f757c6e4, 0x6933a38d5b594c81, 0x0c6f7f7237b46606},
		{0x693c08747876c8f7, 0x22c9850bf9cf80f0, 0x8e9071dab950c124, 0x89bc62d61c7baf23, 0xbc6be2d8dad57c23, 0x17916987aa14a122},
		{0x1be3ff439c1316fd, 0x9965243a7571dfa7, 0xc7f7f62962f5cd81, 0x32c6aa9af394361c, 0xbbc2ee18e1c227f4, 0x0c102cbac531bb34},
		{0x997614c97bacbf07, 0x61f86372b99192c0, 0x5b8c95fc14353fc3, 0xca2b066c2a87492f, 0x16178f5bbf698711, 0x12a6dcd7f0f4e0e8},
		{0x760900000002fffd, 0xebf4000bc40c0002, 0x5f48985753c758ba, 0x77ce585370525745, 0x5c071a97a256ec6d, 0x15f65ec3fa80e493},
		{0, 0, 0, 0, 0, 0},
		{0, 0, 0, 0, 0, 0},
		{0, 0, 0, 0, 0, 0},
		{0, 0, 0, 0, 0, 0},
		{0, 0, 0, 0, 0, 0},
	},
	{
		{0x2b567ff3e2837267, 0x1d4d9e57b958a767, 0xce028fea04bd7373, 0xcc31a30a0b6cd3df, 0x7d7b18a682692693, 0x0d300744d42a0310},
		{0x99c2555fa542493f, 0xfe7f53cc4874f878, 0x5df0608b8f97608a, 0x14e03832052b49c8, 0x706326a6957dd5a4, 0x0a8dadd9c2414555},
		{0x13d942922a5cf63a, 0x357e33e36e261e7d, 0xcf05a27c8456088d, 0x0000bd1de7ba50f0, 0x83d0c7532f8c1fde, 0x13f70bf38bbf2905},
		{0x5c57fd95bfafbdbb, 0x28a359a65e541707, 0x3983ceb4f6360b6d, 0xafe19ff6f97e6d53, 0xb3468f4550192bf7, 0x0bb6cde49d8ba257},
		{0x590b62c7ff8a513f, 0x314b4ce372cacefd, 0x6bef32ce94b8a800, 0x6ddf84a095713d5f, 0x64eace4cb0982191, 0x0386213c651b888d},
		{0xa5310a31111bbcdd, 0xa14ac0f5da148982, 0xf9ad9cc95423d2e9, 0xaa6ec095283ee4a7, 0xcf5b1f022e1c9107, 0x01fddf5aed881793},
		{0x65a572b0d7a7d950, 0xe25c2d8183473a19, 0xc2fcebe7cb877dbd, 0x05b2d36c769a89b0, 0xba12961be86e9efb, 0x07eb1b29c1dfde1f},
		{0x93e09572f7c4cd24, 0x364e929076795091, 0x8569467e68af51b5, 0xa47da89439f5340f, 0xf4fa918082e44d64, 0x0ad52ba3e6695a79},
		{0x911429844e0d5f54, 0xd03f51a3516bb233, 0x3d587e5640536e66, 0xfa86d2a3a9a73482, 0xa90ed5adf1ed5537, 0x149c9c326a5e7393},
		{0x462bbeb03c12921a, 0xdc9af5fa0a274a17, 0x9a558ebde836ebed, 0x649ef8f11a4fae46, 0x8100e1652b3cdc62, 0x1862bd62c291dacb},
		{0x05c9b8ca89f12c26, 0x0194160fa9b9ac4f, 0x6a643d5a6879fa2c, 0x14665bdd8846e19d, 0xbb1d0d53af3ff6bf, 0x12c7e1c3b28962e5},
		{0xb55ebf900b8a3e17, 0xfedc77ec1a9201c4, 0x1f07db10ea1a4df4, 0x0dfbd15dc41a594d, 0x389547f2334a5391, 0x02419f98165871a4},
		{0xb416af000745fc20, 0x8e563e9d1ea6d0f5, 0x7c763e17763a0652, 0x01458ef0159ebbef, 0x8346fe421f96bb13, 0x0d2d7b829ce324d2},
		{0x93096bb538d64615, 0x6f2a2619951d823a, 0x8f66b3ea59514fa4, 0xf563e63704f7092f, 0x724b136c4cf2d9fa, 0x046959cfcfd0bf49},
		{0xea748d4b6e405346, 0x91e9079c2c02d58f, 0x41064965946d9b59, 0xa06731f1d2bbe1ee, 0x07f897e267a33f1b, 0x1017290919210e5f},
		{0x872aa6c17d985097, 0xeecc53161264562a, 0x07afe37afff55002, 0x54759078e5be6838, 0xc4b92d15db8acca8, 0x106d87d1b51d13b9},
	},
	{
		{0xeb6c359d47e52b1c, 0x18ef5f8a10634d60, 0xddfa71a0889d5b7e, 0x723e71dcc5fc1323, 0x52f45700b70d5c69, 0x0a8b981ee47691f1},
		{0x616a3c4f5535b9fb, 0x6f5f037395dbd911, 0xf25f4cc5e35c65da, 0x3e50dffea3c62658, 0x6a33dca523560776, 0x0fadeff77b6bfe3e},
		{0x2be9b66df470059c, 0x24a2c159a3d36742, 0x115dbe7ad10c2a37, 0xb6634a652ee5884d, 0x04fe8bb2b8d81af4, 0x01c2a7a256fe9c41},
		{0xf27bf8ef3b75a386, 0x898b367476c9073f, 0x24482e6b8c2f4e5f, 0xc8e0bbd6fe110806, 0x59b0c17f7631448a, 0x11037cd58b3dbfbd},
		{0x31c7912ea267eec6, 0x1dbf6f1c5fcdb700, 0xd30d4fe3ba86fdb1, 0x3cae528fbee9a2a4, 0xb1cce69b6aa9ad9a, 0x044393bb632d94fb},
		{0xc66ef6efeeb5c7e8, 0x9824c289dd72bb55, 0x71b1a4d2f119981d, 0x104fc1aafb0919cc, 0x0e49df01d942a628, 0x096c3a09773272d4},
		{0x9abc11eb5fadeff4, 0x32dca50a885728f0, 0xfb1fa3721569734c, 0xc4b76271ea6506b3, 0xd466a75599ce728e, 0x0c81d4645f4cb6ed},
		{0x4199f10e5b8be45b, 0xda64e495b1e87930, 0xcb353efe9b33e4ff, 0x9e9efb24aa6424c6, 0xf08d33680a237465, 0x0d3378023e4c7406},
		{0x7eb4ae92ec74d3a5, 0xc341b4aa9fac3497, 0x5be603899e907687, 0x03bfd9cca75cbdeb, 0x564c2935a96bfa93, 0x0ef3c33371e2fdb5},
		{0x7ee91fd449f6ac2e, 0xe5d5bd5cb9357a30, 0x773a8ca5196b1380, 0xd0fda172174ed023, 0x6cb95e0fa776aead, 0x0d22d5a40cec7cff},
		{0xf727e09285fd8519, 0xdc9d55a83017897b, 0x7549d8bd057894ae, 0x178419613d90d8f8, 0xfce95ebdeb5b490a, 0x0467ffaef23fc49e},
		{0xc1769e6a7c385f1b, 0x79bc930deac01c03, 0x5461c75a23ede3b5, 0x6e20829e5c230c45, 0x828e0f1e772a53cd, 0x116aefa749127bff},
		{0x101c10bf2744c10a, 0xbbf18d053a6a3154, 0xa0ecf39ef026f602, 0xfc009d4996dc5153, 0xb9000209d5bd08d3, 0x189e5fe4470cd73c},
		{0x7ebd546ca1575ed2, 0xe47d5a981d081b55, 0x57b2b625b6d4ca21, 0xb0a1ba04228520cc, 0x98738983c2107ff3, 0x13dddbc4799d81d6},
		{0x09319f2e39834935, 0x039e952cbdb05c21, 0x55ba77a9a2f76493, 0xfd04e3dfc6086467, 0xfb95832e7d78742e, 0x0ef9c24eccaf5e0e},
		{0x760900000002fffd, 0xebf4000bc40c0002, 0x5f48985753c758ba, 0x77ce585370525745, 0x5c071a97a256ec6d, 0x15f65ec3fa80e493},
	},
}

var isogenyConstantsG2 = [4][4]*fe2{
	{
		{
			fe{0x47f671c71ce05e62, 0x06dd57071206393e, 0x7c80cd2af3fd71a2, 0x048103ea9e6cd062, 0xc54516acc8d037f6, 0x13808f550920ea41},
			fe{0x47f671c71ce05e62, 0x06dd57071206393e, 0x7c80cd2af3fd71a2, 0x048103ea9e6cd062, 0xc54516acc8d037f6, 0x13808f550920ea41},
		},
		{
			fe{0, 0, 0, 0, 0, 0},
			fe{0x5fe55555554c71d0, 0x873fffdd236aaaa3, 0x6a6b4619b26ef918, 0x21c2888408874945, 0x2836cda7028cabc5, 0x0ac73310a7fd5abd},
		},
		{
			fe{0x0a0c5555555971c3, 0xdb0c00101f9eaaae, 0xb1fb2f941d797997, 0xd3960742ef416e1c, 0xb70040e2c20556f4, 0x149d7861e581393b},
			fe{0xaff2aaaaaaa638e8, 0x439fffee91b55551, 0xb535a30cd9377c8c, 0x90e144420443a4a2, 0x941b66d3814655e2, 0x0563998853fead5e},
		},
		{
			fe{0x40aac71c71c725ed, 0x190955557a84e38e, 0xd817050a8f41abc3, 0xd86485d4c87f6fb1, 0x696eb479f885d059, 0x198e1a74328002d2},
			fe{0, 0, 0, 0, 0, 0},
		},
	},
	{
		{
			fe{0, 0, 0, 0, 0, 0},
			fe{0x1f3affffff13ab97, 0xf25bfc611da3ff3e, 0xca3757cb3819b208, 0x3e6427366f8cec18, 0x03977bc86095b089, 0x04f69db13f39a952},
		},
		{
			fe{0x447600000027552e, 0xdcb8009a43480020, 0x6f7ee9ce4a6e8b59, 0xb10330b7c0a95bc6, 0x6140b1fcfb1e54b7, 0x0381be097f0bb4e1},
			fe{0x7588ffffffd8557d, 0x41f3ff646e0bffdf, 0xf7b1e8d2ac426aca, 0xb3741acd32dbb6f8, 0xe9daf5b9482d581f, 0x167f53e0ba7431b8},
		},
		{
			fe{0x760900000002fffd, 0xebf4000bc40c0002, 0x5f48985753c758ba, 0x77ce585370525745, 0x5c071a97a256ec6d, 0x15f65ec3fa80e493},
			fe{0, 0, 0, 0, 0, 0},
		},
		{
			fe{0, 0, 0, 0, 0, 0},
			fe{0, 0, 0, 0, 0, 0},
		},
	},
	{
		{
			fe{0x96d8f684bdfc77be, 0xb530e4f43b66d0e2, 0x184a88ff379652fd, 0x57cb23ecfae804e1, 0x0fd2e39eada3eba9, 0x08c8055e31c5d5c3},
			fe{0x96d8f684bdfc77be, 0xb530e4f43b66d0e2, 0x184a88ff379652fd, 0x57cb23ecfae804e1, 0x0fd2e39eada3eba9, 0x08c8055e31c5d5c3},
		},
		{
			fe{0, 0, 0, 0, 0, 0},
			fe{0xbf0a71c71c91b406, 0x4d6d55d28b7638fd, 0x9d82f98e5f205aee, 0xa27aa27b1d1a18d5, 0x02c3b2b2d2938e86, 0x0c7d13420b09807f},
		},
		{
			fe{0xd7f9555555531c74, 0x21cffff748daaaa8, 0x5a9ad1866c9bbe46, 0x4870a2210221d251, 0x4a0db369c0a32af1, 0x02b1ccc429ff56af},
			fe{0xe205aaaaaaac8e37, 0xfcdc000768795556, 0x0c96011a8a1537dd, 0x1c06a963f163406e, 0x010df44c82a881e6, 0x174f45260f808feb},
		},
		{
			fe{0xa470bda12f67f35c, 0xc0fe38e23327b425, 0xc9d3d0f2c6f0678d, 0x1c55c9935b5a982e, 0x27f6c0e2f0746764, 0x117c5e6e28aa9054},
			fe{0, 0, 0, 0, 0, 0},
		},
	},
	{
		{
			fe{0x0162fffffa765adf, 0x8f7bea480083fb75, 0x561b3c2259e93611, 0x11e19fc1a9c875d5, 0xca713efc00367660, 0x03c6a03d41da1151},
			fe{0x0162fffffa765adf, 0x8f7bea480083fb75, 0x561b3c2259e93611, 0x11e19fc1a9c875d5, 0xca713efc00367660, 0x03c6a03d41da1151},
		},
		{
			fe{0, 0, 0, 0, 0, 0},
			fe{0x5db0fffffd3b02c5, 0xd713f52358ebfdba, 0x5ea60761a84d161a, 0xbb2c75a34ea6c44a, 0x0ac6735921c1119b, 0x0ee3d913bdacfbf6},
		},
		{
			fe{0x66b10000003affc5, 0xcb1400e764ec0030, 0xa73e5eb56fa5d106, 0x8984c913a0fe09a9, 0x11e10afb78ad7f13, 0x05429d0e3e918f52},
			fe{0x534dffffffc4aae6, 0x5397ff174c67ffcf, 0xbff273eb870b251d, 0xdaf2827152870915, 0x393a9cbaca9e2dc3, 0x14be74dbfaee5748},
		},
		{
			fe{0x760900000002fffd, 0xebf4000bc40c0002, 0x5f48985753c758ba, 0x77ce585370525745, 0x5c071a97a256ec6d, 0x15f65ec3fa80e493},
			fe{0, 0, 0, 0, 0, 0},
		},
	},
}
