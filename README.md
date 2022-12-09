# Socrata Roulette

A random query generator for Socrata open government datasets indexed by [Splitgraph](https://www.splitgraph.com/explore), built with [Yew](https://yew.rs/).

See it in action on https://splitgraph.github.io/socrata-roulette.

## How does it work?

We first pick a random Socrata dataset. Then we go through its columns and classify them as:

- Measures (something that can be counted/calculated, like an `AVG(integer_column)`, `SUM(some_price_column)` etc)
- Dimensions (something that can be aggregated on, like a timestamp, an ID or a text column)

Then we pick a subset of random measures and dimensions to get and order on. We generate a query:

```sql
SELECT
  (selected dimensions),
  (selected measures)
FROM (source table)
GROUP BY (selected dimensions)
ORDER BY (some subset of measured and dimensions)
LIMIT 100
```

and render a Splitgraph query embed with that query prefilled. Splitgraph translates the query to [SoQL](https://dev.socrata.com/docs/queries/) and sends it off to the relevant Socrata data portal.

## FAQ

### I don't see anything?

This uses Yew and works as a client-side WebAssembly app, so is not supported by some browsers like IE 11.

### Sometimes the query times out

Aggregation queries can be rather heavyweight, since they require scanning through the whole dataset. Sometimes Splitgraph can't ship the whole query to the data source, so it has to load the whole dataset from the upstream and run the query locally.

Try a different one! It'll probably work better.

### Sometimes I get an `error` message

It happens if the upstream data portal is having some issues (since Splitgraph proxies the queries to it).

Try a different one! It'll probably work better.

### This doesn't work in an incognito Chrome instance

This is because the Splitgraph query embed is third-party (hosted on a different domain than `splitgraph.github.io`) and tries to use the browser's local storage to store the query text. Try a non-incognito mode for now, or a different browser like Firefox.

### I get a Splitgraph sign-in screen instead?

This is a known bug if you're already a Splitgraph user (not a fresh visitor). Log in as yourself or clean your cookies for `splitgraph.com`.

### Why?!

It's fun! We're also planning on using this feature in some future demos of Splitgraph.

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
