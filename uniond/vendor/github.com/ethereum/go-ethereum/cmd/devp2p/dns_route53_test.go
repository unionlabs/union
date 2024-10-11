// Copyright 2020 The go-ethereum Authors
// This file is part of go-ethereum.
//
// go-ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// go-ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with go-ethereum. If not, see <http://www.gnu.org/licenses/>.

package main

import (
	"reflect"
	"testing"

	"github.com/aws/aws-sdk-go-v2/service/route53/types"
)

// This test checks that computeChanges/splitChanges create DNS changes in
// leaf-added -> root-changed -> leaf-deleted order.
func TestRoute53ChangeSort(t *testing.T) {
	testTree0 := map[string]recordSet{
		"2kfjogvxdqtxxugbh7gs7naaai.n": {ttl: 3333, values: []string{
			`"enr:-HW4QO1ml1DdXLeZLsUxewnthhUy8eROqkDyoMTyavfks9JlYQIlMFEUoM78PovJDPQrAkrb3LRJ-""vtrymDguKCOIAWAgmlkgnY0iXNlY3AyNTZrMaEDffaGfJzgGhUif1JqFruZlYmA31HzathLSWxfbq_QoQ4"`,
		}},
		"fdxn3sn67na5dka4j2gok7bvqi.n": {ttl: treeNodeTTL, values: []string{`"enrtree-branch:"`}},
		"n":                            {ttl: rootTTL, values: []string{`"enrtree-root:v1 e=2KFJOGVXDQTXXUGBH7GS7NAAAI l=FDXN3SN67NA5DKA4J2GOK7BVQI seq=0 sig=v_-J_q_9ICQg5ztExFvLQhDBGMb0lZPJLhe3ts9LAcgqhOhtT3YFJsl8BWNDSwGtamUdR-9xl88_w-X42SVpjwE"`}},
	}

	testTree1 := map[string]string{
		"n":                            "enrtree-root:v1 e=JWXYDBPXYWG6FX3GMDIBFA6CJ4 l=C7HRFPF3BLGF3YR4DY5KX3SMBE seq=1 sig=o908WmNp7LibOfPsr4btQwatZJ5URBr2ZAuxvK4UWHlsB9sUOTJQaGAlLPVAhM__XJesCHxLISo94z5Z2a463gA",
		"C7HRFPF3BLGF3YR4DY5KX3SMBE.n": "enrtree://AM5FCQLWIZX2QFPNJAP7VUERCCRNGRHWZG3YYHIUV7BVDQ5FDPRT2@morenodes.example.org",
		"JWXYDBPXYWG6FX3GMDIBFA6CJ4.n": "enrtree-branch:2XS2367YHAXJFGLZHVAWLQD4ZY,H4FHT4B454P6UXFD7JCYQ5PWDY,MHTDO6TMUBRIA2XWG5LUDACK24",
		"2XS2367YHAXJFGLZHVAWLQD4ZY.n": "enr:-HW4QOFzoVLaFJnNhbgMoDXPnOvcdVuj7pDpqRvh6BRDO68aVi5ZcjB3vzQRZH2IcLBGHzo8uUN3snqmgTiE56CH3AMBgmlkgnY0iXNlY3AyNTZrMaECC2_24YYkYHEgdzxlSNKQEnHhuNAbNlMlWJxrJxbAFvA",
		"H4FHT4B454P6UXFD7JCYQ5PWDY.n": "enr:-HW4QAggRauloj2SDLtIHN1XBkvhFZ1vtf1raYQp9TBW2RD5EEawDzbtSmlXUfnaHcvwOizhVYLtr7e6vw7NAf6mTuoCgmlkgnY0iXNlY3AyNTZrMaECjrXI8TLNXU0f8cthpAMxEshUyQlK-AM0PW2wfrnacNI",
		"MHTDO6TMUBRIA2XWG5LUDACK24.n": "enr:-HW4QLAYqmrwllBEnzWWs7I5Ev2IAs7x_dZlbYdRdMUx5EyKHDXp7AV5CkuPGUPdvbv1_Ms1CPfhcGCvSElSosZmyoqAgmlkgnY0iXNlY3AyNTZrMaECriawHKWdDRk2xeZkrOXBQ0dfMFLHY4eENZwdufn1S1o",
	}

	wantChanges := []types.Change{
		{
			Action: "CREATE",
			ResourceRecordSet: &types.ResourceRecordSet{
				Name: sp("2xs2367yhaxjfglzhvawlqd4zy.n"),
				ResourceRecords: []types.ResourceRecord{{
					Value: sp(`"enr:-HW4QOFzoVLaFJnNhbgMoDXPnOvcdVuj7pDpqRvh6BRDO68aVi5ZcjB3vzQRZH2IcLBGHzo8uUN3snqmgTiE56CH3AMBgmlkgnY0iXNlY3AyNTZrMaECC2_24YYkYHEgdzxlSNKQEnHhuNAbNlMlWJxrJxbAFvA"`),
				}},
				TTL:  ip(treeNodeTTL),
				Type: "TXT",
			},
		},
		{
			Action: "CREATE",
			ResourceRecordSet: &types.ResourceRecordSet{
				Name: sp("c7hrfpf3blgf3yr4dy5kx3smbe.n"),
				ResourceRecords: []types.ResourceRecord{{
					Value: sp(`"enrtree://AM5FCQLWIZX2QFPNJAP7VUERCCRNGRHWZG3YYHIUV7BVDQ5FDPRT2@morenodes.example.org"`),
				}},
				TTL:  ip(treeNodeTTL),
				Type: "TXT",
			},
		},
		{
			Action: "CREATE",
			ResourceRecordSet: &types.ResourceRecordSet{
				Name: sp("h4fht4b454p6uxfd7jcyq5pwdy.n"),
				ResourceRecords: []types.ResourceRecord{{
					Value: sp(`"enr:-HW4QAggRauloj2SDLtIHN1XBkvhFZ1vtf1raYQp9TBW2RD5EEawDzbtSmlXUfnaHcvwOizhVYLtr7e6vw7NAf6mTuoCgmlkgnY0iXNlY3AyNTZrMaECjrXI8TLNXU0f8cthpAMxEshUyQlK-AM0PW2wfrnacNI"`),
				}},
				TTL:  ip(treeNodeTTL),
				Type: "TXT",
			},
		},
		{
			Action: "CREATE",
			ResourceRecordSet: &types.ResourceRecordSet{
				Name: sp("jwxydbpxywg6fx3gmdibfa6cj4.n"),
				ResourceRecords: []types.ResourceRecord{{
					Value: sp(`"enrtree-branch:2XS2367YHAXJFGLZHVAWLQD4ZY,H4FHT4B454P6UXFD7JCYQ5PWDY,MHTDO6TMUBRIA2XWG5LUDACK24"`),
				}},
				TTL:  ip(treeNodeTTL),
				Type: "TXT",
			},
		},
		{
			Action: "CREATE",
			ResourceRecordSet: &types.ResourceRecordSet{
				Name: sp("mhtdo6tmubria2xwg5ludack24.n"),
				ResourceRecords: []types.ResourceRecord{{
					Value: sp(`"enr:-HW4QLAYqmrwllBEnzWWs7I5Ev2IAs7x_dZlbYdRdMUx5EyKHDXp7AV5CkuPGUPdvbv1_Ms1CPfhcGCvSElSosZmyoqAgmlkgnY0iXNlY3AyNTZrMaECriawHKWdDRk2xeZkrOXBQ0dfMFLHY4eENZwdufn1S1o"`),
				}},
				TTL:  ip(treeNodeTTL),
				Type: "TXT",
			},
		},
		{
			Action: "UPSERT",
			ResourceRecordSet: &types.ResourceRecordSet{
				Name: sp("n"),
				ResourceRecords: []types.ResourceRecord{{
					Value: sp(`"enrtree-root:v1 e=JWXYDBPXYWG6FX3GMDIBFA6CJ4 l=C7HRFPF3BLGF3YR4DY5KX3SMBE seq=1 sig=o908WmNp7LibOfPsr4btQwatZJ5URBr2ZAuxvK4UWHlsB9sUOTJQaGAlLPVAhM__XJesCHxLISo94z5Z2a463gA"`),
				}},
				TTL:  ip(rootTTL),
				Type: "TXT",
			},
		},
		{
			Action: "DELETE",
			ResourceRecordSet: &types.ResourceRecordSet{
				Name: sp("2kfjogvxdqtxxugbh7gs7naaai.n"),
				ResourceRecords: []types.ResourceRecord{
					{Value: sp(`"enr:-HW4QO1ml1DdXLeZLsUxewnthhUy8eROqkDyoMTyavfks9JlYQIlMFEUoM78PovJDPQrAkrb3LRJ-""vtrymDguKCOIAWAgmlkgnY0iXNlY3AyNTZrMaEDffaGfJzgGhUif1JqFruZlYmA31HzathLSWxfbq_QoQ4"`)},
				},
				TTL:  ip(3333),
				Type: "TXT",
			},
		},
		{
			Action: "DELETE",
			ResourceRecordSet: &types.ResourceRecordSet{
				Name: sp("fdxn3sn67na5dka4j2gok7bvqi.n"),
				ResourceRecords: []types.ResourceRecord{{
					Value: sp(`"enrtree-branch:"`),
				}},
				TTL:  ip(treeNodeTTL),
				Type: "TXT",
			},
		},
	}

	var client route53Client
	changes := client.computeChanges("n", testTree1, testTree0)
	if !reflect.DeepEqual(changes, wantChanges) {
		t.Fatalf("wrong changes (got %d, want %d)", len(changes), len(wantChanges))
	}

	// Check splitting according to size.
	wantSplit := [][]types.Change{
		wantChanges[:4],
		wantChanges[4:6],
		wantChanges[6:],
	}
	split := splitChanges(changes, 600, 4000)
	if !reflect.DeepEqual(split, wantSplit) {
		t.Fatalf("wrong split batches: got %d, want %d", len(split), len(wantSplit))
	}

	// Check splitting according to count.
	wantSplit = [][]types.Change{
		wantChanges[:5],
		wantChanges[5:],
	}
	split = splitChanges(changes, 10000, 6)
	if !reflect.DeepEqual(split, wantSplit) {
		t.Fatalf("wrong split batches: got %d, want %d", len(split), len(wantSplit))
	}
}

