// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Command-line argument parsing for Aeon Engine.
//!
//! Parses startup parameters passed by the `ae_launcher` or developers via CLI:
//! - `--project <PATH>`: Project directory containing `aeon_project.json`.
//! - `--mode <2D|3D>`: Active engine dimension mode enforcing RAM isolation.
//!

use std::path::PathBuf;

use ae_2d::mode::ActiveDimensionMode;

/// Parsed startup configuration flags passed to Aeon Engine on process initialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliArgs {
    /// Optional path to project directory or project descriptor file.
    pub project_path: Option<PathBuf>,

    /// Initial active dimension mode governing engine memory allocation.
    pub mode: ActiveDimensionMode,
}

impl Default for CliArgs {
    fn default() -> Self {
        Self {
            project_path: None,
            mode: ActiveDimensionMode::Mode3D,
        }
    }
}

impl CliArgs {
    /// Parses CLI arguments from the process environment (`std::env::args()`).
    pub fn from_env() -> Self {
        Self::parse_from(std::env::args())
    }

    /// Parses CLI arguments from an arbitrary sequence of string arguments.
    /// # Parameters
    /// - `args`: Sequence of argument tokens (first element may be binary name).
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
    fn test_cli_default() {
        let args: Vec<&str> = vec!["ae_engine"];
        let cli = CliArgs::parse_from(args);
        assert_eq!(cli.project_path, None);
        assert_eq!(cli.mode, ActiveDimensionMode::Mode3D);
    }

    #[test]
    fn test_cli_parse_project() {
        let args = vec!["ae_engine", "--project", "/home/user/games/my_game"];
        let cli = CliArgs::parse_from(args);
        assert_eq!(
            cli.project_path,
            Some(PathBuf::from("/home/user/games/my_game"))
        );
        assert_eq!(cli.mode, ActiveDimensionMode::Mode3D);
    }

    #[test]
    fn test_cli_parse_mode_2d() {
        let args = vec!["ae_engine", "--mode", "2D"];
        let cli = CliArgs::parse_from(args);
        assert_eq!(cli.mode, ActiveDimensionMode::Mode2D);
    }

    #[test]
    fn test_cli_parse_mode_3d() {
        let args = vec!["ae_engine", "--mode", "3D"];
        let cli = CliArgs::parse_from(args);
        assert_eq!(cli.mode, ActiveDimensionMode::Mode3D);
    }

    #[test]
    fn test_cli_combined_args() {
        let args = vec!["ae_engine", "-p", "/path/to/project", "-m", "2d"];
        let cli = CliArgs::parse_from(args);
        assert_eq!(cli.project_path, Some(PathBuf::from("/path/to/project")));
        assert_eq!(cli.mode, ActiveDimensionMode::Mode2D);
    }
}