use anyhow::Result;
use clap::{Parser, Subcommand};
use cnctl::commands;

#[derive(Parser)]
#[command(name = "cnctl", about = "CLI for Cloud Native Days Norway")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Authenticate via browser and select a conference
    Login,
    /// Clear stored credentials
    Logout,
    /// Show current authentication and conference context
    Status,
    /// Organizer administration commands
    #[command(subcommand)]
    Admin(AdminCommand),
}

#[derive(Subcommand)]
enum AdminCommand {
    /// Manage talk proposals
    #[command(subcommand)]
    Proposals(ProposalCommand),
    /// Manage sponsor pipeline
    #[command(subcommand)]
    Sponsors(SponsorCommand),
}

#[derive(Subcommand)]
enum ProposalCommand {
    /// List all proposals (interactive by default, or use flags for scripting)
    List(commands::proposals::ListArgs),
    /// Show proposal details
    Get {
        /// Proposal ID
        id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum SponsorCommand {
    /// List sponsor pipeline
    List,
    /// Show sponsor details
    Get {
        /// Sponsor-for-conference ID
        id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Login => commands::login::run(),
        Command::Logout => commands::logout::run(),
        Command::Status => commands::status::run(),
        Command::Admin(admin) => match admin {
            AdminCommand::Proposals(cmd) => match cmd {
                ProposalCommand::List(args) => commands::proposals::list(args).await,
                ProposalCommand::Get { id, json } => commands::proposals::get(&id, json).await,
            },
            AdminCommand::Sponsors(cmd) => match cmd {
                SponsorCommand::List => commands::sponsors::list().await,
                SponsorCommand::Get { id } => commands::sponsors::get(&id).await,
            },
        },
    }
}
