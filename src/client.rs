use crate::generated::client::HttpClient;
use crate::{
    Currency, Error, HistoricalPriceResponse, LbmaResponse, LivePriceResponse, Metal,
    MetalHistoryResponse, NaiveDate, RateResponse, RequestStatisticsResponse, StatusResponse,
};

/// Authenticated client sending the key in the `x-access-token` header.
#[derive(Clone)]
pub struct Client {
    inner: HttpClient,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client").finish_non_exhaustive()
    }
}

impl Client {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            inner: HttpClient::new().with_api_key(api_key),
        }
    }

    /// Override the server for testing or a trusted proxy.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        let base_url = base_url.into();
        self.inner = self.inner.with_base_url(base_url.trim_end_matches('/'));
        self
    }

    /// Bound the bytes read from any response. The default is 8 MiB.
    pub fn with_max_response_body_bytes(mut self, limit: usize) -> Self {
        self.inner = self.inner.with_max_response_body_bytes(limit);
        self
    }

    /// Access all generated operations, optional query flags and typed errors.
    pub fn raw(&self) -> &HttpClient {
        &self.inner
    }

    /// Latest spot quote. Optional provider fields remain optional.
    pub async fn spot_price(
        &self,
        metal: Metal,
        currency: Currency,
    ) -> Result<LivePriceResponse, Error> {
        Ok(self
            .inner
            .get_metal_price_v2(metal, currency, None, None, None)
            .await?)
    }

    /// Historical quote for a calendar date, formatted as YYYY-MM-DD.
    pub async fn historical_price(
        &self,
        metal: Metal,
        currency: Currency,
        date: NaiveDate,
    ) -> Result<HistoricalPriceResponse, Error> {
        Ok(self
            .inner
            .get_metal_price_by_date_v2(metal, currency, date.to_string(), None, None, None)
            .await?)
    }

    /// Ordered daily prices over an inclusive range of at most 90 days.
    /// Missing dates are omitted by the provider, not filled with zero prices.
    pub async fn history(
        &self,
        metal: Metal,
        currency: Currency,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<MetalHistoryResponse, Error> {
        let days = to.signed_duration_since(from).num_days();
        if !(0..90).contains(&days) {
            return Err(Error::InvalidRequest(
                "history requires an inclusive range of 1 to 90 days",
            ));
        }
        Ok(self
            .inner
            .get_metal_price_history(metal, currency, from.to_string(), to.to_string())
            .await?)
    }

    /// Current conversion rate from the base currency to the quote currency.
    pub async fn currency_rate(
        &self,
        base: Currency,
        quote: Currency,
    ) -> Result<RateResponse, Error> {
        Ok(self.inner.get_currency_rate(base, quote).await?)
    }

    /// LBMA benchmark in USD per troy ounce. This endpoint's timestamp is in
    /// milliseconds; live quotes and FX rates use seconds.
    pub async fn lbma_price(&self, metal: Metal, date: NaiveDate) -> Result<LbmaResponse, Error> {
        Ok(self
            .inner
            .get_lbma_price_by_date(metal, date.to_string())
            .await?)
    }

    /// Authenticated account request counters.
    pub async fn request_statistics(&self) -> Result<RequestStatisticsResponse, Error> {
        Ok(self.inner.get_request_statistics().await?)
    }

    /// Provider status response; preserve its optional result field.
    pub async fn status(&self) -> Result<StatusResponse, Error> {
        Ok(self.inner.get_api_status().await?)
    }
}
