slint::include_modules!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    tracing::info!("Starting Nolelamphim Desktop application...");

    let main_window = AppWindow::new()?;

    main_window.on_navigate({
        let window_weak = main_window.as_weak();
        move |page| {
            if let Some(window) = window_weak.upgrade() {
                tracing::info!("Navigating to: {}", page);
                window.set_active_page(page);
            }
        }
    });

    main_window.run()?;
    Ok(())
}
