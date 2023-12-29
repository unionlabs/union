package signing

import txv1beta1 "cosmossdk.io/api/cosmos/tx/v1beta1"

// TxData is the data about a transaction that is necessary to generate sign bytes.
type TxData struct {
	// Body is the TxBody that will be part of the transaction.
	Body *txv1beta1.TxBody

	// AuthInfo is the AuthInfo that will be part of the transaction.
	AuthInfo *txv1beta1.AuthInfo

	// BodyBytes is the marshaled body bytes that will be part of TxRaw.
	BodyBytes []byte

	// AuthInfoBytes is the marshaled AuthInfo bytes that will be part of TxRaw.
	AuthInfoBytes []byte

	// BodyHasUnknownNonCriticals should be set to true if the transaction has been
	// decoded and found to have unknown non-critical fields. This is only needed
	// for amino JSON signing.
	BodyHasUnknownNonCriticals bool
}
