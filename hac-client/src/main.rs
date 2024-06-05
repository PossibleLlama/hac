use hac_cli::RuntimeBehavior;
use hac_client::app;
use hac_core::collection::collection;

fn setup_tracing() -> anyhow::Result<tracing_appender::non_blocking::WorkerGuard> {
    let (data_dir, logfile) = hac_config::log_file();
    let appender = tracing_appender::rolling::never(data_dir, logfile);
    let (writer, guard) = tracing_appender::non_blocking(appender);
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .with_writer(writer)
        .with_ansi(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(guard)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let runtime_behavior = hac_cli::Cli::parse_args();

    match runtime_behavior {
        RuntimeBehavior::PrintConfigPath => hac_cli::Cli::print_config_path(
            hac_config::get_config_dir_path(),
            hac_config::get_usual_path(),
        ),
        RuntimeBehavior::PrintDataPath => {
            hac_cli::Cli::print_data_path(hac_config::get_collections_dir())
        }
        RuntimeBehavior::DumpDefaultConfig => {
            hac_cli::Cli::print_default_config(hac_config::default_as_str())
        }
        _ => {}
    }

    let dry_run = runtime_behavior.eq(&RuntimeBehavior::DryRun);

    let _guard = setup_tracing()?;
    hac_config::get_or_create_data_dir();
    let config = hac_config::load_config();

    let colors = hac_colors::Colors::default();
    let mut collections = collection::get_collections_from_config()?;
    collections.sort_by_key(|key| key.info.name.clone());
    let mut app = app::App::new(&colors, collections, &config, dry_run)?;
    app.run().await?;

    Ok(())
}
