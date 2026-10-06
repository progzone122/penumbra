/*
    SPDX-License-Identifier: AGPL-3.0-or-later
    SPDX-FileCopyrightText: 2026 Shomy
*/
pub mod remote;
use std::sync::Arc;

use anyhow::Result;
use penumbra::AuthManager;
pub use remote::RemoteSigner;

use crate::config::AntumbraConfig;

pub fn init_auth(config: Arc<AntumbraConfig>) -> Result<()> {
    let auth = AuthManager::get();

    let signer = Arc::new(RemoteSigner::new(config));

    auth.register_signer(signer)?;

    Ok(())
}
