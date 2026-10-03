//! HTTP contract tests using the official OpenAPI examples, not live captures.
use goldapi::{Client, Currency, Error, Metal, NaiveDate};
use serde_json::json;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const KEY: &str = "fixture-key";

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/fixtures/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

fn date(value: &str) -> NaiveDate {
    value.parse().unwrap()
}

fn client(server: &MockServer) -> Client {
    Client::new(KEY).with_base_url(format!("{}/", server.uri()))
}

async fn mount(server: &MockServer, route: &str, name: &str) {
    Mock::given(method("GET"))
        .and(path(route))
        .and(header("x-access-token", KEY))
        .and(header("accept", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture(name)))
        .expect(1)
        .mount(server)
        .await;
}

#[tokio::test]
async fn spot_quote_uses_documented_endpoint_auth_and_fields() {
    let server = MockServer::start().await;
    mount(&server, "/api/price/XAU/USD", "liveGoldUsd").await;
    let quote = client(&server)
        .spot_price(Metal::Xau, Currency::Usd)
        .await
        .unwrap();
    assert_eq!(quote.price, Some(2391.72));
    assert_eq!(quote.change, Some(11.6));
    assert_eq!(quote.change_percent, Some(0.49));
    assert_eq!(quote.timestamp, Some(1716000000));
    assert_eq!(quote.metal, Some(Metal::Xau));
    assert_eq!(
        quote.datetime.unwrap().to_rfc3339(),
        "2024-05-18T02:40:00+00:00"
    );
    let requests = server.received_requests().await.unwrap();
    assert!(
        requests[0].url.query().is_none(),
        "API key must not be in URL"
    );
}

#[tokio::test]
async fn historical_quote_uses_calendar_date() {
    let server = MockServer::start().await;
    mount(
        &server,
        "/api/price/XAU/USD/2025-01-01",
        "historicalGoldUsd",
    )
    .await;
    let quote = client(&server)
        .historical_price(Metal::Xau, Currency::Usd, date("2025-01-01"))
        .await
        .unwrap();
    assert_eq!(quote.price, Some(2624.18));
    assert_eq!(quote.date.as_deref(), Some("2025-01-01"));
}

#[tokio::test]
async fn history_sends_range_and_preserves_missing_dates() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/history/XAU/USD"))
        .and(header("x-access-token", KEY))
        .and(query_param("from", "2025-01-01"))
        .and(query_param("to", "2025-01-03"))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture("goldUsdHistory")))
        .expect(1)
        .mount(&server)
        .await;
    let history = client(&server)
        .history(
            Metal::Xau,
            Currency::Usd,
            date("2025-01-01"),
            date("2025-01-03"),
        )
        .await
        .unwrap();
    assert_eq!(history.prices.len(), 2);
    assert_eq!(history.prices[0].date, date("2025-01-02"));
    assert_eq!(history.prices[0].price, 2658.12);
    assert_eq!(history.from, date("2025-01-01"));
}

#[tokio::test]
async fn history_checks_inclusive_ninety_day_limit_before_request() {
    let server = MockServer::start().await;
    let client = client(&server);
    for (from, to) in [("2025-01-02", "2025-01-01"), ("2025-01-01", "2025-04-01")] {
        let error = client
            .history(Metal::Xau, Currency::Usd, date(from), date(to))
            .await
            .unwrap_err();
        assert!(matches!(error, Error::InvalidRequest(_)));
    }
    assert!(server.received_requests().await.unwrap().is_empty());
    Mock::given(path("/api/history/XAU/USD"))
        .and(query_param("to", "2025-03-31"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metal":"XAU", "currency":"USD", "from":"2025-01-01", "to":"2025-03-31", "prices":[]
        })))
        .expect(1)
        .mount(&server)
        .await;
    assert!(
        client
            .history(
                Metal::Xau,
                Currency::Usd,
                date("2025-01-01"),
                date("2025-03-31")
            )
            .await
            .unwrap()
            .prices
            .is_empty()
    );
}

#[tokio::test]
async fn currency_rate_preserves_base_quote_and_timestamp() {
    let server = MockServer::start().await;
    mount(&server, "/api/rates/USD/EUR", "usdEur").await;
    let rate = client(&server)
        .currency_rate(Currency::Usd, Currency::Eur)
        .await
        .unwrap();
    assert_eq!(rate.price, 0.963245);
    assert_eq!(rate.base, Currency::Usd);
    assert_eq!(rate.quote, Currency::Eur);
    assert_eq!(rate.timestamp, 1735689600);
}

