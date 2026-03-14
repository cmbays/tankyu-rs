use anyhow::Result;

use crate::context::AppContext;

pub async fn list(
    _ctx: &AppContext,
    _state: Option<&str>,
    _signal: Option<&str>,
    _source: Option<&str>,
    _topic: Option<&str>,
    _limit: Option<usize>,
) -> Result<()> {
    todo!("entry list not yet implemented")
}

pub async fn inspect(_ctx: &AppContext, _id: &str) -> Result<()> {
    todo!("entry inspect not yet implemented")
}
