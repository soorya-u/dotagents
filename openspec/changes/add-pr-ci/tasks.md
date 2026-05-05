## 1. Create workflow file scaffold

- [ ] 1.1 Create `.github/workflows/ci.yml` with the trigger block: `on: pull_request` and `on: push: branches: [main]`
- [ ] 1.2 Add `rust-toolchain.toml` to the repo root pinning `channel = "1.92"` so the toolchain pin is also explicit outside CI

## 2. Add fmt job

- [ ] 2.1 Add `fmt` job to `ci.yml` running on `ubuntu-latest` with steps: `actions/checkout@v4`, `dtolnay/rust-toolchain@1.92`, `oven-sh/setup-bun@v2`, `Swatinem/rust-cache@v2`
- [ ] 2.2 Add step: `cargo fmt --all -- --check`
- [ ] 2.3 Add step: `cd tests/e2e && bunx biome check .`

## 3. Add clippy job

- [ ] 3.1 Add `clippy` job running on `ubuntu-latest` with steps: `actions/checkout@v4`, `dtolnay/rust-toolchain@1.92` (with `components: clippy`), `Swatinem/rust-cache@v2`
- [ ] 3.2 Add step: `cargo clippy --locked -- -D warnings`

## 4. Add test job

- [ ] 4.1 Add `test` job running on `ubuntu-latest` with steps: `actions/checkout@v4`, `dtolnay/rust-toolchain@1.92`, `Swatinem/rust-cache@v2`
- [ ] 4.2 Add step: `cargo test --bin dotagents --locked`
- [ ] 4.3 Add step: `cargo test --test integration --locked`

## 5. Add cross-platform check job

- [ ] 5.1 Add `check` job with `strategy.fail-fast: false` and matrix of four entries: `windows-latest/x86_64-pc-windows-msvc`, `ubuntu-latest/x86_64-unknown-linux-musl`, `macos-latest/x86_64-apple-darwin`, `macos-latest/aarch64-apple-darwin`
- [ ] 5.2 Add step: `rustup target add ${{ matrix.target }}`
- [ ] 5.3 Add step for the musl leg only: `sudo apt-get install -y musl-tools` (use `if: matrix.target == 'x86_64-unknown-linux-musl'`)
- [ ] 5.4 Add step: `cargo check --verbose --locked --target ${{ matrix.target }}`

## 6. Add check-publish job

- [ ] 6.1 Add `check-publish` job running on `ubuntu-latest` with steps: `actions/checkout@v4`, `dtolnay/rust-toolchain@1.92`, `Swatinem/rust-cache@v2`
- [ ] 6.2 Add step: `cargo publish --dry-run --verbose --locked`

## 7. Verification

- [ ] 7.1 Open a draft PR against `main` and confirm all five jobs appear in the Actions tab and pass
- [ ] 7.2 Verify the `check` job shows four separate matrix legs in the GitHub Actions UI
- [ ] 7.3 Confirm `cargo publish --dry-run` passes locally before merging
