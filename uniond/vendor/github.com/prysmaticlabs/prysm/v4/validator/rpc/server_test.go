package rpc

import (
	"net/http"
	"testing"

	"github.com/gorilla/mux"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestServer_InitializeRoutes(t *testing.T) {
	s := Server{
		router: mux.NewRouter(),
	}
	err := s.InitializeRoutes()
	require.NoError(t, err)

	wantRouteList := map[string][]string{
		"/eth/v1/keystores":                          {http.MethodGet, http.MethodPost, http.MethodDelete},
		"/eth/v1/remotekeys":                         {http.MethodGet, http.MethodPost, http.MethodDelete},
		"/eth/v1/validator/{pubkey}/gas_limit":       {http.MethodGet, http.MethodPost, http.MethodDelete},
		"/eth/v1/validator/{pubkey}/feerecipient":    {http.MethodGet, http.MethodPost, http.MethodDelete},
		"/eth/v1/validator/{pubkey}/voluntary_exit":  {http.MethodPost},
		"/v2/validator/health/version":               {http.MethodGet},
		"/v2/validator/health/logs/validator/stream": {http.MethodGet},
		"/v2/validator/health/logs/beacon/stream":    {http.MethodGet},
		"/v2/validator/wallet":                       {http.MethodGet},
		"/v2/validator/wallet/create":                {http.MethodPost},
		"/v2/validator/wallet/keystores/validate":    {http.MethodPost},
		"/v2/validator/wallet/recover":               {http.MethodPost},
		"/v2/validator/slashing-protection/export":   {http.MethodGet},
		"/v2/validator/slashing-protection/import":   {http.MethodPost},
		"/v2/validator/accounts":                     {http.MethodGet},
		"/v2/validator/accounts/backup":              {http.MethodPost},
		"/v2/validator/accounts/voluntary-exit":      {http.MethodPost},
		"/v2/validator/beacon/balances":              {http.MethodGet},
		"/v2/validator/beacon/peers":                 {http.MethodGet},
		"/v2/validator/beacon/status":                {http.MethodGet},
		"/v2/validator/beacon/summary":               {http.MethodGet},
		"/v2/validator/beacon/validators":            {http.MethodGet},
		"/v2/validator/initialize":                   {http.MethodGet},
	}
	gotRouteList := make(map[string][]string)
	err = s.router.Walk(func(route *mux.Route, router *mux.Router, ancestors []*mux.Route) error {
		tpl, err1 := route.GetPathTemplate()
		require.NoError(t, err1)
		met, err2 := route.GetMethods()
		require.NoError(t, err2)
		methods, ok := gotRouteList[tpl]
		if !ok {
			gotRouteList[tpl] = []string{met[0]}
		} else {
			gotRouteList[tpl] = append(methods, met[0])
		}
		return nil
	})
	require.NoError(t, err)
	require.DeepEqual(t, wantRouteList, gotRouteList)
}
