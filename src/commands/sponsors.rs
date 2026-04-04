use anyhow::Result;

use super::require_client;
use crate::client::TrpcClient;
use crate::display;
use crate::types::SponsorForConference;

pub async fn list() -> Result<()> {
    let client = require_client()?;
    list_with(&client).await
}

pub async fn get(id: &str) -> Result<()> {
    let client = require_client()?;
    get_with(&client, id).await
}

pub async fn list_with(client: &TrpcClient) -> Result<()> {
    let sponsors: Vec<SponsorForConference> = client
        .query("sponsor.crm.list", Some(&serde_json::json!({})))
        .await?;
    display::print_sponsor_list(&sponsors);
    Ok(())
}

pub async fn get_with(client: &TrpcClient, id: &str) -> Result<()> {
    let sponsors: Vec<SponsorForConference> = client
        .query("sponsor.crm.list", Some(&serde_json::json!({})))
        .await?;

    let sponsor = sponsors
        .iter()
        .find(|s| s.id == id)
        .ok_or_else(|| anyhow::anyhow!("Sponsor not found: {id}"))?;

    display::print_sponsor_detail(sponsor);
    Ok(())
}
