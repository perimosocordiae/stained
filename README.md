# stained

Run agents vs themselves:

```sh
# One game with all moves printed.
cargo run --example autoplay

# Many games with stats summary.
cargo run --release --example autoplay -- -q --repeats 100
```
