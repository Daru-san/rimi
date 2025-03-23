use crate::app;
use crate::backend::error::AppError;
use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use clap_complete_nushell::Nushell;
use std::io::stdout;
use std::str::FromStr;

#[derive(Parser, Debug)]
pub struct CompletionArgs {
    /// Shell to print completions for
    #[clap(value_enum)]
    shell: ShellExt,
}

impl CompletionArgs {
    pub fn run(&self) -> Result<()> {
        match self.shell.return_shell() {
            Ok((None, Some(shell))) => {
                generate(shell, &mut app::Args::command(), "rimi", &mut stdout())
            }
            Ok((_, None)) => generate(Nushell, &mut app::Args::command(), "rimi", &mut stdout()),
            _ => return Err(AppError::UnknownShell.into()),
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct ShellExt {
    name: String,
}

impl FromStr for ShellExt {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "BASH" => Ok(ShellExt {
                name: "Bash".to_string(),
            }),
            "ZSH" => Ok(ShellExt {
                name: "Zsh".to_string(),
            }),
            "POWERSHELL" => Ok(ShellExt {
                name: "PowerShell".to_string(),
            }),
            "PWSH" => Ok(ShellExt {
                name: "PowerShell".to_string(),
            }),
            "FISH" => Ok(ShellExt {
                name: "Fish".to_string(),
            }),
            "NUSHELL" => Ok(ShellExt {
                name: "Nushell".to_string(),
            }),
            "ELVISH" => Ok(ShellExt {
                name: "Elvish".to_string(),
            }),
            _ => Err(format!("{:?} is not a known or supported shell", s)),
        }
    }
}

impl ShellExt {
    fn return_shell(&self) -> Result<(Option<Nushell>, Option<Shell>)> {
        match self.name.to_uppercase().as_str() {
            "BASH" => Ok((None, Some(Shell::Bash))),
            "ZSH" => Ok((None, Some(Shell::Zsh))),
            "POWERSHELL" => Ok((None, Some(Shell::PowerShell))),
            "PWSH" => Ok((None, Some(Shell::PowerShell))),
            "FISH" => Ok((None, Some(Shell::Fish))),
            "NUSHELL" => Ok((Some(Nushell), None)),
            "ELVISH" => Ok((None, Some(Shell::Elvish))),
            _ => Err(AppError::UnknownShell.into()),
        }
    }
}
