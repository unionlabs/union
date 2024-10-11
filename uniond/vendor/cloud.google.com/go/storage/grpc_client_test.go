// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package storage

import (
	"crypto/md5"
	"hash/crc32"
	"math/rand"
	"testing"
	"time"

	"cloud.google.com/go/storage/internal/apiv2/storagepb"
	"github.com/google/go-cmp/cmp"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/testing/protocmp"
)

func TestBytesCodec(t *testing.T) {
	// Generate some random content.
	content := make([]byte, 1<<10+1) // 1 kib + 1 byte
	rand.New(rand.NewSource(0)).Read(content)

	// Calculate full content hashes.
	crc32c := crc32.Checksum(content, crc32.MakeTable(crc32.Castagnoli))
	hasher := md5.New()
	if _, err := hasher.Write(content); err != nil {
		t.Errorf("hasher.Write: %v", err)
	}
	md5 := hasher.Sum(nil)

	trueBool := true
	metadata := &storagepb.Object{
		Name:               "object-name",
		Bucket:             "bucket-name",
		Etag:               "etag",
		Generation:         100,
		Metageneration:     907,
		StorageClass:       "Standard",
		Size:               1025,
		ContentEncoding:    "none",
		ContentDisposition: "inline",
		CacheControl:       "public, max-age=3600",
		Acl: []*storagepb.ObjectAccessControl{{
			Role:   "role",
			Id:     "id",
			Entity: "allUsers",
			Etag:   "tag",
			Email:  "email@foo.com",
		}},
		ContentLanguage: "mi, en",
		DeleteTime:      toProtoTimestamp(time.Now()),
		ContentType:     "application/octet-stream",
		CreateTime:      toProtoTimestamp(time.Now()),
		ComponentCount:  1,
		Checksums: &storagepb.ObjectChecksums{
			Crc32C:  &crc32c,
			Md5Hash: md5,
		},
		TemporaryHold: true,
		Metadata: map[string]string{
			"a-key": "a-value",
		},
		EventBasedHold: &trueBool,
		Owner: &storagepb.Owner{
			Entity:   "user-1",
			EntityId: "1",
		},
		CustomerEncryption: &storagepb.CustomerEncryption{
			EncryptionAlgorithm: "alg",
			KeySha256Bytes:      []byte("bytes"),
		},
		HardDeleteTime: toProtoTimestamp(time.Now()),
	}

	for _, test := range []struct {
		desc string
		resp *storagepb.ReadObjectResponse
	}{
		{
			desc: "filled object response",
			resp: &storagepb.ReadObjectResponse{
				ChecksummedData: &storagepb.ChecksummedData{
					Content: content,
					Crc32C:  &crc32c,
				},
				ObjectChecksums: &storagepb.ObjectChecksums{
					Crc32C:  &crc32c,
					Md5Hash: md5,
				},
				ContentRange: &storagepb.ContentRange{
					Start:          0,
					End:            1025,
					CompleteLength: 1025,
				},
				Metadata: metadata,
			},
		},
		{
			desc: "empty object response",
			resp: &storagepb.ReadObjectResponse{},
		},
		{
			desc: "partially empty",
			resp: &storagepb.ReadObjectResponse{
				ChecksummedData: &storagepb.ChecksummedData{},
				ObjectChecksums: &storagepb.ObjectChecksums{Md5Hash: md5},
				Metadata:        &storagepb.Object{},
			},
		},
	} {
		t.Run(test.desc, func(t *testing.T) {
			// Encode the response.
			encodedResp, err := proto.Marshal(test.resp)
			if err != nil {
				t.Fatalf("proto.Marshal: %v", err)
			}

			// Unmarshal and decode response using custom decoding.
			encodedBytes := &[]byte{}
			if err := bytesCodec.Unmarshal(bytesCodec{}, encodedResp, encodedBytes); err != nil {
				t.Fatalf("unmarshal: %v", err)
			}

			got, err := readFullObjectResponse(*encodedBytes)
			if err != nil {
				t.Fatalf("readFullObjectResponse: %v", err)
			}

			// Compare the result with the original ReadObjectResponse.
			if diff := cmp.Diff(got, test.resp, protocmp.Transform()); diff != "" {
				t.Errorf("cmp.Diff got(-),want(+):\n%s", diff)
			}
		})
	}
}
