package strcase

import (
	"testing"
)

func TestConcurrency(t *testing.T) {
	for i := 0; i < 10000; i++ {
		go doConvert()
	}

}

func doConvert() {
	var snakes = []string{"code", "exchange", "pe_ratio", "profit_margin", "updated_date"}
	for _, v := range snakes {
		_ = convertDatabaseNameToCamelCase(v)
	}
}

func convertDatabaseNameToCamelCase(d string) (s string) {
	ConfigureAcronym("price_book_mrq", "PriceBookMRQ")
	ConfigureAcronym("pe_ratio", "PERatio")
	ConfigureAcronym("peg_ratio", "PEGRatio")
	ConfigureAcronym("eps_estimate_current_year", "EPSEstimateCurrentYear")
	ConfigureAcronym("eps_estimate_next_year", "EPSEstimateNextYear")
	ConfigureAcronym("eps_estimate_next_quarter", "EPSNextQuarter")
	ConfigureAcronym("eps_estimate_current_quarter", "EPSEstimateCurrentQuarter")
	ConfigureAcronym("ebitda", "EBITDA")
	ConfigureAcronym("operating_margin_ttm", "OperatingMarginTTM")
	ConfigureAcronym("return_on_assets_ttm", "ReturnOnAssetsTTM")
	ConfigureAcronym("return_on_equity_ttm", "ReturnOnEquityTTM")
	ConfigureAcronym("revenue_ttm", "RevenueTTM")
	ConfigureAcronym("revenue_per_share_ttm", "RevenuePerShareTTM")
	ConfigureAcronym("quarterly_revenue_growth_yoy", "QuarterlyRevenueGrowthYOY")
	ConfigureAcronym("gross_profit_ttm", "GrossProfitTTM")
	ConfigureAcronym("diluted_eps_ttm", "DilutedEpsTTM")
	ConfigureAcronym("quarterly_earnings_growth_yoy", "QuarterlyEarningsGrowthYOY")
	ConfigureAcronym("two_hundred_day_ma", "TwoHundredDayMA")
	ConfigureAcronym("trailing_pe", "TrailingPE")
	ConfigureAcronym("forward_pe", "ForwardPE")
	ConfigureAcronym("price_sales_ttm", "PriceSalesTTM")
	ConfigureAcronym("price_book_mrq", "PriceBookMRQ")
	s = ToCamel(d)
	return
}
