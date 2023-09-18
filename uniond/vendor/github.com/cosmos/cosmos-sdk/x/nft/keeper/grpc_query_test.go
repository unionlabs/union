package keeper_test

import (
	gocontext "context"
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"

	"github.com/cosmos/cosmos-sdk/x/nft"
)

func TestGRPCQuery(t *testing.T) {
	suite.Run(t, new(TestSuite))
}

func (s *TestSuite) TestBalance() {
	var req *nft.QueryBalanceRequest
	testCases := []struct {
		msg      string
		malleate func(index int, require *require.Assertions)
		expError string
		balance  uint64
		postTest func(index int, require *require.Assertions, res *nft.QueryBalanceResponse, expBalance uint64)
	}{
		{
			"fail empty ClassId",
			func(index int, require *require.Assertions) {
				req = &nft.QueryBalanceRequest{}
			},
			nft.ErrEmptyClassID.Error(),
			0,
			func(index int, require *require.Assertions, res *nft.QueryBalanceResponse, expBalance uint64) {},
		},
		{
			"fail invalid Owner addr",
			func(index int, require *require.Assertions) {
				req = &nft.QueryBalanceRequest{
					ClassId: testClassID,
					Owner:   "owner",
				}
			},
			"decoding bech32 failed",
			0,
			func(index int, require *require.Assertions, res *nft.QueryBalanceResponse, expBalance uint64) {},
		},
		{
			"Success",
			func(index int, require *require.Assertions) {
				s.TestMint()
				req = &nft.QueryBalanceRequest{
					ClassId: testClassID,
					Owner:   s.addrs[0].String(),
				}
			},
			"",
			2,
			func(index int, require *require.Assertions, res *nft.QueryBalanceResponse, expBalance uint64) {
				require.Equal(res.Amount, expBalance, "the error occurred on:%d", index)
			},
		},
	}
	for index, tc := range testCases {
		s.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			require := s.Require()
			tc.malleate(index, require)
			result, err := s.queryClient.Balance(gocontext.Background(), req)
			if tc.expError == "" {
				require.NoError(err)
			} else {
				require.Error(err)
				require.Contains(err.Error(), tc.expError)
			}
			tc.postTest(index, require, result, tc.balance)
		})
	}
}

func (s *TestSuite) TestOwner() {
	var (
		req   *nft.QueryOwnerRequest
		owner string
	)
	testCases := []struct {
		msg      string
		malleate func(index int, require *require.Assertions)
		expError string
		postTest func(index int, require *require.Assertions, res *nft.QueryOwnerResponse)
	}{
		{
			"fail empty ClassId",
			func(index int, require *require.Assertions) {
				req = &nft.QueryOwnerRequest{
					Id: testID,
				}
			},
			nft.ErrEmptyClassID.Error(),
			func(index int, require *require.Assertions, res *nft.QueryOwnerResponse) {},
		},
		{
			"fail empty nft id",
			func(index int, require *require.Assertions) {
				req = &nft.QueryOwnerRequest{
					ClassId: testClassID,
				}
			},
			nft.ErrEmptyNFTID.Error(),
			func(index int, require *require.Assertions, res *nft.QueryOwnerResponse) {},
		},
		{
			"success but nft id not exist",
			func(index int, require *require.Assertions) {
				req = &nft.QueryOwnerRequest{
					ClassId: testClassID,
					Id:      "kitty2",
				}
			},
			"",
			func(index int, require *require.Assertions, res *nft.QueryOwnerResponse) {
				require.Equal(res.Owner, owner, "the error occurred on:%d", index)
			},
		},
		{
			"success but class id not exist",
			func(index int, require *require.Assertions) {
				req = &nft.QueryOwnerRequest{
					ClassId: "kitty1",
					Id:      testID,
				}
			},
			"",
			func(index int, require *require.Assertions, res *nft.QueryOwnerResponse) {
				require.Equal(res.Owner, owner, "the error occurred on:%d", index)
			},
		},
		{
			"Success",
			func(index int, require *require.Assertions) {
				s.TestMint()
				req = &nft.QueryOwnerRequest{
					ClassId: testClassID,
					Id:      testID,
				}
				owner = s.addrs[0].String()
			},
			"",
			func(index int, require *require.Assertions, res *nft.QueryOwnerResponse) {
				require.Equal(res.Owner, owner, "the error occurred on:%d", index)
			},
		},
	}
	for index, tc := range testCases {
		s.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			require := s.Require()
			tc.malleate(index, require)
			result, err := s.queryClient.Owner(gocontext.Background(), req)
			if tc.expError == "" {
				require.NoError(err)
			} else {
				require.Error(err)
				require.Contains(err.Error(), tc.expError)
			}
			tc.postTest(index, require, result)
		})
	}
}

