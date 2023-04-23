//+build ledger_mock

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
	"bytes"
	"encoding/hex"
	"log"
)

type LedgerAdminMock struct{}

type LedgerDeviceMock struct{}

func NewLedgerAdmin() *LedgerAdminMock {
	return &LedgerAdminMock{}
}

func (admin *LedgerAdminMock) ListDevices() ([]string, error) {
	x := []string{"Mock device"}
	return x, nil
}

func (admin *LedgerAdminMock) CountDevices() int {
	return 1
}

func (admin *LedgerAdminMock) Connect(deviceIndex int) (*LedgerDeviceMock, error) {
	return &LedgerDeviceMock{}, nil
}

func (ledger *LedgerDeviceMock) Exchange(command []byte) ([]byte, error) {
	// Some predetermined command/replies
	infoCommand := []byte{0xE0, 0x01, 0, 0, 0}
	infoReply, _ := hex.DecodeString("311000040853706563756c6f73000b53706563756c6f734d4355")

	reply := []byte{}

	log.Printf("exchange [mock] >>> %s", hex.EncodeToString(command))

	if bytes.Equal(command, infoCommand) {
		reply = infoReply
	}
	// always return the same

	log.Printf("exchange [mock] <<< %s", hex.EncodeToString(reply))

	return reply, nil
}

func (ledger *LedgerDeviceMock) Close() error {
	// Nothing to do here
	return nil
}
