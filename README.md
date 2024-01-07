[![Security audit](https://github.com/zachary-cauchi/rust-starter-template/actions/workflows/audit.yml/badge.svg?branch=main)](https://github.com/zachary-cauchi/rust-starter-template/actions/workflows/audit.yml) [![Tests](https://github.com/zachary-cauchi/rust-starter-template/actions/workflows/run_tests.yml/badge.svg?branch=main)](https://github.com/zachary-cauchi/rust-starter-template/actions/workflows/run_tests.yml)

# rust-starter-template
My take on a Rust project starter. It may not have everything, it may not be the best, but it is mine.

## Features

* GitHub Workflows to audit, lint, and run tests on new commits.
* Workspace project layout to separate application code from core code.
* Configuration-loading using [`config-rs`](https://github.com/mehcode/config-rs).
* Logging handled using [`tracing`](https://github.com/tokio-rs/tracing) with feature-controlled support for journal logging and rolling logfiles.
* Core-level error building using [`thiserror`](https://github.com/dtolnay/thiserror).
* Error and panic reporting using [`color-eyre`](https://github.com/eyre-rs/color-eyre).
* Dedicated package to place application code away from core integration.

## Project layout

* `cli` - Define your main app commands and cli options here. Any arguments needed that can be processed outside of the app runtime are processed here.
* `configurations` - Contains the default config file that sets the default config values at compile-time.
* `rt` - Application-level code without any engine logic.
* `src` - The top-level main function and engine-level initialisations or teardown.
* `utils` - Core types, logging setup, panic-handling, and macros for shared variables.

## Building on top of the template

* Application code goes in the `rt` runtime.
* App-level error types can be defined in `utils/core_types.rs`.
* To modify the application configuration, do so in `configuration/app_config.rs` and then modify the default_config.toml.
