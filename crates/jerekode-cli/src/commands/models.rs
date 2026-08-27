use clap::Args;
use jerekode_config::{CliOverrides, ConfigLoader};

use crate::util::provider_registry;

#[derive(Args, Debug)]
pub struct ModelsArgs {
    /// Optional provider id filter
    pub provider: Option<String>,

    /// Project root for config discovery
    #[arg(long)]
    pub project: Option<String>,
}

pub async fn execute(args: ModelsArgs) -> anyhow::Result<()> {
    let project = args
        .project
        .map(Into::into)
        .unwrap_or_else(|| std::env::current_dir().expect("current dir"));
    let _ = ConfigLoader::load_discovered(&project, &CliOverrides::default())?;

    let registry = provider_registry();
    let filter = args.provider.as_deref();

    for id in registry.ids() {
        if let Some(want) = filter
            && id.0 != want
        {
            continue;
        }
        let Some(provider) = registry.get(&id.0) else {
            continue;
        };
        match provider.list_models().await {
            Ok(models) => {
                for model in models {
                    println!("{}/{}", id.0, model.id);
                }
            }
            Err(e) => {
                tracing::warn!(provider = %id.0, error = %e, "failed to list models");
            }
        }
    }
    Ok(())
}
