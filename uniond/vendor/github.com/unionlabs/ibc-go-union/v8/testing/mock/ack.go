package mock

// EmptyAcknowledgement implements the exported.Acknowledgement interface and always returns an empty byte string as Response
type EmptyAcknowledgement struct {
	Response []byte
}

// NewEmptyAcknowledgement returns a new instance of EmptyAcknowledgement
func NewEmptyAcknowledgement() EmptyAcknowledgement {
	return EmptyAcknowledgement{
		Response: []byte{},
	}
}

// Success implements the Acknowledgement interface
func (EmptyAcknowledgement) Success() bool {
	return true
}

// Acknowledgement implements the Acknowledgement interface
func (EmptyAcknowledgement) Acknowledgement() []byte {
	return []byte{}
}
