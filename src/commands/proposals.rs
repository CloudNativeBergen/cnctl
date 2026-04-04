use anyhow::{Context, Result};

use crate::client::TrpcClient;
use crate::config;
use crate::display;
use crate::types::Proposal;

pub async fn list() -> Result<()> {
    let cfg = config::load().context("Not logged in. Run `cnctl login` first.")?;
    let client = TrpcClient::from_config(&cfg);
    list_with(&client).await
}

pub async fn get(id: &str) -> Result<()> {
    let cfg = config::load().context("Not logged in. Run `cnctl login` first.")?;
    let client = TrpcClient::from_config(&cfg);
    get_with(&client, id).await
}

pub async fn list_with(client: &TrpcClient) -> Result<()> {
    let proposals: Vec<Proposal> = client.query("proposal.admin.list", None).await?;
    display::print_proposal_list(&proposals);
    Ok(())
}

pub async fn get_with(client: &TrpcClient, id: &str) -> Result<()> {
    let input = serde_json::json!({ "id": id });
    let proposal: Proposal = client.query("proposal.admin.getById", Some(&input)).await?;
    display::print_proposal_detail(&proposal);
    Ok(())
}
