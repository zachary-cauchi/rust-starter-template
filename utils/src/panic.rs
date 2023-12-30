use tracing::error;

use crate::core_types::CoreResult;

pub fn initialize_panic_handler() -> CoreResult<()> {
    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default()
        .panic_section(format!(
            "This is a bug. Consider reporting it at {}",
            env!("CARGO_PKG_REPOSITORY")
        ))
        .capture_span_trace_by_default(true)
        .display_location_section(true)
        .display_env_section(true)
        .into_hooks();

    eyre_hook.install()?;

    std::panic::set_hook(Box::new(move |panic_info| {
        error!("Panic detected!");

        // Print the scary stuff first.
        {
            let msg = format!("{}", panic_hook.panic_report(panic_info));
            error!("Error: {}", msg);
        }

        // If we're not running in debug mode, print an end-user-friendly panic message.
        #[cfg(not(debug_assertions))]
        {
            use human_panic::{handle_dump, print_msg, Metadata};
            let meta = Metadata {
                version: env!("CARGO_PKG_VERSION").into(),
                name: "rust-starter-template".into(),
                authors: env!("CARGO_PKG_AUTHORS").replace(':', ", ").into(),
                homepage: env!("CARGO_PKG_HOMEPAGE").into(),
            };

            let file_path = handle_dump(&meta, panic_info);

            trace!(
                "Panic report generated at {}",
                file_path.as_ref().unwrap().display()
            );

            // prints human-panic message
            print_msg(file_path, &meta)
                .expect("human-panic: printing error message to console failed");
        }

        std::process::exit(1);
    }));

    Ok(())
}
