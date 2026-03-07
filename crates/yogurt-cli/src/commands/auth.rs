//! Auth command — store Subgraph Studio deploy key.

use anyhow::Result;
use console::style;

use crate::credentials::Credentials;

pub fn run(deploy_key: &str) -> Result<()> {
    println!("{}", style("yogurt auth").bold().cyan());
    println!();

    // Validate the key isn't empty
    if deploy_key.trim().is_empty() {
        anyhow::bail!("Deploy key cannot be empty");
    }

    // Load existing credentials (or create new)
    let mut creds = Credentials::load()?;
    creds.studio_deploy_key = Some(deploy_key.to_string());
    creds.save()?;

    // Show masked key for confirmation
    let masked = if deploy_key.len() > 8 {
        format!("{}...{}", &deploy_key[..4], &deploy_key[deploy_key.len() - 4..])
    } else {
        "****".to_string()
    };

    println!("{}", style("✓ Deploy key saved").green());
    println!();
    println!("  Key: {}", style(masked).dim());
    println!("  Stored in: {}", style("~/.yogurt/credentials.json").dim());
    println!();
    println!(
        "You can now deploy to Subgraph Studio with: {}",
        style("yogurt deploy <name> --studio").yellow()
    );

    Ok(())
}
