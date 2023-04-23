/*******************************************************************************
*   (c) 2018 - 2022 ZondaX AG
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License.
********************************************************************************/

package ledger_go

import (
	"encoding/binary"
	"fmt"

	"github.com/pkg/errors"
)

var codec = binary.BigEndian

func ErrorMessage(errorCode uint16) string {
	switch errorCode {
	// FIXME: Code and description don't match for 0x6982 and 0x6983 based on
	// apdu spec: https://www.eftlab.co.uk/index.php/site-map/knowledge-base/118-apdu-response-list

	case 0x6400:
		return "[APDU_CODE_EXECUTION_ERROR] No information given (NV-Ram not changed)"
	case 0x6700:
		return "[APDU_CODE_WRONG_LENGTH] Wrong length"
	case 0x6982:
		return "[APDU_CODE_EMPTY_BUFFER] Security condition not satisfied"
	case 0x6983:
		return "[APDU_CODE_OUTPUT_BUFFER_TOO_SMALL] Authentication method blocked"
	case 0x6984:
		return "[APDU_CODE_DATA_INVALID] Referenced data reversibly blocked (invalidated)"
	case 0x6985:
		return "[APDU_CODE_CONDITIONS_NOT_SATISFIED] Conditions of use not satisfied"
	case 0x6986:
		return "[APDU_CODE_COMMAND_NOT_ALLOWED] Command not allowed / User Rejected (no current EF)"
	case 0x6A80:
		return "[APDU_CODE_BAD_KEY_HANDLE] The parameters in the data field are incorrect"
	case 0x6B00:
		return "[APDU_CODE_INVALID_P1P2] Wrong parameter(s) P1-P2"
	case 0x6D00:
		return "[APDU_CODE_INS_NOT_SUPPORTED] Instruction code not supported or invalid"
	case 0x6E00:
		return "[APDU_CODE_CLA_NOT_SUPPORTED] CLA not supported"
	case 0x6E01:
		return "[APDU_CODE_APP_NOT_OPEN] Ledger Connected but Chain Specific App Not Open"
	case 0x6F00:
		return "APDU_CODE_UNKNOWN"
	case 0x6F01:
		return "APDU_CODE_SIGN_VERIFY_ERROR"
	default:
		return fmt.Sprintf("Error code: %04x", errorCode)
	}
}

func SerializePacket(
	channel uint16,
	command []byte,
	packetSize int,
	sequenceIdx uint16) (result []byte, offset int, err error) {

	if packetSize < 3 {
		return nil, 0, errors.New("Packet size must be at least 3")
	}

	var headerOffset uint8

	result = make([]byte, packetSize)
	var buffer = result

	// Insert channel (2 bytes)
	codec.PutUint16(buffer, channel)
	headerOffset += 2

	// Insert tag (1 byte)
	buffer[headerOffset] = 0x05
	headerOffset += 1

	var commandLength uint16
	commandLength = uint16(len(command))

	// Insert sequenceIdx (2 bytes)
	codec.PutUint16(buffer[headerOffset:], sequenceIdx)
	headerOffset += 2

	// Only insert total size of the command in the first package
	if sequenceIdx == 0 {
		// Insert sequenceIdx (2 bytes)
		codec.PutUint16(buffer[headerOffset:], commandLength)
		headerOffset += 2
	}

	buffer = buffer[headerOffset:]
	offset = copy(buffer, command)
	return result, offset, nil
}

func DeserializePacket(
	channel uint16,
	buffer []byte,
	sequenceIdx uint16) (result []byte, totalResponseLength uint16, isSequenceZero bool, err error) {

	isSequenceZero = false

	if (sequenceIdx == 0 && len(buffer) < 7) || (sequenceIdx > 0 && len(buffer) < 5) {
		return nil, 0, isSequenceZero, errors.New("Cannot deserialize the packet. Header information is missing.")
	}

	var headerOffset uint8

	if codec.Uint16(buffer) != channel {
		return nil, 0, isSequenceZero, errors.New(fmt.Sprintf("Invalid channel.  Expected %d, Got: %d", channel, codec.Uint16(buffer)))
	}
	headerOffset += 2

	if buffer[headerOffset] != 0x05 {
		return nil, 0, isSequenceZero, errors.New(fmt.Sprintf("Invalid tag.  Expected %d, Got: %d", 0x05, buffer[headerOffset]))
	}
	headerOffset++

	foundSequenceIdx := codec.Uint16(buffer[headerOffset:])
	if foundSequenceIdx == 0 {
		isSequenceZero = true
	} else {
		isSequenceZero = false
	}

	if foundSequenceIdx != sequenceIdx {
		return nil, 0, isSequenceZero, errors.New(fmt.Sprintf("Wrong sequenceIdx.  Expected %d, Got: %d", sequenceIdx, foundSequenceIdx))
	}
	headerOffset += 2

	if sequenceIdx == 0 {
		totalResponseLength = codec.Uint16(buffer[headerOffset:])
		headerOffset += 2
	}

	result = make([]byte, len(buffer)-int(headerOffset))
	copy(result, buffer[headerOffset:])

	return result, totalResponseLength, isSequenceZero, nil
}

// WrapCommandAPDU turns the command into a sequence of 64 byte packets
func WrapCommandAPDU(
	channel uint16,
	command []byte,
	packetSize int) (result []byte, err error) {

	var offset int
	var totalResult []byte
	var sequenceIdx uint16

	for len(command) > 0 {
		result, offset, err = SerializePacket(channel, command, packetSize, sequenceIdx)
		if err != nil {
			return nil, err
		}
		command = command[offset:]
		totalResult = append(totalResult, result...)
		sequenceIdx++
	}

	return totalResult, nil
}

// UnwrapResponseAPDU parses a response of 64 byte packets into the real data
func UnwrapResponseAPDU(channel uint16, pipe <-chan []byte, packetSize int) ([]byte, error) {
	var sequenceIdx uint16

	var totalResult []byte
	var totalSize uint16
	var done = false

	// return values from DeserializePacket
	var result []byte
	var responseSize uint16
	var err error

	foundZeroSequence := false
	isSequenceZero := false

	for !done {
		// Read next packet from the channel
		buffer := <-pipe

		result, responseSize, isSequenceZero, err = DeserializePacket(channel, buffer, sequenceIdx) // this may fail if the wrong sequence arrives (espeically if left over all 0000 was in the buffer from the last tx)
		if err != nil {
			return nil, err
		}

		// Recover from a known error condition:
		// * Discard messages left over from previous exchange until isSequenceZero == true
		if foundZeroSequence == false && isSequenceZero == false {
			continue
		}
		foundZeroSequence = true

		// Initialize totalSize (previously we did this if sequenceIdx == 0, but sometimes Nano X can provide the first sequenceIdx == 0 packet with all zeros, then a useful packet with sequenceIdx == 1
		if totalSize == 0 {
			totalSize = responseSize
		}

		buffer = buffer[packetSize:]
		totalResult = append(totalResult, result...)
		sequenceIdx++

		if len(totalResult) >= int(totalSize) {
			done = true
		}
	}

	// Remove trailing zeros
	totalResult = totalResult[:totalSize]
	return totalResult, nil
}
