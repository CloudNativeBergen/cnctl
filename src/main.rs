use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use cnctl::commands;
use colored::Colorize;

#[derive(Parser)]
#[command(
    name = "cnctl",
    about = "CLI for Cloud Native Days Norway — Optimized for humans and LLM agents.",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Enable agent-optimized output and hints
    #[arg(long, global = true)]
    agent: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Authenticate via browser and select a conference
    Login,
    /// Clear stored credentials
    Logout {
        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// Show current authentication and conference context
    Status,
    /// Get machine-readable context and capability map for LLM agents
    AgentInfo {
        /// Output as JSON
        #[arg(long, default_value_t = true)]
        json: bool,
    },
    /// Output full CLI specification as JSON for agent ingestion
    HelpJson,
    /// Organizer administration commands
    #[command(subcommand)]
    Admin(AdminCommand),
    /// Manage conference-specific agent instructions
    Agents(commands::agents::AgentArgs),
}

#[derive(Subcommand)]
enum AdminCommand {
    /// Manage talk proposals
    #[command(subcommand)]
    Proposals(ProposalCommand),
    /// Manage sponsor pipeline
    #[command(subcommand)]
    Sponsors(SponsorCommand),
    /// Manage speaker profiles
    #[command(subcommand)]
    Speakers(commands::speakers::SpeakerCommand),
    /// Manage featured content on the front page
    Featured(commands::featured::FeaturedArgs),
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
    /// Add a new manual proposal
    Add(commands::proposals::CreateArgs),
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
    /// Delete a proposal
    Delete(commands::proposals::DeleteArgs),
    /// Perform an action on a proposal (accept, reject, etc.)
    Action(commands::proposals::ActionArgs),
    /// Update proposal details
    Update(commands::proposals::UpdateArgs),
    /// Find the next unreviewed proposal
    NextReview,
    /// Add a speaker to an existing proposal
    AddSpeaker {
        /// Proposal ID
        proposal_id: String,
        /// Speaker ID or Email
        speaker: String,
    },
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
    let is_agent = cli.agent;

    let res = match cli.command {
        Command::Login => commands::login::run(),
        Command::Logout { yes } => commands::logout::run(yes),
        Command::Status => commands::status::run(),
        Command::AgentInfo { json } => commands::agent_discovery::run_agent_info(json).await,
        Command::HelpJson => commands::agent_discovery::run_help_json(&Cli::command()),
        Command::Admin(cmd) => match cmd {
            AdminCommand::Proposals(cmd) => match cmd {
                ProposalCommand::List(args) => commands::proposals::list(args).await,
                ProposalCommand::Add(args) => commands::proposals::add(args).await,
                ProposalCommand::Get { id, json } => commands::proposals::get(&id, json).await,
                ProposalCommand::Review(args) => commands::proposals::review(args).await,
                ProposalCommand::Delete(args) => commands::proposals::delete(args).await,
                ProposalCommand::Action(args) => commands::proposals::action(args).await,
                ProposalCommand::Update(args) => commands::proposals::update(args).await,
                ProposalCommand::NextReview => commands::proposals::next_review().await,
                ProposalCommand::AddSpeaker {
                    proposal_id,
                    speaker,
                } => commands::proposals::add_speaker(&proposal_id, &speaker).await,
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
            AdminCommand::Speakers(cmd) => match cmd {
                commands::speakers::SpeakerCommand::List(args) => {
                    commands::speakers::list(args).await
                }
                commands::speakers::SpeakerCommand::Get { id, json } => {
                    commands::speakers::get(&id, json).await
                }
                commands::speakers::SpeakerCommand::Add(args) => {
                    commands::speakers::add(args).await
                }
                commands::speakers::SpeakerCommand::Delete { id, yes } => {
                    commands::speakers::delete(&id, yes).await
                }
                commands::speakers::SpeakerCommand::Broadcast {
                    subject,
                    message,
                    sync,
                } => {
                    commands::speakers::broadcast(subject.as_deref(), message.as_deref(), sync)
                        .await
                }
                commands::speakers::SpeakerCommand::FindOrCreate(args) => {
                    commands::speakers::find_or_create(args).await
                }
            },
            AdminCommand::Featured(args) => commands::featured::run(args).await,
            AdminCommand::Status { json } => commands::admin_status::run(json).await,
        },
        Command::Agents(args) => commands::agents::run(args).await,
    };

    if let Err(e) = res {
        if is_agent {
            let mut hints = Vec::new();
            let error_str = e.to_string();

            if error_str.contains("Authentication required") || error_str.contains("unauthorized") {
                hints.push("Run 'cnctl login' to authenticate.");
            }
            if error_str.contains("not found") {
                hints.push("Use 'list' commands with '--search' or '--all' to verify IDs.");
            }
            if error_str.contains("conference context") {
                hints.push("Run 'cnctl status' to verify your active conference.");
            }

            let err_json = serde_json::json!({
                "error": error_str,
                "hints": hints
            });
            eprintln!("{}", serde_json::to_string_pretty(&err_json)?);
        } else {
            eprintln!("{} {}", "Error:".red().bold(), e);
        }
        std::process::exit(1);
    }

    Ok(())
}
