use anyhow::{anyhow, Context, Result};
use files::File;
use futures::{FutureExt, StreamExt, TryStreamExt};
use tokio::sync::mpsc::UnboundedSender;

use crate::types::*;
use Function::*;

macro_rules! res {
    ($res_sender: expr, $id: expr, $fut: expr) => {{
        let res = $fut.map(|r| Response::new($id, r.into())).await;
        tokio::task::spawn_blocking(move || {
            let res = serde_json::to_vec(&res)
                .with_context(|| format!("could not serialize `{res:?}`"))?;
            $res_sender
                .send(res)
                .with_context(|| format!("response channel closed"))?;
            anyhow::Ok(())
        })
        .await?
    }};
}

macro_rules! stream_res {
    ($res_sender: expr, $id: expr, $stream: expr) => {{
        $stream
            .map(|r| Response::new($id, r.into()))
            .map(|r| {
                let res_sender = $res_sender.clone();
                tokio::task::spawn_blocking(move || {
                    let res = serde_json::to_vec(&r)
                        .with_context(|| format!("could not serialize `{r:?}`"))?;
                    res_sender
                        .send(res)
                        .with_context(|| format!("response channel closed"))?;
                    anyhow::Ok(())
                })
            })
            .buffered(1000)
            .map_err(|e| anyhow!("{e}"))
            .try_for_each(|r| async { r })
            .await?;
        Ok(())
    }};
}

pub async fn do_request(req: Request, res_sender: UnboundedSender<Vec<u8>>) -> Result<()> {
    match req.fun {
        Create {
            file_type,
            name,
            parent_id,
        } => res!(res_sender, req.id, File::new(&file_type, &name, &parent_id)),
        Get { id } => res!(res_sender, req.id, File::get(&id)),
        Rename { mut file, new_name } => res!(res_sender, req.id, file.rename(&new_name)),
        Delete { file } => res!(res_sender, req.id, file.delete()),
        Mime { file } => res!(res_sender, req.id, file.mime()),
        MoveToDir { mut file, dir_id } => res!(res_sender, req.id, file.move_to_dir(&dir_id)),
        List { file } => stream_res!(res_sender, req.id, file.list()),
        CopyToDir { file, dir_id } => stream_res!(res_sender, req.id, file.copy_to_dir(&dir_id)),
    }
}