// This test checks that computeChanges compares the quoted value of the records correctly.
func TestRoute53NoChange(t *testing.T) {
	// Existing record set.
	testTree0 := map[string]recordSet{
		"n": {ttl: rootTTL, values: []string{
			`"enrtree-root:v1 e=JWXYDBPXYWG6FX3GMDIBFA6CJ4 l=C7HRFPF3BLGF3YR4DY5KX3SMBE seq=1 sig=o908WmNp7LibOfPsr4btQwatZJ5URBr2ZAuxvK4UWHlsB9sUOTJQaGAlLPVAhM__XJesCHxLISo94z5Z2a463gA"`,
		}},
		"2xs2367yhaxjfglzhvawlqd4zy.n": {ttl: treeNodeTTL, values: []string{
			`"enr:-HW4QOFzoVLaFJnNhbgMoDXPnOvcdVuj7pDpqRvh6BRDO68aVi5ZcjB3vzQRZH2IcLBGHzo8uUN3snqmgTiE56CH3AMBgmlkgnY0iXNlY3AyNTZrMaECC2_24YYkYHEgdzxlSNKQEnHhuNAbNlMlWJxrJxbAFvA"`,
		}},
	}
	// New set.
	testTree1 := map[string]string{
		"n":                            "enrtree-root:v1 e=JWXYDBPXYWG6FX3GMDIBFA6CJ4 l=C7HRFPF3BLGF3YR4DY5KX3SMBE seq=1 sig=o908WmNp7LibOfPsr4btQwatZJ5URBr2ZAuxvK4UWHlsB9sUOTJQaGAlLPVAhM__XJesCHxLISo94z5Z2a463gA",
		"2XS2367YHAXJFGLZHVAWLQD4ZY.n": "enr:-HW4QOFzoVLaFJnNhbgMoDXPnOvcdVuj7pDpqRvh6BRDO68aVi5ZcjB3vzQRZH2IcLBGHzo8uUN3snqmgTiE56CH3AMBgmlkgnY0iXNlY3AyNTZrMaECC2_24YYkYHEgdzxlSNKQEnHhuNAbNlMlWJxrJxbAFvA",
	}

	var client route53Client
	changes := client.computeChanges("n", testTree1, testTree0)
	if len(changes) > 0 {
		t.Fatalf("wrong changes (got %d, want 0)", len(changes))
	}
}

func sp(s string) *string { return &s }
func ip(i int64) *int64   { return &i }
