# perforce-cli

A type-safe Rust builder library for spawning [Perforce](https://www.perforce.com/) (`p4`) commands.

Instead of concatenating raw argument strings, you construct commands with a
fluent builder API. Mutually exclusive option groups and command forms are
encoded in the type system, so invalid combinations are rejected at compile
time rather than failing at runtime.

## Features

- **Compile-time option isolation** — mutually exclusive flags are modeled as
  type-state markers. Once a mode is selected, incompatible options simply do
  not exist on the resulting type.
- **Fluent builders** — every option has a consuming builder (`x(true)`) and a
  `&mut self` setter (`set_x(true)`), paired with getters (`get_x()`).
- **Generic spawn/output traits** — run any command asynchronously with
  `spawn` or to completion with `output`, through the `ParameterizedSpawn`
  trait and its extension traits.
- **Multi-version support** — select your target Helix Core server version
  with a single Cargo feature; options introduced or removed in specific
  releases are gated automatically.
- **Zero dependencies**.

## Requirements

- A `p4` executable available on `PATH` (or a custom path passed to
  `P4Cli::new`).
- Rust 1.85+ (edition 2024).

## Usage

```toml
[dependencies]
perforce-cli = "0.1.0-alpha.3"
```

### Quick start

```rust
use std::ffi::OsStr;

use perforce_cli::spawn::ParameterizedOutput;
use perforce_cli::P4Cli;

fn main() -> std::io::Result<()> {
    let p4 = P4Cli::default();

    // `p4 print -q //depot/project/README.md`, output captured.
    let mut print = p4.print().quiet_mode(true);
    let output = print.output_with((&[OsStr::new("//depot/project/README.md")],))?;
    println!("{}", String::from_utf8_lossy(&output.stdout));

    Ok(())
}
```

Global options (the `g-opts` accepted by every `p4` command) are configured on
`GlobalOpts`:

```rust
use perforce_cli::global::GlobalOpts;
use perforce_cli::P4Cli;

let global_opts = GlobalOpts::new()
    .port("ssl:helix.example.com:1666")
    .client("my-workspace")
    .user("alice");

let p4 = P4Cli::new("p4", global_opts);
```

### Type-state: incompatible modes cannot be combined

`sync` demonstrates compile-time mode isolation. `-f`/`-k`/`-r` force a full
sync, while `-s` (safe check) and `-p` (populate) are mutually exclusive with
them:

```rust
use std::ffi::OsStr;

use perforce_cli::spawn::{ParameterizedSpawn, SpawnExt};
use perforce_cli::P4Cli;

let p4 = P4Cli::default();

// `p4 sync -f -q`
let mut force_sync = p4.sync().force(true).quiet_mode(true);
force_sync.spawn_with((Vec::<&OsStr>::new(),))?;

// `p4 sync -s`
let mut safe_sync = p4.sync().enable_safe_check();
safe_sync.spawn_with((Vec::<&OsStr>::new(),))?;

// `p4 sync -p //depot/project/...`
let mut populate = p4.sync().populate_client_workspace();
populate.spawn_with((&[OsStr::new("//depot/project/...")],))?;
```

Calling `enable_safe_check()` after `force(true)` does not compile — the
method is not defined for the force mode type, so the invalid command can never
be built.

### Commands without file arguments

Some commands take no positional arguments. Use the `spawn` / `output` methods
from `SpawnExt` / `OutputExt` instead of `spawn_with`:

```rust
use perforce_cli::spawn::OutputExt;
use perforce_cli::P4Cli;

let p4 = P4Cli::default();

// `p4 admin stop`
let output = p4.admin().stop().output()?;
```

### Spawning traits

Every command implements `ParameterizedSpawn<I>` for the input tuple shape(s) `I`
it accepts; convenience methods are provided by blanket impls:

| Trait | Arguments | Input tuple `I` |
|---|---|---|
| `SpawnExt` / `OutputExt` | none | `()` |
| `SpawnExt1` / `OutputExt1` | one | `(T1,)` |
| `SpawnExt2` / `OutputExt2` … `SpawnExt8` / `OutputExt8` | many | `(T1, …, TN)` |
| `ParameterizedSpawn` / `ParameterizedOutput` | raw input | `spawn_with(input)` / `output_with(input)` |

- `spawn` / `spawn_with` return a `std::process::Child` with stdout and stderr
  piped.
- `output` / `output_with` wait for the child and return a
  `std::process::Output`.

Import the trait matching the call you make; or bring everything into scope
with the prelude:

```rust
use perforce_cli::prelude::*;
```

## Targeting a Helix Core version

The crate carries per-version knowledge of every command's options. Enable
**exactly one** version feature for the oldest server version you need to
support; options that did not yet exist (or have been removed) are gated out of
the API.

```toml
[dependencies]
# Defaults to the newest supported release.
perforce-cli = "0.1.0-alpha.3"

# Or pin an older server line.
perforce-cli = { version = "0.1.0-alpha.3", default-features = false, features = ["v2022_2"] }
```

Supported versions: `v2014_1`, `v2014_2`, `v2015_1`, `v2015_2`, `v2016_1`,
`v2016_2`, `v2017_1`, `v2017_2`, `v2018_1`, `v2018_2`, `v2019_1`, `v2019_2`,
`v2020_1`, `v2020_2`, `v2021_1`, `v2021_2`, `v2022_1`, `v2022_2`, `v2023_1`,
`v2023_2`, `v2024_1`, `v2024_2`, `v2025_1`, `v2025_2`, `v2026_1` (default).

## Supported commands

`add`, `admin`, `aliases`, `annotate`, `archive`, `attribute`, `changes`,
`describe`, `edit`, `filelog`, `print`, `sync`, `where`.

More commands are being added.

## Examples

See [`examples/print.rs`](examples/print.rs), which covers capturing output,
chaining builders, and spawning a streaming child process.

```text
cargo run --example print
```

## License

Licensed under the Mozilla Public License, Version 2.0 ([LICENSE](LICENSE)).