#[tokio::test]
async fn lbma_preserves_gold_fixes_silver_price_and_millisecond_timestamp() {
    let server = MockServer::start().await;
    mount(&server, "/api/lbma/XAU/2025-01-01", "goldFixes").await;
    mount(&server, "/api/lbma/XAG/2025-01-01", "silverPrice").await;
    let gold = client(&server)
        .lbma_price(Metal::Xau, date("2025-01-01"))
        .await
        .unwrap();
    assert_eq!(gold.am, Some(Some(2624.5)));
    assert_eq!(gold.pm, Some(Some(2627.25)));
    assert_eq!(gold.price, None);
    assert_eq!(gold.timestamp, 1735689600000);
    let silver = client(&server)
        .lbma_price(Metal::Xag, date("2025-01-01"))
        .await
        .unwrap();
    assert_eq!(silver.am, None);
    assert_eq!(silver.price, Some(Some(29.41)));
}

#[tokio::test]
async fn status_and_usage_are_authenticated_and_decoded() {
    let server = MockServer::start().await;
    mount(&server, "/api/status", "ok").await;
    mount(&server, "/api/stat", "usage").await;
    assert_eq!(client(&server).status().await.unwrap().result, Some(true));
    assert_eq!(
        client(&server)
            .request_statistics()
            .await
            .unwrap()
            .requests_month,
        2437
    );
}

#[tokio::test]
async fn raw_client_exposes_optional_query_flags() {
    let server = MockServer::start().await;
    Mock::given(path("/api/price/XAU/USD"))
        .and(header("x-access-token", KEY))
        .and(query_param("melt_price", "true"))
        .and(query_param("currency_info", "false"))
        .and(query_param("purity", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture("liveGoldUsd")))
        .expect(1)
        .mount(&server)
        .await;
    client(&server)
        .raw()
        .get_metal_price_v2(
            Metal::Xau,
            Currency::Usd,
            Some(true),
            Some(false),
            Some(true),
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn unauthorized_forbidden_and_quota_errors_keep_status_and_body() {
    for status in [401, 403, 429] {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(status)
                    .insert_header("retry-after", "60")
                    .set_body_json(json!({"error":"rejected"})),
            )
            .mount(&server)
            .await;
        let error = client(&server)
            .spot_price(Metal::Xau, Currency::Usd)
            .await
            .unwrap_err();
        assert_eq!(error.is_unauthorized(), status != 429);
        assert_eq!(error.is_rate_limited(), status == 429);
        match error {
            Error::Api {
                status: actual,
                body,
                retry_after,
            } => {
                assert_eq!(actual, status);
                assert!(body.contains("rejected"));
                assert_eq!(retry_after.as_deref(), Some("60"));
            }
            error => panic!("unexpected error: {error}"),
        }
    }
}

#[tokio::test]
async fn malformed_success_is_distinct_from_http_failure() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"price":"bad"}"#))
        .mount(&server)
        .await;
    assert!(matches!(
        client(&server).spot_price(Metal::Xau, Currency::Usd).await,
        Err(Error::Decode(_))
    ));
}

#[tokio::test]
async fn absent_price_is_not_invented_as_zero() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"metal":"XAU"})))
        .mount(&server)
        .await;
    let quote = client(&server)
        .spot_price(Metal::Xau, Currency::Usd)
        .await
        .unwrap();
    assert_eq!(quote.price, None);
    assert_eq!(quote.timestamp, None);
}

#[tokio::test]
async fn response_size_limit_is_enforced() {
    let server = MockServer::start().await;
    mount(&server, "/api/price/XAU/USD", "liveGoldUsd").await;
    let result = client(&server)
        .with_max_response_body_bytes(16)
        .spot_price(Metal::Xau, Currency::Usd)
        .await;
    assert!(matches!(
        result,
        Err(Error::Transport(
            goldapi::generated::client::HttpError::ResponseTooLarge { limit: 16 }
        ))
    ));
}

#[test]
fn client_debug_omits_api_key() {
    assert!(!format!("{:?}", Client::new(KEY)).contains(KEY));
}
