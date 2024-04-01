# Golang (GO) Javascript Object Signing and Encryption (JOSE) and JSON Web Token (JWT) implementation

[![GoDoc](https://godoc.org/github.com/dvsekhvalnov/jose2go?status.svg)](http://godoc.org/github.com/dvsekhvalnov/jose2go)

Pure Golang (GO) library for generating, decoding and encrypting [JSON Web Tokens](https://tools.ietf.org/html/rfc7519). Zero dependency, relies only
on standard library.

Supports full suite of signing, encryption and compression algorithms defined by [JSON Web Algorithms](https://tools.ietf.org/html/draft-ietf-jose-json-web-algorithms-31) as of July 4, 2014 version.

Extensively unit tested and cross tested (100+ tests) for compatibility with [jose.4.j](https://bitbucket.org/b_c/jose4j/wiki/Home), [Nimbus-JOSE-JWT](https://bitbucket.org/nimbusds/nimbus-jose-jwt/wiki/Home), [json-jwt](https://github.com/nov/json-jwt) and
[jose-jwt](https://github.com/dvsekhvalnov/jose-jwt) libraries.


## Status
Used in production. GA ready. Current version is 1.6.

## Important
v1.6 security tuning options

v1.5 bug fix release

v1.4 changes default behavior of inserting `typ=JWT` header if not overriden. As of 1.4 no
extra headers added by library automatically. To mimic pre 1.4 behaviour use:
```Go
token, err := jose.Sign(..., jose.Header("typ", "JWT"))

//or

token, err := jose.Encrypt(..., jose.Header("typ", "JWT"))
```

v1.3 fixed potential Invalid Curve Attack on NIST curves within ECDH key management.
Upgrade strongly recommended.

v1.2 breaks `jose.Decode` interface by returning 3 values instead of 2.

v1.2 deprecates `jose.Compress` method in favor of using configuration options to `jose.Encrypt`,
the method will be removed in next release.

### Migration to v1.2
Pre v1.2 decoding:

```Go
payload,err := jose.Decode(token,sharedKey)
```

Should be updated to v1.2:

```Go
payload, headers, err := jose.Decode(token,sharedKey)
```

Pre v1.2 compression:

```Go
token,err := jose.Compress(payload,jose.DIR,jose.A128GCM,jose.DEF, key)
```

Should be update to v1.2:

```Go
token, err := jose.Encrypt(payload, jose.DIR, jose.A128GCM, key, jose.Zip(jose.DEF))
```

## Supported JWA algorithms

**Signing**
- HMAC signatures with HS256, HS384 and HS512.
- RSASSA-PKCS1-V1_5 signatures with RS256, RS384 and RS512.
- RSASSA-PSS signatures (probabilistic signature scheme with appendix) with PS256, PS384 and PS512.
- ECDSA signatures with ES256, ES384 and ES512.
- NONE (unprotected) plain text algorithm without integrity protection

**Encryption**
- RSAES OAEP (using SHA-1 and MGF1 with SHA-1) encryption with A128CBC-HS256, A192CBC-HS384, A256CBC-HS512, A128GCM, A192GCM, A256GCM
- RSAES OAEP 256 (using SHA-256 and MGF1 with SHA-256) encryption with A128CBC-HS256, A192CBC-HS384, A256CBC-HS512, A128GCM, A192GCM, A256GCM
- RSAES-PKCS1-V1_5 encryption with A128CBC-HS256, A192CBC-HS384, A256CBC-HS512, A128GCM, A192GCM, A256GCM
- A128KW, A192KW, A256KW encryption with A128CBC-HS256, A192CBC-HS384, A256CBC-HS512, A128GCM, A192GCM, A256GCM
- A128GCMKW, A192GCMKW, A256GCMKW encryption with A128CBC-HS256, A192CBC-HS384, A256CBC-HS512, A128GCM, A192GCM, A256GCM
- ECDH-ES with A128CBC-HS256, A192CBC-HS384, A256CBC-HS512, A128GCM, A192GCM, A256GCM
- ECDH-ES+A128KW, ECDH-ES+A192KW, ECDH-ES+A256KW with A128CBC-HS256, A192CBC-HS384, A256CBC-HS512, A128GCM, A192GCM, A256GCM
- PBES2-HS256+A128KW, PBES2-HS384+A192KW, PBES2-HS512+A256KW with A128CBC-HS256, A192CBC-HS384, A256CBC-HS512, A128GCM, A192GCM, A256GCM
- Direct symmetric key encryption with pre-shared key A128CBC-HS256, A192CBC-HS384, A256CBC-HS512, A128GCM, A192GCM and A256GCM

**Compression**
- DEFLATE compression

## Installation
### Grab package from github
`go get github.com/dvsekhvalnov/jose2go` or `go get -u github.com/dvsekhvalnov/jose2go` to update to latest version

### Import package
```Go
import (
	"github.com/dvsekhvalnov/jose2go"
)
```

## Usage
#### Creating Plaintext (unprotected) Tokens

```Go
package main

import (
	"fmt"
	"github.com/dvsekhvalnov/jose2go"
)

func main() {

	payload :=  `{"hello": "world"}`

	token,err := jose.Sign(payload,jose.NONE, nil)

	if(err==nil) {
		//go use token
		fmt.Printf("\nPlaintext = %v\n",token)
	}
}
```

### Creating signed tokens
#### HS-256, HS-384 and HS-512
Signing with HS256, HS384, HS512 expecting `[]byte` array key of corresponding length:

```Go
package main

import (
	"fmt"
	"github.com/dvsekhvalnov/jose2go"
)

func main() {

	payload :=  `{"hello": "world"}`

	key := []byte{97,48,97,50,97,98,100,56,45,54,49,54,50,45,52,49,99,51,45,56,51,100,54,45,49,99,102,53,53,57,98,52,54,97,102,99}

	token,err := jose.Sign(payload,jose.HS256,key)

	if(err==nil) {
		//go use token
		fmt.Printf("\nHS256 = %v\n",token)
	}
}
```

#### RS-256, RS-384 and RS-512, PS-256, PS-384 and PS-512
Signing with RS256, RS384, RS512, PS256, PS384, PS512 expecting `*rsa.PrivateKey` private key of corresponding length. **jose2go** [provides convenient utils](#dealing-with-keys) to construct `*rsa.PrivateKey` instance from PEM encoded PKCS1 or PKCS8 data: `Rsa.ReadPrivate([]byte)` under `jose2go/keys/rsa` package.

```Go
package main

import (
	"fmt"
	"io/ioutil"
	Rsa "github.com/dvsekhvalnov/jose2go/keys/rsa"
	"github.com/dvsekhvalnov/jose2go"
)

func main() {

	payload :=  `{"hello": "world"}`

	keyBytes,err := ioutil.ReadFile("private.key")

	if(err!=nil) {
		panic("invalid key file")
	}

	privateKey,e:=Rsa.ReadPrivate(keyBytes)

	if(e!=nil) {
		panic("invalid key format")
	}

	token,err := jose.Sign(payload,jose.RS256, privateKey)

	if(err==nil) {
		//go use token
		fmt.Printf("\nRS256 = %v\n",token)
	}
}
```

#### ES-256, ES-384 and ES-512
ES256, ES384, ES512 ECDSA signatures expecting `*ecdsa.PrivateKey` private elliptic curve key of corresponding length.  **jose2go** [provides convenient utils](#dealing-with-keys) to construct `*ecdsa.PrivateKey` instance from PEM encoded PKCS1 or PKCS8 data: `ecc.ReadPrivate([]byte)` or directly from `X,Y,D` parameters: `ecc.NewPrivate(x,y,d []byte)` under `jose2go/keys/ecc` package.

```Go
package main

import (
    "fmt"
    "github.com/dvsekhvalnov/jose2go/keys/ecc"
    "github.com/dvsekhvalnov/jose2go"
)

func main() {

    payload := `{"hello":"world"}`

	privateKey:=ecc.NewPrivate([]byte{4, 114, 29, 223, 58, 3, 191, 170, 67, 128, 229, 33, 242, 178, 157, 150, 133, 25, 209, 139, 166, 69, 55, 26, 84, 48, 169, 165, 67, 232, 98, 9},
	 			 			   []byte{131, 116, 8, 14, 22, 150, 18, 75, 24, 181, 159, 78, 90, 51, 71, 159, 214, 186, 250, 47, 207, 246, 142, 127, 54, 183, 72, 72, 253, 21, 88, 53},
							   []byte{ 42, 148, 231, 48, 225, 196, 166, 201, 23, 190, 229, 199, 20, 39, 226, 70, 209, 148, 29, 70, 125, 14, 174, 66, 9, 198, 80, 251, 95, 107, 98, 206 })

    token,err := jose.Sign(payload, jose.ES256, privateKey)

    if(err==nil) {
        //go use token
        fmt.Printf("\ntoken = %v\n",token)
    }
}
```

### Creating encrypted tokens
#### RSA-OAEP-256, RSA-OAEP and RSA1\_5 key management algorithm
RSA-OAEP-256, RSA-OAEP and RSA1_5 key management expecting `*rsa.PublicKey` public key of corresponding length.

```Go
package main

import (
    "fmt"
    "io/ioutil"
    Rsa "github.com/dvsekhvalnov/jose2go/keys/rsa"
    "github.com/dvsekhvalnov/jose2go"
)

func main() {

	payload :=  `{"hello": "world"}`

	keyBytes,err := ioutil.ReadFile("public.key")

	if(err!=nil) {
		panic("invalid key file")
	}

	publicKey,e:=Rsa.ReadPublic(keyBytes)

	if(e!=nil) {
		panic("invalid key format")
	}

	//OR:
	//token,err := jose.Encrypt(payload, jose.RSA1_5, jose.A256GCM, publicKey)
	token,err := jose.Encrypt(payload, jose.RSA_OAEP, jose.A256GCM, publicKey)

    if(err==nil) {
        //go use token
        fmt.Printf("\ntoken = %v\n",token)
    }
}
```

#### AES Key Wrap key management family of algorithms
AES128KW, AES192KW and AES256KW key management requires `[]byte` array key of corresponding length

```Go
package main

import (
	"fmt"
	"github.com/dvsekhvalnov/jose2go"
)

func main() {

	payload :=  `{"hello": "world"}`

	sharedKey :=[]byte{194,164,235,6,138,248,171,239,24,216,11,22,137,199,215,133}

	token,err := jose.Encrypt(payload,jose.A128KW,jose.A128GCM,sharedKey)

	if(err==nil) {
		//go use token
		fmt.Printf("\nA128KW A128GCM = %v\n",token)
	}
}
```

#### AES GCM Key Wrap key management family of algorithms
AES128GCMKW, AES192GCMKW and AES256GCMKW key management requires `[]byte` array key of corresponding length

```Go
package main

import (
	"fmt"
	"github.com/dvsekhvalnov/jose2go"
)

func main() {

	payload :=  `{"hello": "world"}`

	sharedKey :=[]byte{194,164,235,6,138,248,171,239,24,216,11,22,137,199,215,133}

	token,err := jose.Encrypt(payload,jose.A128GCMKW,jose.A128GCM,sharedKey)

	if(err==nil) {
		//go use token
		fmt.Printf("\nA128GCMKW A128GCM = %v\n",token)
	}
}
```

#### ECDH-ES and ECDH-ES with AES Key Wrap key management family of algorithms
ECDH-ES and ECDH-ES+A128KW, ECDH-ES+A192KW, ECDH-ES+A256KW key management requires `*ecdsa.PublicKey` elliptic curve key of corresponding length. **jose2go** [provides convenient utils](#dealing-with-keys) to construct `*ecdsa.PublicKey` instance from PEM encoded PKCS1 X509 certificate or PKIX data: `ecc.ReadPublic([]byte)` or directly from `X,Y` parameters: `ecc.NewPublic(x,y []byte)`under `jose2go/keys/ecc` package:

```Go
package main

import (
    "fmt"
    "github.com/dvsekhvalnov/jose2go/keys/ecc"
    "github.com/dvsekhvalnov/jose2go"
)

func main() {

    payload := `{"hello":"world"}`

    publicKey:=ecc.NewPublic([]byte{4, 114, 29, 223, 58, 3, 191, 170, 67, 128, 229, 33, 242, 178, 157, 150, 133, 25, 209, 139, 166, 69, 55, 26, 84, 48, 169, 165, 67, 232, 98, 9},
                             []byte{131, 116, 8, 14, 22, 150, 18, 75, 24, 181, 159, 78, 90, 51, 71, 159, 214, 186, 250, 47, 207, 246, 142, 127, 54, 183, 72, 72, 253, 21, 88, 53})

    token,err := jose.Encrypt(payload, jose.ECDH_ES, jose.A128CBC_HS256, publicKey)

    if(err==nil) {
        //go use token
        fmt.Printf("\ntoken = %v\n",token)
    }
}
```

#### PBES2 using HMAC SHA with AES Key Wrap key management family of algorithms
PBES2-HS256+A128KW, PBES2-HS384+A192KW, PBES2-HS512+A256KW key management requires `string` passphrase from which actual key will be derived

```Go
package main

import (
	"fmt"
	"github.com/dvsekhvalnov/jose2go"
)

func main() {

	payload :=  `{"hello": "world"}`

	passphrase := `top secret`

	token,err := jose.Encrypt(payload,jose.PBES2_HS256_A128KW,jose.A256GCM,passphrase)

	if(err==nil) {
		//go use token
		fmt.Printf("\nPBES2_HS256_A128KW A256GCM = %v\n",token)
	}
}
```

#### DIR direct pre-shared symmetric key management
Direct key management with pre-shared symmetric keys expecting `[]byte` array key of corresponding length:

```Go
package main

import (
	"fmt"
	"github.com/dvsekhvalnov/jose2go"
)

func main() {

	payload :=  `{"hello": "world"}`

	sharedKey :=[]byte{194,164,235,6,138,248,171,239,24,216,11,22,137,199,215,133}

	token,err := jose.Encrypt(payload,jose.DIR,jose.A128GCM,sharedKey)

	if(err==nil) {
		//go use token
		fmt.Printf("\nDIR A128GCM = %v\n",token)
	}
}
```

### Creating compressed & encrypted tokens
#### DEFLATE compression
**jose2go** supports optional DEFLATE compression of payload before encrypting, can be used with all supported encryption and key management algorithms:

```Go
package main

import (
	"fmt"
	"github.com/dvsekhvalnov/jose2go"
)

func main() {

	payload := `{"hello": "world"}`

	sharedKey := []byte{194, 164, 235, 6, 138, 248, 171, 239, 24, 216, 11, 22, 137, 199, 215, 133}

	token, err := jose.Encrypt(payload, jose.DIR, jose.A128GCM, sharedKey, jose.Zip(jose.DEF))

	if err == nil {
		//go use token
		fmt.Printf("\nDIR A128GCM DEFLATED= %v\n", token)
	}
}
```

### Verifying, Decoding and Decompressing tokens
Decoding json web tokens is fully symmetric to creating signed or encrypted tokens (with respect to public/private cryptography), decompressing deflated payloads is handled automatically:

As of v1.2 decode method defined as `jose.Decode() payload string, headers map[string]interface{}, err error` and returns both payload as unprocessed string and headers as map.

**HS256, HS384, HS512** signatures, **A128KW, A192KW, A256KW**,**A128GCMKW, A192GCMKW, A256GCMKW** and **DIR** key management algorithm expecting `[]byte` array key:

```Go
package main

import (
	"fmt"
	"github.com/dvsekhvalnov/jose2go"
)

func main() {

	token := "eyJhbGciOiJIUzI1NiIsImN0eSI6InRleHRcL3BsYWluIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.chIoYWrQMA8XL5nFz6oLDJyvgHk2KA4BrFGrKymjC8E"

	sharedKey :=[]byte{97,48,97,50,97,98,100,56,45,54,49,54,50,45,52,49,99,51,45,56,51,100,54,45,49,99,102,53,53,57,98,52,54,97,102,99}

	payload, headers, err := jose.Decode(token,sharedKey)

	if(err==nil) {
		//go use token
		fmt.Printf("\npayload = %v\n",payload)

        //and/or use headers
        fmt.Printf("\nheaders = %v\n",headers)
	}
}
```

**RS256, RS384, RS512**,**PS256, PS384, PS512** signatures expecting `*rsa.PublicKey` public key of corresponding length. **jose2go** [provides convenient utils](#dealing-with-keys) to construct `*rsa.PublicKey` instance from PEM encoded PKCS1 X509 certificate or PKIX data: `Rsa.ReadPublic([]byte)` under `jose2go/keys/rsa` package:

```Go
package main

import (
    "fmt"
    "io/ioutil"
    Rsa "github.com/dvsekhvalnov/jose2go/keys/rsa"
    "github.com/dvsekhvalnov/jose2go"
)

func main() {

    token := "eyJhbGciOiJSUzI1NiIsImN0eSI6InRleHRcL3BsYWluIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.NL_dfVpZkhNn4bZpCyMq5TmnXbT4yiyecuB6Kax_lV8Yq2dG8wLfea-T4UKnrjLOwxlbwLwuKzffWcnWv3LVAWfeBxhGTa0c4_0TX_wzLnsgLuU6s9M2GBkAIuSMHY6UTFumJlEeRBeiqZNrlqvmAzQ9ppJHfWWkW4stcgLCLMAZbTqvRSppC1SMxnvPXnZSWn_Fk_q3oGKWw6Nf0-j-aOhK0S0Lcr0PV69ZE4xBYM9PUS1MpMe2zF5J3Tqlc1VBcJ94fjDj1F7y8twmMT3H1PI9RozO-21R0SiXZ_a93fxhE_l_dj5drgOek7jUN9uBDjkXUwJPAyp9YPehrjyLdw"

    keyBytes, err := ioutil.ReadFile("public.key")

    if(err!=nil) {
        panic("invalid key file")
    }

    publicKey, e:=Rsa.ReadPublic(keyBytes)

    if(e!=nil) {
        panic("invalid key format")
    }

    payload, headers, err := jose.Decode(token, publicKey)

    if(err==nil) {
        //go use token
        fmt.Printf("\npayload = %v\n",payload)

        //and/or use headers
        fmt.Printf("\nheaders = %v\n",headers)
    }
}
```

**RSA-OAEP-256**, **RSA-OAEP** and **RSA1_5** key management algorithms expecting `*rsa.PrivateKey` private key of corresponding length:

```Go
package main

import (
    "fmt"
    "io/ioutil"
    Rsa "github.com/dvsekhvalnov/jose2go/keys/rsa"
    "github.com/dvsekhvalnov/jose2go"
)

func main() {

    token := "eyJhbGciOiJSU0ExXzUiLCJlbmMiOiJBMjU2R0NNIn0.ixD3WVOkvaxeLKi0kyVqTzM6W2EW25SHHYCAr9473Xq528xSK0AVux6kUtv7QMkQKgkMvO8X4VdvonyGkDZTK2jgYUiI06dz7I1sjWJIbyNVrANbBsmBiwikwB-9DLEaKuM85Lwu6gnzbOF6B9R0428ckxmITCPDrzMaXwYZHh46FiSg9djChUTex0pHGhNDiEIgaINpsmqsOFX1L2Y7KM2ZR7wtpR3kidMV3JlxHdKheiPKnDx_eNcdoE-eogPbRGFdkhEE8Dyass1ZSxt4fP27NwsIer5pc0b922_3XWdi1r1TL_fLvGktHLvt6HK6IruXFHpU4x5Z2gTXWxEIog.zzTNmovBowdX2_hi.QSPSgXn0w25ugvzmu2TnhePn.0I3B9BE064HFNP2E0I7M9g"

    keyBytes, err := ioutil.ReadFile("private.key")

    if(err!=nil) {
        panic("invalid key file")
    }

    privateKey, e:=Rsa.ReadPrivate(keyBytes)

    if(e!=nil) {
        panic("invalid key format")
    }

    payload, headers, err := jose.Decode(token, privateKey)

    if(err==nil) {
        //go use payload
        fmt.Printf("\npayload = %v\n",payload)

        //and/or use headers
        fmt.Printf("\nheaders = %v\n",headers)
    }
}
```

**PBES2-HS256+A128KW, PBES2-HS384+A192KW, PBES2-HS512+A256KW** key management algorithms expects `string` passpharase as a key

```Go
package main

import (
	"fmt"
	"github.com/dvsekhvalnov/jose2go"
)

func main() {

	token :=  `eyJhbGciOiJQQkVTMi1IUzI1NitBMTI4S1ciLCJlbmMiOiJBMjU2R0NNIiwicDJjIjo4MTkyLCJwMnMiOiJlZWpFZTF0YmJVbU5XV2s2In0.J2HTgltxH3p7A2zDgQWpZPgA2CHTSnDmMhlZWeSOMoZ0YvhphCeg-w.FzYG5AOptknu7jsG.L8jAxfxZhDNIqb0T96YWoznQ.yNeOfQWUbm8KuDGZ_5lL_g`

	passphrase := `top secret`

	payload, headers, err := jose.Decode(token,passphrase)

	if(err==nil) {
		//go use token
		fmt.Printf("\npayload = %v\n",payload)

        //and/or use headers
        fmt.Printf("\nheaders = %v\n",headers)
	}
}
```

**ES256, ES284, ES512** signatures expecting `*ecdsa.PublicKey` public elliptic curve key of corresponding length. **jose2go** [provides convenient utils](#dealing-with-keys) to construct `*ecdsa.PublicKey` instance from PEM encoded PKCS1 X509 certificate or PKIX data: `ecc.ReadPublic([]byte)` or directly from `X,Y` parameters: `ecc.NewPublic(x,y []byte)`under `jose2go/keys/ecc` package:

```Go
package main

import (
    "fmt"
    "github.com/dvsekhvalnov/jose2go/keys/ecc"
    "github.com/dvsekhvalnov/jose2go"
)

func main() {

    token := "eyJhbGciOiJFUzI1NiIsImN0eSI6InRleHRcL3BsYWluIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.EVnmDMlz-oi05AQzts-R3aqWvaBlwVZddWkmaaHyMx5Phb2NSLgyI0kccpgjjAyo1S5KCB3LIMPfmxCX_obMKA"

	publicKey:=ecc.NewPublic([]byte{4, 114, 29, 223, 58, 3, 191, 170, 67, 128, 229, 33, 242, 178, 157, 150, 133, 25, 209, 139, 166, 69, 55, 26, 84, 48, 169, 165, 67, 232, 98, 9},
	 			 			 []byte{131, 116, 8, 14, 22, 150, 18, 75, 24, 181, 159, 78, 90, 51, 71, 159, 214, 186, 250, 47, 207, 246, 142, 127, 54, 183, 72, 72, 253, 21, 88, 53})

    payload, headers, err := jose.Decode(token, publicKey)

    if(err==nil) {
        //go use token
        fmt.Printf("\npayload = %v\n",payload)

        //and/or use headers
        fmt.Printf("\nheaders = %v\n",headers)
    }
}
```

**ECDH-ES** and **ECDH-ES+A128KW**, **ECDH-ES+A192KW**, **ECDH-ES+A256KW** key management expecting `*ecdsa.PrivateKey` private elliptic curve key of corresponding length.  **jose2go** [provides convenient utils](#dealing-with-keys) to construct `*ecdsa.PrivateKey` instance from PEM encoded PKCS1 or PKCS8 data: `ecc.ReadPrivate([]byte)` or directly from `X,Y,D` parameters: `ecc.NewPrivate(x,y,d []byte)` under `jose2go/keys/ecc` package:

```Go
package main

import (
    "fmt"
    "github.com/dvsekhvalnov/jose2go/keys/ecc"
    "github.com/dvsekhvalnov/jose2go"
)

func main() {

    token := "eyJhbGciOiJFQ0RILUVTIiwiZW5jIjoiQTEyOENCQy1IUzI1NiIsImVwayI6eyJrdHkiOiJFQyIsIngiOiItVk1LTG5NeW9IVHRGUlpGNnFXNndkRm5BN21KQkdiNzk4V3FVMFV3QVhZIiwieSI6ImhQQWNReTgzVS01Qjl1U21xbnNXcFZzbHVoZGJSZE1nbnZ0cGdmNVhXTjgiLCJjcnYiOiJQLTI1NiJ9fQ..UA3N2j-TbYKKD361AxlXUA.XxFur_nY1GauVp5W_KO2DEHfof5s7kUwvOgghiNNNmnB4Vxj5j8VRS8vMOb51nYy2wqmBb2gBf1IHDcKZdACkCOMqMIcpBvhyqbuKiZPLHiilwSgVV6ubIV88X0vK0C8ZPe5lEyRudbgFjdlTnf8TmsvuAsdtPn9dXwDjUR23bD2ocp8UGAV0lKqKzpAw528vTfD0gwMG8gt_op8yZAxqqLLljMuZdTnjofAfsW2Rq3Z6GyLUlxR51DAUlQKi6UpsKMJoXTrm1Jw8sXBHpsRqA.UHCYOtnqk4SfhAknCnymaQ"

	privateKey:=ecc.NewPrivate([]byte{4, 114, 29, 223, 58, 3, 191, 170, 67, 128, 229, 33, 242, 178, 157, 150, 133, 25, 209, 139, 166, 69, 55, 26, 84, 48, 169, 165, 67, 232, 98, 9},
	 			 			   []byte{131, 116, 8, 14, 22, 150, 18, 75, 24, 181, 159, 78, 90, 51, 71, 159, 214, 186, 250, 47, 207, 246, 142, 127, 54, 183, 72, 72, 253, 21, 88, 53},
							   []byte{ 42, 148, 231, 48, 225, 196, 166, 201, 23, 190, 229, 199, 20, 39, 226, 70, 209, 148, 29, 70, 125, 14, 174, 66, 9, 198, 80, 251, 95, 107, 98, 206 })

    payload, headers, err := jose.Decode(token, privateKey)

    if(err==nil) {
        //go use token
        fmt.Printf("\npayload = %v\n",payload)

        //and/or use headers
        fmt.Printf("\nheaders = %v\n",headers)
    }
}
```

### Adding extra headers
It's possible to pass additional headers while encoding token. **jose2go** provides convenience configuration helpers: `Header(name string, value interface{})` and `Headers(headers map[string]interface{})` that can be passed to `Sign(..)` and `Encrypt(..)` calls.

Note: **jose2go** do not allow to override `alg`, `enc` and `zip` headers.

Example of signing with extra headers:
```Go
	token, err := jose.Sign(payload, jose.ES256, key,
                    		jose.Header("keyid", "111-222-333"),
                    		jose.Header("trans-id", "aaa-bbb"))
```

Encryption with extra headers:
```Go
token, err := jose.Encrypt(payload, jose.DIR, jose.A128GCM, sharedKey,
                    jose.Headers(map[string]interface{}{"keyid": "111-22-33", "cty": "text/plain"}))
```

### Two phase validation
In some cases validation (decoding) key can be unknown prior to examining token content. For instance one can use different keys per token issuer or rely on headers information to determine which key to use, do logging or other things.

**jose2go** allows to pass `func(headers map[string]interface{}, payload string) key interface{}` callback instead of key to `jose.Decode(..)`. Callback will be executed prior to decoding and integrity validation and will recieve parsed headers and payload as is (for encrypted tokens it will be cipher text). Callback should return key to be used for actual decoding process or `error` if decoding should be stopped, given error object will be returned from `jose.Decode(..)` call.

Example of decoding token with callback:

```Go
package main

import (
	"crypto/rsa"
	"fmt"
	"github.com/dvsekhvalnov/jose2go"
	"github.com/dvsekhvalnov/jose2go/keys/rsa"
	"io/ioutil"
	"errors"
)

func main() {

	token := "eyJhbGciOiJSUzI1NiIsImN0eSI6InRleHRcL3BsYWluIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.NL_dfVpZkhNn4bZpCyMq5TmnXbT4yiyecuB6Kax_lV8Yq2dG8wLfea-T4UKnrjLOwxlbwLwuKzffWcnWv3LVAWfeBxhGTa0c4_0TX_wzLnsgLuU6s9M2GBkAIuSMHY6UTFumJlEeRBeiqZNrlqvmAzQ9ppJHfWWkW4stcgLCLMAZbTqvRSppC1SMxnvPXnZSWn_Fk_q3oGKWw6Nf0-j-aOhK0S0Lcr0PV69ZE4xBYM9PUS1MpMe2zF5J3Tqlc1VBcJ94fjDj1F7y8twmMT3H1PI9RozO-21R0SiXZ_a93fxhE_l_dj5drgOek7jUN9uBDjkXUwJPAyp9YPehrjyLdw"

	payload, _, err := jose.Decode(token,
		func(headers map[string]interface{}, payload string) interface{} {
            //log something
			fmt.Printf("\nHeaders before decoding: %v\n", headers)
			fmt.Printf("\nPayload before decoding: %v\n", payload)

            //lookup key based on keyid header as en example
            //or lookup based on something from payload, e.g. 'iss' claim for instance
            key := FindKey(headers['keyid'])

            if(key==nil) {
                return errors.New("Key not found")
            }

            return key;
		})

	if err == nil {
		//go use token
		fmt.Printf("\ndecoded payload = %v\n", payload)
	}
}
```

Two phase validation can be used for implementing additional things like strict `alg` or `enc` validation, see [Customizing library for security](#customizing-library-for-security) for more information.

### Working with binary payload
In addition to work with string payloads (typical use-case) `jose2go` supports
encoding and decoding of raw binary data. `jose.DecodeBytes`, `jose.SignBytes`
and `jose.EncryptBytes` functions provides similar interface but accepting
`[]byte` payloads.

Examples:

```Go
package main

import (
	"github.com/dvsekhvalnov/jose2go"
)

func main() {

	token :=  `eyJhbGciOiJQQkVTMi1IUzI1NitBMTI4S1ciLCJlbmMiOiJBMjU2R0NNIiwicDJjIjo4MTkyLCJwMnMiOiJlZWpFZTF0YmJVbU5XV2s2In0.J2HTgltxH3p7A2zDgQWpZPgA2CHTSnDmMhlZWeSOMoZ0YvhphCeg-w.FzYG5AOptknu7jsG.L8jAxfxZhDNIqb0T96YWoznQ.yNeOfQWUbm8KuDGZ_5lL_g`

	passphrase := `top secret`

	payload, headers, err := jose.DecodeBytes(token,passphrase)

	if(err==nil) {
		//go use token
		//payload = []byte{....}
	}
}
```

```Go
package main

import (
    "fmt"
    "io/ioutil"
    Rsa "github.com/dvsekhvalnov/jose2go/keys/rsa"
    "github.com/dvsekhvalnov/jose2go"
)

func main() {

    payload :=  []byte {0x01, 0x02, 0x03, 0x04}

    keyBytes,err := ioutil.ReadFile("private.key")

    if(err!=nil) {
        panic("invalid key file")
    }

    privateKey,e:=Rsa.ReadPrivate(keyBytes)

    if(e!=nil) {
        panic("invalid key format")
    }

    token,err := jose.SignBytes(payload,jose.RS256, privateKey)

    if(err==nil) {
        //go use token
        fmt.Printf("\nRS256 = %v\n",token)
    }
}
```

```Go
package main

import (
    "fmt"
    "io/ioutil"
    Rsa "github.com/dvsekhvalnov/jose2go/keys/rsa"
    "github.com/dvsekhvalnov/jose2go"
)

func main() {

    payload :=  []byte {0x01, 0x02, 0x03, 0x04}

    keyBytes,err := ioutil.ReadFile("public.key")

    if(err!=nil) {
        panic("invalid key file")
    }

    publicKey,e:=Rsa.ReadPublic(keyBytes)

    if(e!=nil) {
        panic("invalid key format")
    }

    token,err := jose.EncryptBytes(payload, jose.RSA_OAEP, jose.A256GCM, publicKey)

    if(err==nil) {
        //go use token
        fmt.Printf("\ntoken = %v\n",token)
    }
}
```
### Dealing with keys
**jose2go** provides several helper methods to simplify loading & importing of elliptic and rsa keys. Import `jose2go/keys/rsa` or `jose2go/keys/ecc` respectively:

#### RSA keys
1. `Rsa.ReadPrivate(raw []byte) (key *rsa.PrivateKey,err error)` attempts to parse RSA private key from PKCS1 or PKCS8 format (`BEGIN RSA PRIVATE KEY` and `BEGIN PRIVATE KEY` headers)

```Go
package main

import (
	"fmt"
	Rsa "github.com/dvsekhvalnov/jose2go/keys/rsa"
	"io/ioutil"
)

func main() {

    keyBytes, _ := ioutil.ReadFile("private.key")

    privateKey, err:=Rsa.ReadPrivate(keyBytes)

    if(err!=nil) {
        panic("invalid key format")
    }

	fmt.Printf("privateKey = %v\n",privateKey)
}
```

2. `Rsa.ReadPublic(raw []byte) (key *rsa.PublicKey,err error)` attempts to parse RSA public key from PKIX key format or PKCS1 X509 certificate (`BEGIN PUBLIC KEY` and `BEGIN CERTIFICATE` headers)

```Go
package main

import (
	"fmt"
	Rsa "github.com/dvsekhvalnov/jose2go/keys/rsa"
	"io/ioutil"
)

func main() {

    keyBytes, _ := ioutil.ReadFile("public.cer")

    publicKey, err:=Rsa.ReadPublic(keyBytes)

    if(err!=nil) {
        panic("invalid key format")
    }

	fmt.Printf("publicKey = %v\n",publicKey)
}
```

#### ECC keys
1. `ecc.ReadPrivate(raw []byte) (key *ecdsa.PrivateKey,err error)` attemps to parse elliptic curve private key from PKCS1 or PKCS8 format (`BEGIN EC PRIVATE KEY` and `BEGIN PRIVATE KEY` headers)

```Go
package main

import (
	"fmt"
    "github.com/dvsekhvalnov/jose2go/keys/ecc"
	"io/ioutil"
)

func main() {

    keyBytes, _ := ioutil.ReadFile("ec-private.pem")

    ecPrivKey, err:=ecc.ReadPrivate(keyBytes)

    if(err!=nil) {
        panic("invalid key format")
    }

	fmt.Printf("ecPrivKey = %v\n",ecPrivKey)
}
```

2. `ecc.ReadPublic(raw []byte) (key *ecdsa.PublicKey,err error)` attemps to parse elliptic curve public key from PKCS1 X509 or PKIX format (`BEGIN PUBLIC KEY` and `BEGIN CERTIFICATE` headers)

```Go
package main

import (
	"fmt"
    "github.com/dvsekhvalnov/jose2go/keys/ecc"
	"io/ioutil"
)

func main() {

    keyBytes, _ := ioutil.ReadFile("ec-public.key")

    ecPubKey, err:=ecc.ReadPublic(keyBytes)

    if(err!=nil) {
        panic("invalid key format")
    }

	fmt.Printf("ecPubKey = %v\n",ecPubKey)
}
```

3. `ecc.NewPublic(x,y []byte) (*ecdsa.PublicKey)` constructs elliptic public key from (X,Y) represented as bytes. Supported are NIST curves P-256,P-384 and P-521. Curve detected automatically by input length.

```Go
package main

import (
	"fmt"
    "github.com/dvsekhvalnov/jose2go/keys/ecc"
)

func main() {

    ecPubKey:=ecc.NewPublic([]byte{4, 114, 29, 223, 58, 3, 191, 170, 67, 128, 229, 33, 242, 178, 157, 150, 133, 25, 209, 139, 166, 69, 55, 26, 84, 48, 169, 165, 67, 232, 98, 9},
		 				    []byte{131, 116, 8, 14, 22, 150, 18, 75, 24, 181, 159, 78, 90, 51, 71, 159, 214, 186, 250, 47, 207, 246, 142, 127, 54, 183, 72, 72, 253, 21, 88, 53})

	fmt.Printf("ecPubKey = %v\n",ecPubKey)
}
```

4. `ecc.NewPrivate(x,y,d []byte) (*ecdsa.PrivateKey)` constructs elliptic private key from (X,Y) and D represented as bytes. Supported are NIST curves P-256,P-384 and P-521. Curve detected automatically by input length.

```Go
package main

import (
	"fmt"
    "github.com/dvsekhvalnov/jose2go/keys/ecc"
)

func main() {

    ecPrivKey:=ecc.NewPrivate([]byte{4, 114, 29, 223, 58, 3, 191, 170, 67, 128, 229, 33, 242, 178, 157, 150, 133, 25, 209, 139, 166, 69, 55, 26, 84, 48, 169, 165, 67, 232, 98, 9},
		 					  []byte{131, 116, 8, 14, 22, 150, 18, 75, 24, 181, 159, 78, 90, 51, 71, 159, 214, 186, 250, 47, 207, 246, 142, 127, 54, 183, 72, 72, 253, 21, 88, 53},
							  []byte{ 42, 148, 231, 48, 225, 196, 166, 201, 23, 190, 229, 199, 20, 39, 226, 70, 209, 148, 29, 70, 125, 14, 174, 66, 9, 198, 80, 251, 95, 107, 98, 206 })

	fmt.Printf("ecPrivKey = %v\n",ecPrivKey)
}
```

### More examples
Checkout `jose_test.go` for more examples.

## Customizing library for security
In response to ever increasing attacks on various JWT implementations, `jose2go` as of version v1.6 introduced number of additional security controls to limit potential attack surface on services and projects using the library.

### Deregister algorithm implementations
One can use following methods to deregister any signing, encryption, key management or compression algorithms from runtime suite, that is considered unsafe or simply not expected by service.

- `func DeregisterJwa(alg string) JwaAlgorithm`
- `func DeregisterJwe(alg string) JweEncryption`
- `func DeregisterJws(alg string) JwsAlgorithm`
- `func DeregisterJwc(alg string) JwcAlgorithm`

All of them expecting alg name matching `jose` constants and returns implementation that have been deregistered.

### Strict validation
Sometimes it is desirable to verify that `alg` or `enc` values are matching expected before attempting to decode actual payload.
`jose2go` provides helper matchers to be used within [Two-phase validation](#two-phase-validation) precheck:

- `jose.Alg(key, alg)` - to match alg header
- `jose.Enc(key, alg)` - to match alg and enc headers

```Go
	token := "eyJhbGciOiJSUzI1NiIsImN0eSI6InRleHRcL3BsYWluIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.NL_dfVpZkhNn4bZpCyMq5TmnXbT4yiyecuB6Kax_lV8Yq2dG8wLfea-T4UKnrjLOwxlbwLwuKzffWcnWv3LVAWfeBxhGTa0c4_0TX_wzLnsgLuU6s9M2GBkAIuSMHY6UTFumJlEeRBeiqZNrlqvmAzQ9ppJHfWWkW4stcgLCLMAZbTqvRSppC1SMxnvPXnZSWn_Fk_q3oGKWw6Nf0-j-aOhK0S0Lcr0PV69ZE4xBYM9PUS1MpMe2zF5J3Tqlc1VBcJ94fjDj1F7y8twmMT3H1PI9RozO-21R0SiXZ_a93fxhE_l_dj5drgOek7jUN9uBDjkXUwJPAyp9YPehrjyLdw"

	key := Rsa.ReadPublic(....)

	// we expecting 'RS256' alg here and if matching continue to decode with a key
	payload, header, err := jose.Decode(token, Alg(key, "RS256"))

	// or match both alg and enc for decrypting scenarios
	payload, header, err := jose.Decode(token, Enc(key, "RSA-OAEP-256", "A192CBC-HS384"))
```

### Customizing PBKDF2
As it quite easy to abuse PBES2 family of algorithms via forging header with extra large p2c values, jose-jwt library introduced iteration count limits in v1.6 to reduce runtime exposure.

By default, maxIterations is set according to [OWASP PBKDF2](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#pbkdf2) Recomendations:

```
PBES2-HS256+A128KW: 1300000
PBES2-HS384+A192KW: 950000
PBES2-HS512+A256KW: 600000
```

, while minIterations kept at 0 for backward compatibility.

If it is desired to implement different limits, register new implementation with new parameters:

```Go
	jose.RegisterJwa(NewPbse2HmacAesKWAlg(128, 1300000, 1300000))
	jose.RegisterJwa(NewPbse2HmacAesKWAlg(192, 950000, 950000))
	jose.RegisterJwa(NewPbse2HmacAesKWAlg(256, 600000, 600000))
```

In case you can't upgrade to latest version, but would like to have protections against PBES2 abuse, it is recommended to stick with [Two-phase validation](#two-phase-validation) precheck before decoding:

```Go
test, headers, err := Decode(token, func(headers map[string]interface{}, payload string) interface{} {
	alg := headers["alg"].(string)
	p2c := headers["p2c"].(float64)

	if strings.HasPrefix(alg, "PBES2-") && int64(p2c) > 100 {
		return errors.New("Too many p2c interation count, aborting")
	}

	return "top secret"
})
```

## Changelog
### 1.6
- ability to deregister specific algorithms
- configurable min/max restrictions for PBES2-HS256+A128KW, PBES2-HS384+A192KW, PBES2-HS512+A256KW

### 1.5
- security and bug fixes

### 1.4
- removed extra headers to be inserted by library

### 1.3
- security fixes: Invalid Curve Attack on NIST curves

### 1.2
- interface to access token headers after decoding
- interface to provide extra headers for token encoding
- two-phase validation support

### 1.1
- security and bug fixes

### 1.0
- initial stable version with full suite JOSE spec support
