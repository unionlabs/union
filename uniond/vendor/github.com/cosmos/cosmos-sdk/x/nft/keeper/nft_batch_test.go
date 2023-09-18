package keeper_test

import (
	"fmt"
	"math/rand"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/nft"
)

func (s *TestSuite) TestBatchMint() {
	receiver := s.addrs[0]
	testCases := []struct {
		msg      string
		malleate func([]nft.NFT)
		tokens   []nft.NFT
		expPass  bool
	}{
		{
			"success with empty nft",
			func(tokens []nft.NFT) {
				s.saveClass(tokens)
			},
			[]nft.NFT{},
			true,
		},
		{
			"success with single nft",
			func(tokens []nft.NFT) {
				s.saveClass(tokens)
			},
			[]nft.NFT{
				{ClassId: "classID1", Id: "nftID1"},
			},
			true,
		},
		{
			"success with multiple nft",
			func(tokens []nft.NFT) {
				s.saveClass(tokens)
			},
			[]nft.NFT{
				{ClassId: "classID1", Id: "nftID1"},
				{ClassId: "classID1", Id: "nftID2"},
			},
			true,
		},
		{
			"success with multiple class and multiple nft",
			func(tokens []nft.NFT) {
				s.saveClass(tokens)
			},
			[]nft.NFT{
				{ClassId: "classID1", Id: "nftID1"},
				{ClassId: "classID1", Id: "nftID2"},
				{ClassId: "classID2", Id: "nftID1"},
				{ClassId: "classID2", Id: "nftID2"},
			},
			true,
		},
		{
			"faild with repeated nft",
			func(tokens []nft.NFT) {
				s.saveClass(tokens)
			},
			[]nft.NFT{
				{ClassId: "classID1", Id: "nftID1"},
				{ClassId: "classID1", Id: "nftID1"},
				{ClassId: "classID2", Id: "nftID2"},
			},
			false,
		},
		{
			"faild with not exist class",
			func(tokens []nft.NFT) {
				// do nothing
			},
			[]nft.NFT{
				{ClassId: "classID1", Id: "nftID1"},
				{ClassId: "classID1", Id: "nftID1"},
				{ClassId: "classID2", Id: "nftID2"},
			},
			false,
		},
		{
			"faild with exist nft",
			func(tokens []nft.NFT) {
				s.saveClass(tokens)
				idx := rand.Intn(len(tokens))
				s.nftKeeper.Mint(s.ctx, tokens[idx], receiver)
			},
			[]nft.NFT{
				{ClassId: "classID1", Id: "nftID1"},
				{ClassId: "classID1", Id: "nftID2"},
				{ClassId: "classID2", Id: "nftID2"},
			},
			false,
		},
	}

	for _, tc := range testCases {
		s.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			s.SetupTest() // reset
			tc.malleate(tc.tokens)

			err := s.nftKeeper.BatchMint(s.ctx, tc.tokens, receiver)
			if tc.expPass {
				s.Require().NoError(err)

				classMap := groupByClassID(tc.tokens)
				for classID, tokens := range classMap {
					for _, token := range tokens {
						actNFT, has := s.nftKeeper.GetNFT(s.ctx, token.ClassId, token.Id)
						s.Require().True(has)
						s.Require().EqualValues(token, actNFT)

						owner := s.nftKeeper.GetOwner(s.ctx, token.ClassId, token.Id)
						s.Require().True(receiver.Equals(owner))
					}

					actNFTs := s.nftKeeper.GetNFTsOfClass(s.ctx, classID)
					s.Require().EqualValues(tokens, actNFTs)

					actNFTs = s.nftKeeper.GetNFTsOfClassByOwner(s.ctx, classID, receiver)
					s.Require().EqualValues(tokens, actNFTs)

					balance := s.nftKeeper.GetBalance(s.ctx, classID, receiver)
					s.Require().EqualValues(len(tokens), balance)

					supply := s.nftKeeper.GetTotalSupply(s.ctx, classID)
					s.Require().EqualValues(len(tokens), supply)
				}
				return
			}
			s.Require().Error(err)
		})
	}
}

func (s *TestSuite) TestBatchBurn() {
	receiver := s.addrs[0]
	tokens := []nft.NFT{
		{ClassId: "classID1", Id: "nftID1"},
		{ClassId: "classID1", Id: "nftID2"},
		{ClassId: "classID2", Id: "nftID1"},
		{ClassId: "classID2", Id: "nftID2"},
	}

	testCases := []struct {
		msg      string
		malleate func()
		classID  string
		nftIDs   []string
		expPass  bool
	}{
		{
			"success",
			func() {
				s.saveClass(tokens)
				s.nftKeeper.BatchMint(s.ctx, tokens, receiver)
			},
			"classID1",
			[]string{"nftID1", "nftID2"},
			true,
		},
		{
			"failed with not exist classID",
			func() {},
			"classID1",
			[]string{"nftID1", "nftID2"},
			false,
		},
		{
			"failed with not exist nftID",
			func() {
				s.saveClass(tokens)
			},
			"classID1",
			[]string{"nftID1", "nftID2"},
			false,
		},
	}

	for _, tc := range testCases {
		s.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			s.SetupTest() // reset
			tc.malleate()

			err := s.nftKeeper.BatchBurn(s.ctx, tc.classID, tc.nftIDs)
			if tc.expPass {
				s.Require().NoError(err)
				for _, nftID := range tc.nftIDs {
					s.Require().False(s.nftKeeper.HasNFT(s.ctx, tc.classID, nftID))
				}
				return
			}
			s.Require().Error(err)
		})
	}
}

