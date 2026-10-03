# goldapi

Unofficial typed async Rust client for [GoldAPI.io](https://www.goldapi.io).
Local prototype with generated API bindings and a thin convenience client.

The low-level client and response types are generated from GoldAPI's official
[OpenAPI 3.1 specification](https://www.goldapi.io/openapi.json) with
[openapi-to-rust](https://github.com/gpu-cli/openapi-to-rust) 0.19.0. A thin wrapper
adds convenient method names, typed dates, a history-range check and shared errors.
No application models, instrument-code mappings, database or scheduler are included.

## Usage

This crate has not been published. From a sibling checkout:

```toml
[dependencies]
goldapi = { path = "../goldapi-rs" }
```

```rust,no_run
use goldapi::{Client, Currency, Metal, NaiveDate};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = Client::new("your-api-key");
let quote = client.spot_price(Metal::Xau, Currency::Usd).await?;
println!("Gold price: {:?}, as of {:?}", quote.price, quote.timestamp);

let history = client.history(
    Metal::Xau,
    Currency::Usd,
    "2025-01-01".parse::<NaiveDate>()?,
    "2025-01-31".parse::<NaiveDate>()?,
).await?;
for point in history.prices {
    println!("{}: {}", point.date, point.price);
}
# Ok(())
# }
```

The `cargo run --example spot` example reads an API key from standard input.
Running that example contacts the provider and uses your account's quota.
Ordinary tests use local mock servers and need no API key.

| Method | Documented endpoint |
|---|---|
| `spot_price` | `/api/price/{metal}/{currency}` |
| `historical_price` | `/api/price/{metal}/{currency}/{date}` |
| `history` | `/api/history/{metal}/{currency}?from=…&to=…` |
| `currency_rate` | `/api/rates/{base}/{quote}` |
| `lbma_price` | `/api/lbma/{metal}/{date}` |
| `request_statistics` | `/api/stat` |
| `status` | `/api/status` |

`Metal` and `Currency` are aliases for the generated provider enums. Use
`client.raw()` for the generated operations, typed per-operation errors and
optional `melt_price`, `currency_info` and `purity` flags.

## Behavior

- Authentication uses the `x-access-token` header. `Client` debug output hides
  the key. `with_base_url` supports mock servers and trusted proxies.
- Responses are bounded to 8 MiB by default, configurable through
  `with_max_response_body_bytes`. The generated HTTP timeout is 30 seconds.
- Errors distinguish HTTP failures, transport/body-limit failures, malformed
  success responses and invalid history ranges. `is_unauthorized` recognises
  401/403; `is_rate_limited` recognises 429. HTTP errors retain their body and
  `Retry-After` header. There are no automatic retries or background requests.
- Optional prices and timestamps remain optional. The consuming application
  must check required data and freshness before using a quote for a rule.
- History accepts 1–90 calendar days inclusive. The provider omits dates without
  data; this client never fills them with zero or invents a closing price. The
  server remains responsible for other constraints, such as rejecting future dates.
- Live/FX timestamps are seconds. LBMA timestamps are milliseconds. The raw
  values and provider field documentation are preserved.
- LBMA optional nullable fields have three states: `None` means absent,
  `Some(None)` means explicit JSON null, and `Some(Some(value))` means a value.
- Price values follow the published schema's floating-point types. The client
  does not convert units or assert that a spot series equals a futures series.

## Layout and regeneration

- `specs/goldapi.json`: unmodified provider specification retrieved 2026-10-03.
- `openapi-to-rust.toml`: generator configuration and header authentication.
- `src/generated/`: checked-in generator output, skipped by rustfmt and never
  edited by hand. `effective.json` records the input used for generation.
  `src/lib.rs` declares the generated files; the generated `mod.rs` is retained
  as an artifact but not used.
- `src/client.rs`, `src/error.rs`: handwritten convenience layer.
- `tests/fixtures/`: the provider specification's eight response examples.
  Error and boundary fixtures in the tests are synthetic.
- `mise.toml`: Rust 1.85 and pinned generator install/generate/check tasks,
  for reproducible builds. The generator builds using Rust 1.88 in its own
  `target/tools` installation.

No schema overlay is currently necessary. Add an overlay only when evidence
shows a discrepancy; retain the original specification unchanged.

```sh
mise run generate
mise run check
```

With `openapi-to-rust` 0.19.0 already installed, the equivalent checks are:

```sh
openapi-to-rust generate --config openapi-to-rust.toml --check
cargo +1.85 fmt --all -- --check
cargo +1.85 clippy --all-targets --locked -- -D warnings
cargo +1.85 test --locked
```

To update the upstream specification deliberately:

```sh
curl --fail --location https://www.goldapi.io/openapi.json --output specs/goldapi.json
mise run generate
```

Review the spec diff and refresh affected example fixtures before running the checks.
The initial specification SHA-256 is
`8f31297748bf54096050d7b2874275ceddec691b84c832b42bf65ecf404b9ebb`.

## Verification and integration status

The HTTP tests use the published examples and prove agreement with that
specification. On 2026-10-03, `cargo +1.85 run --locked --example spot`
successfully fetched and decoded an authenticated XAU/USD quote.
The documented `/api/price/XAU/USD`
endpoint returned `price`, `change`, `change_percent`, `timestamp` and `datetime`
in the expected types. Additional fields, including `unit`, `bid`, `ask` and
`price_per_unit`, were preserved by the generated response's additional-properties
map. No schema overlay was needed. The key was not printed or stored in the crate.

This live check covers spot prices only. Other endpoints remain verified against
published examples, and the checked-in fixtures remain specification examples.

The older Shareruler adapter calls `/api/{metal}/{currency}` and reads `ch`/`chp`.
This client follows the published `/api/price/…` endpoints and reads
`change`/`change_percent`. It does not silently fall back to the old API.
The spot endpoint has now been verified with an account. Replacing Shareruler's
adapter still needs its field mapping and missing-data behavior checked;
Shareruler's current provider implementation is unchanged.

## License and publication

See [RELEASING.md](RELEASING.md) for the commit, tag, verification and crates.io
publishing workflow. Run `mise run release` for a dry run, then
`mise run release:publish` to upload. Both detect the version tag at HEAD and
check that it matches `Cargo.toml`.

The handwritten wrapper is dual-licensed under
[MIT](LICENSE-MIT) and [Apache-2.0](LICENSE-APACHE).
The upstream specification identifies its licence as `Commercial`. The licence
position for redistributing the specification and derived artifacts remains
unresolved. The manifest permits publication to crates.io; the included licence
files do not relicense the provider's material or data.