func (s *TestSuite) TestSupply() {
	var req *nft.QuerySupplyRequest
	testCases := []struct {
		msg      string
		malleate func(index int, require *require.Assertions)
		expError string
		supply   uint64
		postTest func(index int, require *require.Assertions, res *nft.QuerySupplyResponse, supply uint64)
	}{
		{
			"fail empty ClassId",
			func(index int, require *require.Assertions) {
				req = &nft.QuerySupplyRequest{}
			},
			nft.ErrEmptyClassID.Error(),
			0,
			func(index int, require *require.Assertions, res *nft.QuerySupplyResponse, supply uint64) {},
		},
		{
			"success but class id not exist",
			func(index int, require *require.Assertions) {
				req = &nft.QuerySupplyRequest{
					ClassId: "kitty1",
				}
			},
			"",
			0,
			func(index int, require *require.Assertions, res *nft.QuerySupplyResponse, supply uint64) {
				require.Equal(res.Amount, supply, "the error occurred on:%d", index)
			},
		},
		{
			"success but supply equal zero",
			func(index int, require *require.Assertions) {
				req = &nft.QuerySupplyRequest{
					ClassId: testClassID,
				}
				s.TestSaveClass()
			},
			"",
			0,
			func(index int, require *require.Assertions, res *nft.QuerySupplyResponse, supply uint64) {
				require.Equal(res.Amount, supply, "the error occurred on:%d", index)
			},
		},
		{
			"Success",
			func(index int, require *require.Assertions) {
				n := nft.NFT{
					ClassId: testClassID,
					Id:      testID,
					Uri:     testURI,
				}
				err := s.nftKeeper.Mint(s.ctx, n, s.addrs[0])
				require.NoError(err, "the error occurred on:%d", index)

				req = &nft.QuerySupplyRequest{
					ClassId: testClassID,
				}
			},
			"",
			1,
			func(index int, require *require.Assertions, res *nft.QuerySupplyResponse, supply uint64) {
				require.Equal(res.Amount, supply, "the error occurred on:%d", index)
			},
		},
	}
	for index, tc := range testCases {
		s.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			require := s.Require()
			tc.malleate(index, require)
			result, err := s.queryClient.Supply(gocontext.Background(), req)
			if tc.expError == "" {
				require.NoError(err)
			} else {
				require.Error(err)
				require.Contains(err.Error(), tc.expError)
			}
			tc.postTest(index, require, result, tc.supply)
		})
	}
}