func (s *TestSuite) TestBatchUpdate() {
	receiver := s.addrs[0]
	tokens := []nft.NFT{
		{ClassId: "classID1", Id: "nftID1"},
		{ClassId: "classID1", Id: "nftID2"},
		{ClassId: "classID2", Id: "nftID1"},
		{ClassId: "classID2", Id: "nftID2"},
	}
	testCases := []struct {
		msg      string
		malleate func()
		tokens   []nft.NFT
		expPass  bool
	}{
		{
			"success",
			func() {
				s.saveClass(tokens)
				s.nftKeeper.BatchMint(s.ctx, tokens, receiver)
			},
			[]nft.NFT{
				{ClassId: "classID1", Id: "nftID1", Uri: "nftID1_URI"},
				{ClassId: "classID2", Id: "nftID2", Uri: "nftID2_URI"},
			},
			true,
		},
		{
			"failed with not exist classID",
			func() {},
			[]nft.NFT{
				{ClassId: "classID1", Id: "nftID1", Uri: "nftID1_URI"},
				{ClassId: "classID2", Id: "nftID2", Uri: "nftID2_URI"},
			},
			false,
		},
		{
			"failed with not exist nftID",
			func() {
				s.saveClass(tokens)
			},
			[]nft.NFT{
				{ClassId: "classID1", Id: "nftID1", Uri: "nftID1_URI"},
				{ClassId: "classID2", Id: "nftID2", Uri: "nftID2_URI"},
			},
			false,
		},
	}

	for _, tc := range testCases {
		s.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			s.SetupTest() // reset
			tc.malleate()

			err := s.nftKeeper.BatchUpdate(s.ctx, tc.tokens)
			if tc.expPass {
				s.Require().NoError(err)
				for _, token := range tc.tokens {
					actToken, found := s.nftKeeper.GetNFT(s.ctx, token.ClassId, token.Id)
					s.Require().True(found)
					s.Require().EqualValues(token, actToken)
				}
				return
			}
			s.Require().Error(err)
		})
	}
}

func (s *TestSuite) TestBatchTransfer() {
	owner := s.addrs[0]
	receiver := s.addrs[1]
	tokens := []nft.NFT{
		{ClassId: "classID1", Id: "nftID1"},
		{ClassId: "classID1", Id: "nftID2"},
		{ClassId: "classID2", Id: "nftID1"},
		{ClassId: "classID2", Id: "nftID2"},
	}
	testCases := []struct {
		msg      string
		malleate func()
		classID  string
		nftIDs   []string
		expPass  bool
	}{
		{
			"success",
			func() {
				s.saveClass(tokens)
				s.nftKeeper.BatchMint(s.ctx, tokens, owner)
			},
			"classID1",
			[]string{"nftID1", "nftID2"},
			true,
		},
		{
			"failed with not exist classID",
			func() {
				s.saveClass(tokens)
				s.nftKeeper.BatchMint(s.ctx, tokens, receiver)
			},
			"classID3",
			[]string{"nftID1", "nftID2"},
			false,
		},
		{
			"failed with not exist nftID",
			func() {
				s.saveClass(tokens)
			},
			"classID1",
			[]string{"nftID1", "nftID2"},
			false,
		},
	}

	for _, tc := range testCases {
		s.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			s.SetupTest() // reset
			tc.malleate()

			err := s.nftKeeper.BatchTransfer(s.ctx, tc.classID, tc.nftIDs, receiver)
			if tc.expPass {
				s.Require().NoError(err)
				for _, nftID := range tc.nftIDs {
					actOwner := s.nftKeeper.GetOwner(s.ctx, tc.classID, nftID)
					s.Require().EqualValues(receiver, actOwner)
				}
				return
			}
			s.Require().Error(err)
		})
	}
}

func groupByClassID(tokens []nft.NFT) map[string][]nft.NFT {
	classMap := make(map[string][]nft.NFT, len(tokens))
	for _, token := range tokens {
		if _, ok := classMap[token.ClassId]; !ok {
			classMap[token.ClassId] = make([]nft.NFT, 0)
		}
		classMap[token.ClassId] = append(classMap[token.ClassId], token)
	}
	return classMap
}

func (s *TestSuite) saveClass(tokens []nft.NFT) {
	classMap := groupByClassID(tokens)
	for classID := range classMap {
		err := s.nftKeeper.SaveClass(s.ctx, nft.Class{Id: classID})
		s.Require().NoError(err)
	}
}

func (s *TestSuite) mintNFT(tokens []nft.NFT, receiver sdk.AccAddress) {
	for _, token := range tokens {
		err := s.nftKeeper.Mint(s.ctx, token, receiver)
		s.Require().NoError(err)
	}
}
