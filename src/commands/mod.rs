use anyhow::{Context, Result};

use crate::client::TrpcClient;
use crate::config;

pub mod login;
pub mod logout;
pub mod proposals;
pub mod sponsors;
pub mod status;

pub fn require_client() -> Result<TrpcClient> {
    let cfg = config::load().context("Not logged in. Run `cnctl login` first.")?;
    Ok(TrpcClient::from_config(&cfg))
}
