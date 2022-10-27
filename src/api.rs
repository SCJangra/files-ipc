use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use files::File;
use futures::{FutureExt, StreamExt, TryStreamExt};
use tokio::{
    sync::{mpsc::UnboundedSender, RwLock},
    task::JoinHandle,
};

use crate::types::*;
use Function::*;

type ReqHandel = JoinHandle<Result<()>>;

lazy_static::lazy_static! {
    pub static ref REQUESTS: RwLock<HashMap<usize, ReqHandel>> = RwLock::new(HashMap::<usize, ReqHandel>::new());
}

macro_rules! res {
    ($res_sender: expr, $id: expr, $fut: expr) => {{
        let res = $fut.map(|r| Response::new($id, r.into())).await;
        let res =
            serde_json::to_vec(&res).with_context(|| format!("could not serialize `{res:?}`"))?;
        $res_sender
            .send(res)
            .with_context(|| format!("response channel closed"))?;

        remove_req!($id);

        Ok(())
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

        let r = Response::new($id, RpcResult::Ok(serde_json::Value::Null));
        let v = serde_json::to_vec(&r).with_context(|| format!("could not serialize `{r:?}`"))?;
        $res_sender
            .send(v)
            .with_context(|| format!("response channel closed"))?;

        remove_req!($id);

        Ok(())
    }};
}

macro_rules! remove_req {
    ($id: expr) => {{
        REQUESTS.write().await.remove(&$id);
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
        Cancel { id } => cancel_request(req.id, id, res_sender).await,
    }
}

async fn cancel_request(
    req_id: usize,
    id: usize,
    res_sender: UnboundedSender<Vec<u8>>,
) -> Result<()> {
    if let Some(h) = REQUESTS.write().await.remove(&id) {
        h.abort();
    }
    let r = serde_json::to_vec(&Response::new(req_id, RpcResult::Ok(())))?;
    res_sender.send(r)?;
    Ok(())
}
