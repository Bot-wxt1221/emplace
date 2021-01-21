use crate::{config::Config, package::Package, repo::Repo};
use anyhow::{anyhow, Context, Result};
use dialoguer::MultiSelect;
use log::{debug, error};
use std::{path::Path, process::Command};

pub fn install<P>(config_path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    // Get the config
    let config = Config::from_path_or_new(&config_path)?;

    // Get the repository from the config
    let repo = Repo::new(config, true).context("opening repository")?;

    // Read the packages from the repository
    let packages = repo.read().context("reading packages to install")?;

    println!("Checking which packages haven't been installed yet..");
    let packages_to_install: Vec<&Package> = packages
        .iter()
        // Only keep packages where we have the package manager of
        .filter(|package| package.is_available())
        // Only keep packages that can actually be installed
        .filter(|package| !package.is_installed().expect("Could not perform command"))
        // Make it a vector again
        .collect::<_>();

    let package_names = packages_to_install
        .iter()
        // Get the names
        .map(|package| package.color_full_name())
        // Make it a vector again
        .collect::<Vec<String>>();

    let package_names = package_names
        // Convert the String into a &str
        .iter()
        .map(|name| name.as_str())
        .collect::<Vec<&str>>();

    // If there's nothing to install just return
    if package_names.is_empty() {
        println!("Nothing to install.");
    } else {
        // Prompt the user for which package to install
        let selections = MultiSelect::new()
            .with_prompt("Select the packages you want to install (space to add)")
            .items(&package_names[..])
            .interact()
            .context("failed constructing checkboxes")?;

        // Install the selected packages
        for selection in selections {
            let package = packages_to_install[selection];
            println!("Installing: {}.", package.color_full_name());

            let install_command = package.install_command();
            debug!("Installing: {}.", install_command);

            match call(install_command.split_ascii_whitespace().collect()) {
                Ok(_) => println!("{} installed successfully.", package.color_full_name()),
                Err(err) => error!("\"{}\": {:?}", package.install_command(), err),
            };
        }
    }

    Ok(())
}

fn call(command: Vec<&str>) -> Result<()> {
    let mut iter = command.iter();
    let cmd_name = iter.next().unwrap();

    let mut cmd = Command::new(cmd_name);
    for arg in iter {
        if !arg.is_empty() {
            cmd.arg(arg);
        }
    }

    let result = cmd.output()?;
    // Return stderr when the command failed
    if result.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8(result.stderr)?;

        Err(anyhow!("{}", stderr))
    }
}
