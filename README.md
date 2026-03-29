# Payments Engine

A Rust payments engine that reads transactions from CSV, processes account state transitions, and writes final account balances as CSV.

This project follows the take-home specification for processing:

- `deposit`
- `withdrawal`
- `dispute`
- `resolve`
- `chargeback`

## Run

Build:

```bash
cargo build --release
```

Run against an input CSV:

```bash
cargo run -- transactions.csv > accounts.csv
```

The input file path is the first and only CLI argument.

## Input format

Expected header:

```csv
type,client,tx,amount
```

Where:

- `type`: one of `deposit`, `withdrawal`, `dispute`, `resolve`, `chargeback`
- `client`: `u16`
- `tx`: `u32` (globally unique id for create transactions)
- `amount`: decimal with up to 4 fractional digits (required for deposit/withdrawal)

## Output format

CSV written to stdout with header:

```csv
client,available,held,total,locked
```

Row order is not guaranteed.

## Behavior summary

- **Deposit**: increases `available` and `total`
- **Withdrawal**: decreases `available` and `total` if sufficient available funds exist
- **Dispute**: moves referenced deposit amount from `available` to `held`
- **Resolve**: moves disputed amount from `held` back to `available`
- **Chargeback**: removes disputed amount from `held` and `total`, and locks the account

Invalid operations are ignored (for example: missing referenced tx, wrong lifecycle transition, locked account operations).

## Design notes

- Uses fixed-point arithmetic (`Amount`) with scale `10_000` (4 decimal places).
- Uses per-client account state with:
  - balances (`available`, `held`, `total`, `locked`)
  - tracked deposits (`tx -> amount`) for dispute references
  - open disputes (`tx -> amount`)
- Uses a multi-worker processing model:
  - transactions are routed by `client_id % worker_count`
  - all transactions for one client stay on the same worker
- Global duplicate filtering is applied to create transactions (`deposit`/`withdrawal`) using tx id.

## Data structure backend switching

The project supports two map/set backends:

- default: `hashbrown`
- optional: std collections

Default build:

```bash
cargo build
```

Use std backend:

```bash
cargo build --no-default-features --features std-collections
```

## Testing

Run all tests:

```bash
cargo test
```

Unit tests cover core account logic, tx processor transitions, tx engine duplicate filtering and end-to-end source processing, CSV row parsing, report formatting, and in-memory account storage behavior.

## Coverage

If `cargo-llvm-cov` is installed:

```bash
cargo llvm-cov --workspace --summary-only
```

Generate HTML report:

```bash
cargo llvm-cov --workspace --html
```

Open:

`target/llvm-cov/html/index.html`
