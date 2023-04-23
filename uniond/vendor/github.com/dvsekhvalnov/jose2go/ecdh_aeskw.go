package jose

func init() {
	RegisterJwa(&EcdhAesKW{ keySizeBits: 128, aesKW: &AesKW{ keySizeBits: 128}, ecdh: &Ecdh{directAgreement:false}})
	RegisterJwa(&EcdhAesKW{ keySizeBits: 192, aesKW: &AesKW{ keySizeBits: 192}, ecdh: &Ecdh{directAgreement:false}})
	RegisterJwa(&EcdhAesKW{ keySizeBits: 256, aesKW: &AesKW{ keySizeBits: 256}, ecdh: &Ecdh{directAgreement:false}})
}

// Elliptic curve Diffie–Hellman with AES Key Wrap key management algorithm implementation
type EcdhAesKW struct{
	keySizeBits int
	aesKW JwaAlgorithm
	ecdh JwaAlgorithm
}

func (alg *EcdhAesKW) Name() string {
	switch alg.keySizeBits {
		case 128: return ECDH_ES_A128KW
		case 192: return ECDH_ES_A192KW
		default: return  ECDH_ES_A256KW
	}
}

func (alg *EcdhAesKW) WrapNewKey(cekSizeBits int, key interface{}, header map[string]interface{}) (cek []byte, encryptedCek []byte, err error) {
	var kek []byte

	if kek,_,err=alg.ecdh.WrapNewKey(alg.keySizeBits, key, header);err!=nil {
		return nil,nil,err
	}
	
	return alg.aesKW.WrapNewKey(cekSizeBits,kek,header)	
}

func (alg *EcdhAesKW) Unwrap(encryptedCek []byte, key interface{}, cekSizeBits int, header map[string]interface{}) (cek []byte, err error) {
	var kek []byte
	
	if kek,err=alg.ecdh.Unwrap(nil, key, alg.keySizeBits, header);err!=nil {
		return nil,err
	}
	
	return alg.aesKW.Unwrap(encryptedCek,kek,cekSizeBits,header)
}