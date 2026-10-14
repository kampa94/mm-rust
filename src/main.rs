use clap::Parser;
use mm_rust::cli::{Cli, Commands};
use mm_rust::commands::{
    add_shortcut, create_file, execute_shortcut, is_file_exists,
    list_shortcuts, remove_shortcut, show_shortcut,
};
use mm_rust::utils::resolve_config_path;

fn main() {
    let cli = Cli::parse();
    let path = resolve_config_path(cli.global);
    let mut stdout = std::io::stdout();

    if !is_file_exists(&path)
        && let Err(e) = create_file(&path) {
            eprintln!("Error: cannot create config file '{path}': {e}");
            std::process::exit(1);
        }

    let result = match &cli.action {
        Commands::Add {
            short_cmd: alias,
            long_cmd: cmd,
        } => {
            if let (Some(alias), Some(cmd)) = (alias, cmd) {
                let command_string = cmd.join(" ");
                add_shortcut(path.as_str(), alias, &command_string, &mut stdout)
            } else {
                eprintln!("Error: Both alias and command are required for add");
                std::process::exit(2);
            }
        }
        Commands::Remove { short_cmd: alias } => {
            if let Some(alias) = alias {
                remove_shortcut(path.as_str(), alias, &mut stdout)
            } else {
                eprintln!("Error: alias is required for remove");
                std::process::exit(2);
            }
        }
        Commands::Show { short_cmd } => {
            if let Some(alias) = short_cmd {
                show_shortcut(path.as_str(), alias, &mut stdout)
            } else {
                eprintln!("Error: short_cmd is required for show");
                std::process::exit(2);
            }
        }
        Commands::List {} => list_shortcuts(path.as_str(), &mut stdout),
        Commands::External(args) => execute_shortcut(path.as_str(), args.clone(), &mut stdout),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
