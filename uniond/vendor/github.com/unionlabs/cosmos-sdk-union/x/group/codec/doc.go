/*
Package codec provides a singleton instance of Amino codec that should be used to register
any concrete type that can later be referenced inside a MsgSubmitProposal instance so that they
can be (de)serialized properly.

Amino types should be ideally registered inside this codec within the init function of each module's
codec.go file as follows:

	func init() {
		// ...

		RegisterLegacyAminoCodec(govcodec.Amino)
	RegisterLegacyAminoCodec(groupcodec.Amino)

	}

The codec instance is put inside this package and not the x/gov/types package in order to avoid any dependency cycle.
*/
package codec
