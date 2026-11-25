// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Command-line argument parsing for the Aeon Studio IDE desktop application.

use ae_2d::mode::ActiveDimensionMode;
use std::path::PathBuf;

/// Parsed startup configuration flags passed to the editor executable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliArgs {
    /// Optional path to project directory or project descriptor file.
    pub project_path: Option<PathBuf>,
    /// Initial active dimension mode governing workspace isolation.
    pub mode: ActiveDimensionMode,
    /// Optional local bind address (e.g. `"127.0.0.1:7878"`) enabling the live inspection server.
    pub inspect_addr: Option<String>,
}

impl Default for CliArgs {
    fn default() -> Self {
        Self {
            project_path: None,
            mode: ActiveDimensionMode::Mode3D,
            inspect_addr: None,
        }
    }
}

impl CliArgs {
    /// Parses CLI arguments from the process environment (`std::env::args()`).
    #[must_use]
    pub fn from_env() -> Self {
        Self::parse_from(std::env::args())
    }

    /// Parses CLI arguments from an arbitrary sequence of string tokens.
    pub fn parse_from<I, T>(args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        let mut cli = Self::default();
        let tokens: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();

        let mut i = 0;
        while i < tokens.len() {
            match tokens[i].as_str() {
                "--project" | "-p" if i + 1 < tokens.len() => {
                    cli.project_path = Some(PathBuf::from(&tokens[i + 1]));
                    i += 1;
                }
                "--mode" | "-m" if i + 1 < tokens.len() => {
                    let mode_str = tokens[i + 1].trim();
                    if mode_str.eq_ignore_ascii_case("2d") {
                        cli.mode = ActiveDimensionMode::Mode2D;
                    } else if mode_str.eq_ignore_ascii_case("3d") {
                        cli.mode = ActiveDimensionMode::Mode3D;
                    }
                    i += 1;
                }
                "--inspect" if i + 1 < tokens.len() => {
                    cli.inspect_addr = Some(tokens[i + 1].trim().to_string());
                    i += 1;
                }
                _ => {}
            }
            i += 1;
        }

        cli
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parses_defaults_project_mode_and_inspect() {
        assert_eq!(CliArgs::parse_from(["ae_editor_app"]), CliArgs::default());

        let parsed = CliArgs::parse_from([
            "ae_editor_app",
            "-p",
            "/path/to/project",
            "-m",
            "2d",
            "--inspect",
            "127.0.0.1:7878",
        ]);
        assert_eq!(parsed.project_path, Some(PathBuf::from("/path/to/project")));
        assert_eq!(parsed.mode, ActiveDimensionMode::Mode2D);
        assert_eq!(parsed.inspect_addr.as_deref(), Some("127.0.0.1:7878"));

        let mode_3d = CliArgs::parse_from(["ae_editor_app", "--mode", "3D", "--invalid"]);
        assert_eq!(mode_3d.mode, ActiveDimensionMode::Mode3D);
    }
}