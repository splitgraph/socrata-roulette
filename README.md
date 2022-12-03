# Socrata Roulette

## Development

```bash
# Install prerequisites
rustup target add wasm32-unknown-unknown
cargo install --locked trunk

# Rebuild Tailwind CSS (done automatically by Trunk)
./build_tailwind.sh

# Dev mode
trunk serve --open

# Optimized release build in dist/
trunk build --release --public-url socrata-roulette/
```