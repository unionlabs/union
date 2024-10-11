package secretservice

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestNewKeypair(t *testing.T) {
	group := rfc2409SecondOakleyGroup()
	private, public, err := group.NewKeypair()
	require.NoError(t, err)
	require.NotNil(t, private)
	require.NotNil(t, public)
	private2, public2, err := group.NewKeypair()
	require.NoError(t, err)
	require.NotEqual(t, private.Cmp(private2), 0, "should get different private key with every keygen")
	require.NotEqual(t, public.Cmp(public2), 0, "should get different public key with every keygen")
}

func TestKeygen(t *testing.T) {
	group := rfc2409SecondOakleyGroup()
	myPrivate, myPublic, err := group.NewKeypair()
	require.NoError(t, err)
	theirPrivate, theirPublic, err := group.NewKeypair()
	require.NoError(t, err)

	myKey, err := group.keygenHKDFSHA256AES128(theirPublic, myPrivate)
	require.NoError(t, err)
	theirKey, err := group.keygenHKDFSHA256AES128(myPublic, theirPrivate)
	require.NoError(t, err)
	require.Equal(t, myKey, theirKey)
}

func TestEncryption(t *testing.T) {
	key := []byte("YELLOW SUBMARINE")
	plaintext := []byte("hello world")
	iv, ciphertext, err := unauthenticatedAESCBCEncrypt(plaintext, key)
	require.NoError(t, err)
	gotPlaintext, err := unauthenticatedAESCBCDecrypt(iv, ciphertext, key)
	require.NoError(t, err)
	require.Equal(t, plaintext, gotPlaintext)
}

func TestEncryptionRng(t *testing.T) {
	key := []byte("YELLOW SUBMARINE")
	plaintext := []byte("hello world")
	iv1, ciphertext1, err := unauthenticatedAESCBCEncrypt(plaintext, key)
	require.NoError(t, err)
	iv2, ciphertext2, err := unauthenticatedAESCBCEncrypt(plaintext, key)
	require.NoError(t, err)
	require.NotEqual(t, iv1, iv2)
	require.NotEqual(t, ciphertext1, ciphertext2)
}

var pkcs7tests = []struct {
	in  []byte
	out []byte
}{
	{[]byte{}, []byte{4, 4, 4, 4}},
	{[]byte{1, 2}, []byte{1, 2, 2, 2}},
	{[]byte{1, 2, 3}, []byte{1, 2, 3, 1}},
	{[]byte{1, 2, 3, 4}, []byte{1, 2, 3, 4, 4, 4, 4, 4}},
	{[]byte{1, 2, 3, 4, 5}, []byte{1, 2, 3, 4, 5, 3, 3, 3}},
	{[]byte{1, 2, 3, 4, 1, 1, 1}, []byte{1, 2, 3, 4, 1, 1, 1, 1}},
}

func TestPKCS7(t *testing.T) {
	for _, testCase := range pkcs7tests {
		require.Equal(t, padPKCS7(testCase.in, 4), testCase.out)
		preimage, err := unpadPKCS7(testCase.out, 4)
		require.NoError(t, err)
		require.Equal(t, preimage, testCase.in)
	}

	_, err := unpadPKCS7([]byte{}, 4)
	require.Error(t, err)
	_, err = unpadPKCS7([]byte{1, 2, 3, 4}, 4)
	require.Error(t, err)
	_, err = unpadPKCS7([]byte{1, 2, 3, 3}, 4)
	require.Error(t, err)
	_, err = unpadPKCS7([]byte{1, 2, 3, 4, 1, 1, 1, 2}, 4)
	require.Error(t, err)
}
