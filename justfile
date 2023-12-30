run:
    #!/usr/bin/env bash
    cargo run

run_with_backtrace:
    #!/usr/bin/env bash
    export RUST_BACKTRACE=1
    cargo run

run_with_full_backtrace:
    #!/usr/bin/env bash
    export RUST_BACKTRACE=full
    cargo run

open_journal:
    #!/usr/bin/env bash
    journalctl -e -t rust-starter-template
