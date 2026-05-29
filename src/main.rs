use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use cnctl::commands;

#[derive(Parser)]
#[command(name = "cnctl", about = "CLI for Cloud Native Days Norway", version)]
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
    /// Show conference status summary (sponsors, proposals, tickets, targets)
    Status {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
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
    /// Submit or update a review for a proposal
    Review(commands::proposals::ReviewArgs),
}

#[derive(Subcommand)]
enum SponsorCommand {
    /// List sponsor pipeline (interactive by default, or use flags for scripting)
    List(commands::sponsors::ListArgs),
    /// Add a new sponsor to the CRM
    Add(commands::sponsors::CreateArgs),
    /// Show sponsor details
    Get {
        /// Sponsor-for-conference ID
        id: String,
    },
    /// Show sponsor history (activities, notes, stage changes)
    History {
        /// Sponsor-for-conference ID
        id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Add a manual note/activity to a sponsor's history
    Note(commands::sponsors::NoteArgs),
    /// Send an email to a sponsor using templates
    Email(commands::sponsors::EmailArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Login => tokio::task::spawn_blocking(commands::login::run)
            .await
            .context("Login task panicked")?,
        Command::Logout => commands::logout::run(),
        Command::Status => commands::status::run(),
        Command::Admin(admin) => match admin {
            AdminCommand::Proposals(cmd) => match cmd {
                ProposalCommand::List(args) => commands::proposals::list(args).await,
                ProposalCommand::Get { id, json } => commands::proposals::get(&id, json).await,
                ProposalCommand::Review(args) => commands::proposals::review(args).await,
            },
            AdminCommand::Sponsors(cmd) => match cmd {
                SponsorCommand::List(args) => commands::sponsors::list(args).await,
                SponsorCommand::Add(args) => commands::sponsors::create(args).await,
                SponsorCommand::Get { id } => commands::sponsors::get(&id).await,
                SponsorCommand::History { id, json } => {
                    commands::sponsors::history(&id, json).await
                }
                SponsorCommand::Note(args) => commands::sponsors::add_note(args).await,
                SponsorCommand::Email(args) => commands::sponsors::email::run(args).await,
            },
            AdminCommand::Status { json } => commands::admin_status::run(json).await,
        },
    }
}
