use super::types::*;
use crate::{notify_err, notify_ok, utils::*};
use files::{fun as lib, fun_ext as lib_ext, types::*};
use futures::{self as futs, TryStreamExt};
use futures_async_stream::for_await;
use jsonrpc_core as jrpc;
use jsonrpc_pubsub::{self as ps, manager::IdProvider, typed as pst};
use std::collections as cl;
use tokio::{sync, task};
use unwrap_or::unwrap_ok_or;

pub async fn list_meta(id: FileId) -> AnyResult<Vec<FileMeta>> {
    let mut dirs = vec![];
    let mut files = vec![];

    lib::list_meta(&id)
        .await?
        .map_ok(|m| {
            match m.file_type {
                FileType::Dir => dirs.push(m),
                _ => files.push(m),
            };
        })
        .try_for_each(|_| async move { Ok(()) })
        .await?;

    let sort_dirs = task::spawn_blocking(move || {
        dirs.sort_by(|a, b| a.name.cmp(&b.name));
        dirs
    });

    let sort_files = task::spawn_blocking(move || {
        files.sort_by(|a, b| a.name.cmp(&b.name));
        files
    });

    let (mut d, f) = futs::try_join!(sort_dirs, sort_files)?;
    d.extend(f);

    Ok(d)
}

lazy_static::lazy_static! {
    pub static ref ACTIVE: sync::RwLock<cl::HashMap<ps::SubscriptionId, task::JoinHandle<()>>> =
        sync::RwLock::new(cl::HashMap::<
            ps::SubscriptionId,
            task::JoinHandle<()>
        >::new());
    static ref RAND_STR_ID: ps::manager::RandomStringIdProvider =
        ps::manager::RandomStringIdProvider::new();
}

pub async fn get_sink<T>(
    sub: pst::Subscriber<T>,
) -> anyhow::Result<(ps::SubscriptionId, pst::Sink<T>)> {
    let task_id = ps::SubscriptionId::String(RAND_STR_ID.next_id());
    let sink = sub
        .assign_id_async(task_id.clone())
        .await
        .map_err(|_| anyhow::anyhow!("Could not subscribe!"))?;

    Ok((task_id, sink))
}

pub async fn run<Fut, Fun, T>(sub: pst::Subscriber<T>, fun: Fun)
where
    Fut: futs::Future<Output = ()> + Send + 'static,
    Fun: FnOnce(pst::Sink<T>) -> Fut + Send + Sync + 'static,
    T: Send + 'static,
{
    let (sub_id, sink) = match get_sink(sub).await {
        Err(_e) => {
            /* TODO: Log this error */
            return;
        }
        Ok(v) => v,
    };

    ACTIVE.write().await.insert(
        sub_id.clone(),
        task::spawn(async move {
            fun(sink).await;

            {
                ACTIVE.write().await.remove(&sub_id);
            }
        }),
    );
}

pub async fn sub_c(id: ps::SubscriptionId) -> jrpc::Result<bool> {
    let removed = ACTIVE.write().await.remove(&id);
    if let Some(r) = removed {
        r.abort();
        Ok(true)
    } else {
        Err(jrpc::Error {
            code: jrpc::ErrorCode::InvalidParams,
            message: "Invalid subscription.".into(),
            data: None,
        })
    }
}

pub async fn copy(
    sink: pst::Sink<Option<CopyProg>>,
    files: Vec<FileMeta>,
    dst: FileMeta,
    prog_interval: u128,
) -> anyhow::Result<()> {
    #[for_await]
    for r in lib_ext::copy(&files[..], &dst, prog_interval) {
        let p = unwrap_ok_or!(r, e, {
            notify_err!(sink, to_rpc_err(e))?;
            break;
        });

        notify_ok!(sink, Some(p))?;
    }
    notify_ok!(sink, None)?;

    Ok(())
}

pub async fn mv(
    sink: pst::Sink<Option<Progress>>,
    files: Vec<FileMeta>,
    dst: FileMeta,
) -> anyhow::Result<()> {
    #[for_await]
    for r in lib_ext::mv(&files[..], &dst) {
        let p = unwrap_ok_or!(r, e, {
            notify_err!(sink, to_rpc_err(e))?;
            continue;
        });

        notify_ok!(sink, Some(p))?;
    }
    notify_ok!(sink, None)?;

    Ok(())
}

pub async fn delete(sink: pst::Sink<Option<Progress>>, files: Vec<FileMeta>) -> anyhow::Result<()> {
    #[for_await]
    for r in lib_ext::delete(&files[..]) {
        let p = unwrap_ok_or!(r, e, {
            notify_err!(sink, to_rpc_err(e))?;
            continue;
        });

        notify_ok!(sink, Some(p))?;
    }
    notify_ok!(sink, None)?;

    Ok(())
}