func (s *TestSuite) TestNFTs() {
	var (
		req  *nft.QueryNFTsRequest
		nfts []*nft.NFT
	)
	testCases := []struct {
		msg      string
		malleate func(index int, require *require.Assertions)
		expError string
		postTest func(index int, require *require.Assertions, res *nft.QueryNFTsResponse)
	}{
		{
			"fail empty Owner and ClassId",
			func(index int, require *require.Assertions) {
				req = &nft.QueryNFTsRequest{}
			},
			"must provide at least one of classID or owner",
			func(index int, require *require.Assertions, res *nft.QueryNFTsResponse) {},
		},
		{
			"success,empty ClassId and no nft",
			func(index int, require *require.Assertions) {
				req = &nft.QueryNFTsRequest{
					Owner: s.addrs[1].String(),
				}
				s.TestSaveClass()
			},
			"",
			func(index int, require *require.Assertions, res *nft.QueryNFTsResponse) {
				require.Len(res.Nfts, 0, "the error occurred on:%d", index)
			},
		},
		{
			"success, empty Owner and class id not exist",
			func(index int, require *require.Assertions) {
				req = &nft.QueryNFTsRequest{
					ClassId: "kitty1",
				}
				n := nft.NFT{
					ClassId: testClassID,
					Id:      testID,
					Uri:     testURI,
				}
				err := s.nftKeeper.Mint(s.ctx, n, s.addrs[0])
				require.NoError(err, "the error occurred on:%d", index)
			},
			"",
			func(index int, require *require.Assertions, res *nft.QueryNFTsResponse) {
				require.Len(res.Nfts, 0, "the error occurred on:%d", index)
			},
		},
		{
			"Success,query by owner",
			func(index int, require *require.Assertions) {
				err := s.nftKeeper.SaveClass(s.ctx, nft.Class{
					Id: "MyKitty",
				})
				require.NoError(err)

				nfts = []*nft.NFT{}
				for i := 0; i < 5; i++ {
					n := nft.NFT{
						ClassId: "MyKitty",
						Id:      fmt.Sprintf("MyCat%d", i),
					}
					err := s.nftKeeper.Mint(s.ctx, n, s.addrs[2])
					require.NoError(err)
					nfts = append(nfts, &n)
				}

				req = &nft.QueryNFTsRequest{
					Owner: s.addrs[2].String(),
				}
			},
			"",
			func(index int, require *require.Assertions, res *nft.QueryNFTsResponse) {
				require.EqualValues(res.Nfts, nfts, "the error occurred on:%d", index)
			},
		},
		{
			"Success,query by classID",
			func(index int, require *require.Assertions) {
				req = &nft.QueryNFTsRequest{
					ClassId: "MyKitty",
				}
			},
			"",
			func(index int, require *require.Assertions, res *nft.QueryNFTsResponse) {
				require.EqualValues(res.Nfts, nfts, "the error occurred on:%d", index)
			},
		},
		{
			"Success,query by classId and owner",
			func(index int, require *require.Assertions) {
				req = &nft.QueryNFTsRequest{
					ClassId: testClassID,
					Owner:   s.addrs[0].String(),
				}
				nfts = []*nft.NFT{
					{
						ClassId: testClassID,
						Id:      testID,
						Uri:     testURI,
					},
				}
			},
			"",
			func(index int, require *require.Assertions, res *nft.QueryNFTsResponse) {
				require.Equal(res.Nfts, nfts, "the error occurred on:%d", index)
			},
		},
	}
	for index, tc := range testCases {
		s.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			require := s.Require()
			tc.malleate(index, require)
			result, err := s.queryClient.NFTs(gocontext.Background(), req)
			if tc.expError == "" {
				require.NoError(err)
			} else {
				require.Error(err)
				require.Contains(err.Error(), tc.expError)
			}
			tc.postTest(index, require, result)
		})
	}
}

func (s *TestSuite) TestNFT() {
	var (
		req    *nft.QueryNFTRequest
		expNFT nft.NFT
	)
	testCases := []struct {
		msg      string
		malleate func(index int, require *require.Assertions)
		expError string
		postTest func(index int, require *require.Assertions, res *nft.QueryNFTResponse)
	}{
		{
			"fail empty ClassId",
			func(index int, require *require.Assertions) {
				req = &nft.QueryNFTRequest{}
			},
			nft.ErrEmptyClassID.Error(),
			func(index int, require *require.Assertions, res *nft.QueryNFTResponse) {},
		},
		{
			"fail empty nft id",
			func(index int, require *require.Assertions) {
				req = &nft.QueryNFTRequest{
					ClassId: testClassID,
				}
			},
			nft.ErrEmptyNFTID.Error(),
			func(index int, require *require.Assertions, res *nft.QueryNFTResponse) {},
		},
		{
			"fail ClassId not exist",
			func(index int, require *require.Assertions) {
				req = &nft.QueryNFTRequest{
					ClassId: "kitty1",
					Id:      testID,
				}
				s.TestMint()
			},
			"not found nft",
			func(index int, require *require.Assertions, res *nft.QueryNFTResponse) {},
		},
		{
			"fail nft id not exist",
			func(index int, require *require.Assertions) {
				req = &nft.QueryNFTRequest{
					ClassId: testClassID,
					Id:      "kitty2",
				}
			},
			"not found nft",
			func(index int, require *require.Assertions, res *nft.QueryNFTResponse) {},
		},
		{
			"success",
			func(index int, require *require.Assertions) {
				req = &nft.QueryNFTRequest{
					ClassId: testClassID,
					Id:      testID,
				}
				expNFT = nft.NFT{
					ClassId: testClassID,
					Id:      testID,
					Uri:     testURI,
				}
			},
			"",
			func(index int, require *require.Assertions, res *nft.QueryNFTResponse) {
				require.Equal(*res.Nft, expNFT, "the error occurred on:%d", index)
			},
		},
	}
	for index, tc := range testCases {
		s.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			require := s.Require()
			tc.malleate(index, require)
			result, err := s.queryClient.NFT(gocontext.Background(), req)
			if tc.expError == "" {
				require.NoError(err)
			} else {
				require.Error(err)
				require.Contains(err.Error(), tc.expError)
			}
			tc.postTest(index, require, result)
		})
	}
}

