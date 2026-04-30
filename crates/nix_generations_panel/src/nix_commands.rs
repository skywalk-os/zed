use anyhow::{Context as _, Result};
use serde::Deserialize;
use std::process::Command;

/// A single NixOS generation as returned by `nixos-rebuild list-generations --json`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationJson {
    pub generation: u32,
    pub date: String,
    pub nixos_version: String,
    pub kernel_version: String,
    pub current: bool,
}

/// Parsed generation for use in the panel.
#[derive(Debug, Clone)]
pub struct Generation {
    pub number: u32,
    pub date: String,
    pub nixos_version: String,
    pub kernel_version: String,
    pub current: bool,
}

impl From<GenerationJson> for Generation {
    fn from(g: GenerationJson) -> Self {
        Self {
            number: g.generation,
            date: g.date,
            nixos_version: g.nixos_version,
            kernel_version: g.kernel_version,
            current: g.current,
        }
    }
}

/// List all NixOS system generations by shelling out to
/// `nixos-rebuild list-generations --json` (NixOS 24.05+).
pub async fn list_generations() -> Result<Vec<Generation>> {
    // Run in a blocking task so we don't block the async executor.
    smol::unblock(|| {
        let output = Command::new("nixos-rebuild")
            .args(["list-generations", "--json"])
            .output()
            .context("failed to execute nixos-rebuild list-generations --json")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!(
                "nixos-rebuild list-generations --json exited with {}: {}",
                output.status,
                stderr.trim()
            );
        }

        let stdout = String::from_utf8(output.stdout)
            .context("nixos-rebuild output was not valid UTF-8")?;

        let entries: Vec<GenerationJson> =
            serde_json::from_str(&stdout).context("failed to parse generations JSON")?;

        Ok(entries.into_iter().map(Generation::from).collect())
    })
    .await
}

/// Return the generation number of the currently booted system profile.
///
/// Strategy:
/// 1. `readlink /nix/var/nix/profiles/system` → e.g. `system-452-link`
/// 2. Parse the number out of that symlink name.
pub fn current_generation() -> Result<u32> {
    let link = std::fs::read_link("/nix/var/nix/profiles/system")
        .context("failed to readlink /nix/var/nix/profiles/system")?;

    let name = link
        .file_name()
        .and_then(|n| n.to_str())
        .context("profile symlink has no file name")?;

    // The symlink target looks like `system-<N>-link`.
    let number_str = name
        .strip_prefix("system-")
        .and_then(|s| s.strip_suffix("-link"))
        .context(format!(
            "unexpected profile symlink format: {name:?}, expected system-<N>-link"
        ))?;

    number_str
        .parse::<u32>()
        .context(format!("failed to parse generation number from {number_str:?}"))
}
