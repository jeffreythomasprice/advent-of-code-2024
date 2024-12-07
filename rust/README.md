Run a specific puzzle:
```
cargo test day01a --nocapture
```

... and show stdout:
```
cargo watch test day01a -- --nocapture
```

... and show time:
```
cargo test day01a -- --nocapture -Z unstable-options --report-time
```

... and in release mode:
```
cargo test --release -- -Z unstable-options --report-time
```

Flamegraphs
See also https://github.com/flamegraph-rs/flamegraph
Example given in wsl
```
sudo apt install linux-tools-generic linux-tools-common
export PATH="/usr/lib/linux-tools/6.8.0-49-generic:$PATH"
cargo flamegraph --unit-test -- day05b::tests::test_real
```
