use clap::Parser;
use mm_rust::cli::{Cli, Commands};
use mm_rust::commands::{add_shortcut, create_file, execute_shortcut, is_file_exists, list_keys_for_completion, list_shortcuts, remove_shortcut, show_shortcut};
use mm_rust::utils::resolve_config_path;

fn main() {
    let cli = Cli::parse();
    let path = resolve_config_path(cli.is_global);
    let mut stdout = std::io::stdout();

    if let Commands::Complete = cli.action {
        list_keys_for_completion(resolve_config_path(false).as_str(), &mut stdout);
        list_keys_for_completion(resolve_config_path(true).as_str(), &mut stdout);
        return;
    }
    let exists = is_file_exists(&path, &mut stdout).unwrap_or(false);
    if !exists {
        create_file(&path, &mut stdout).unwrap_or_else(|e| {
            eprintln!("Error creating config file: {}", e);
            std::process::exit(1);
        });
    }

    match &cli.action {
        Commands::Add { short_cmd: alias, long_cmd: cmd } => {
            if let (Some(alias), Some(cmd)) = (alias, cmd) {
                let command_string = cmd.join(" ");
                add_shortcut(path.as_str(), alias, &command_string, &mut stdout);
            } else {
                eprintln!("Error: Both alias and command are required for add");
            }
        }
        Commands::Remove { short_cmd: alias } => {
            if let Some(alias) = alias {
                remove_shortcut(path.as_str(), alias, &mut stdout);
            } else {
                eprintln!("Error: alias is required for remove");
            }
        }
        Commands::Show { short_cmd } => {
            if let Some(alias) = short_cmd {
                show_shortcut(path.as_str(), alias, &mut stdout);
            } else {
                eprintln!("Error: short_cmd is required for show");
            }
        }
        Commands::List {} => {
            list_shortcuts(path.as_str(), &mut stdout);
        }
        Commands::External(args) => {
            execute_shortcut(path.as_str(), args.clone(), &mut stdout);
        }
        Commands::Complete => {
            // Handled above
        }
    }
}