func (s *TestSuite) TestClass() {
	var (
		req   *nft.QueryClassRequest
		class nft.Class
	)
	testCases := []struct {
		msg      string
		malleate func(index int, require *require.Assertions)
		expError string
		postTest func(index int, require *require.Assertions, res *nft.QueryClassResponse)
	}{
		{
			"fail empty ClassId",
			func(index int, require *require.Assertions) {
				req = &nft.QueryClassRequest{}
			},
			nft.ErrEmptyClassID.Error(),
			func(index int, require *require.Assertions, res *nft.QueryClassResponse) {},
		},
		{
			"fail ClassId not exist",
			func(index int, require *require.Assertions) {
				req = &nft.QueryClassRequest{
					ClassId: "kitty1",
				}
				s.TestSaveClass()
			},
			"not found class",
			func(index int, require *require.Assertions, res *nft.QueryClassResponse) {},
		},
		{
			"success",
			func(index int, require *require.Assertions) {
				class = nft.Class{
					Id:          testClassID,
					Name:        testClassName,
					Symbol:      testClassSymbol,
					Description: testClassDescription,
					Uri:         testClassURI,
					UriHash:     testClassURIHash,
				}
				req = &nft.QueryClassRequest{
					ClassId: testClassID,
				}
			},
			"",
			func(index int, require *require.Assertions, res *nft.QueryClassResponse) {
				require.Equal(*res.Class, class, "the error occurred on:%d", index)
			},
		},
	}
	for index, tc := range testCases {
		s.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			require := s.Require()
			tc.malleate(index, require)
			result, err := s.queryClient.Class(gocontext.Background(), req)
			if tc.expError == "" {
				require.NoError(err)
			} else {
				require.Error(err)
				require.Contains(err.Error(), tc.expError)
			}
			tc.postTest(index, require, result)
		})
	}
}

func (s *TestSuite) TestClasses() {
	var (
		req     *nft.QueryClassesRequest
		classes []nft.Class
	)
	testCases := []struct {
		msg      string
		malleate func(index int, require *require.Assertions)
		expError string
		postTest func(index int, require *require.Assertions, res *nft.QueryClassesResponse)
	}{
		{
			"success Class not exist",
			func(index int, require *require.Assertions) {
				req = &nft.QueryClassesRequest{}
			},
			"",
			func(index int, require *require.Assertions, res *nft.QueryClassesResponse) {
				require.Len(res.Classes, 0)
			},
		},
		{
			"success",
			func(index int, require *require.Assertions) {
				req = &nft.QueryClassesRequest{}
				classes = []nft.Class{
					{
						Id:          testClassID,
						Name:        testClassName,
						Symbol:      testClassSymbol,
						Description: testClassDescription,
						Uri:         testClassURI,
						UriHash:     testClassURIHash,
					},
				}
				s.TestSaveClass()
			},
			"",
			func(index int, require *require.Assertions, res *nft.QueryClassesResponse) {
				require.Len(res.Classes, 1, "the error occurred on:%d", index)
				require.Equal(*res.Classes[0], classes[0], "the error occurred on:%d", index)
			},
		},
	}
	for index, tc := range testCases {
		s.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			require := s.Require()
			tc.malleate(index, require)
			result, err := s.queryClient.Classes(gocontext.Background(), req)
			if tc.expError == "" {
				require.NoError(err)
			} else {
				require.Error(err)
				require.Contains(err.Error(), tc.expError)
			}
			tc.postTest(index, require, result)
		})
	}
}
