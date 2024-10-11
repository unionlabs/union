package test

import (
	"reflect"
	"testing"

	"github.com/cometbft/cometbft/abci/tutorials/abci-v2-forum-app/model"
)

func TestParseMessage(t *testing.T) {
	// Test valid message
	tx := []byte("sender:alice,message:hello")
	expected := &model.Message{
		Sender:  "alice",
		Message: "hello",
	}
	msg, err := model.ParseMessage(tx)
	if err != nil {
		t.Errorf("ParseMessage returned error: %v", err)
	}
	if !reflect.DeepEqual(msg, expected) {
		t.Errorf("ParseMessage returned incorrect result, got: %v, want: %v", msg, expected)
	}

	// Test message with missing sender
	tx = []byte("message:hello")
	_, err = model.ParseMessage(tx)
	if err == nil {
		t.Errorf("ParseMessage did not return error for message with missing sender")
	}

	// Test message with missing message
	tx = []byte("sender:alice")
	_, err = model.ParseMessage(tx)
	if err == nil {
		t.Errorf("ParseMessage did not return error for message with missing message")
	}

	// Test message with invalid key-value pair
	tx = []byte("sender:alice,invalid_key:hello")
	_, err = model.ParseMessage(tx)
	if err == nil {
		t.Errorf("ParseMessage did not return error for message with invalid key-value pair")
	}

	// Test message with invalid number of key-value pairs
	tx = []byte("sender:alice")
	_, err = model.ParseMessage(tx)
	if err == nil {
		t.Errorf("ParseMessage did not return error for message with invalid number of key-value pairs")
	}
}
