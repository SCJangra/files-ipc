mod fun;
mod types;

use super::utils::*;
use types::*;

use files::{fun as lib, types::*};
use futures::TryFutureExt;
use jsonrpc_derive::rpc;
use jsonrpc_pubsub::{self as ps, typed as pst};
use tokio::task;

#[rpc(server)]
pub trait Rpc {
    type Metadata;

    #[rpc(name = "get_meta")]
    fn get_meta(&self, id: FileId) -> JrpcFutResult<FileMeta>;

    #[rpc(name = "list_meta")]
    fn list_meta(&self, id: FileId) -> JrpcFutResult<Vec<FileMeta>>;

    #[rpc(name = "create_file")]
    fn create_file(&self, name: String, dir: FileId) -> JrpcFutResult<FileMeta>;

    #[rpc(name = "create_dir")]
    fn create_dir(&self, name: String, dir: FileId) -> JrpcFutResult<FileMeta>;

    #[rpc(name = "delete_file")]
    fn delete_file(&self, id: FileId) -> JrpcFutResult<bool>;

    #[rpc(name = "delete_dir")]
    fn delete_dir(&self, id: FileId) -> JrpcFutResult<bool>;

    #[rpc(name = "rename")]
    fn rename(&self, id: FileId, new_name: String) -> JrpcFutResult<FileMeta>;

    #[rpc(name = "move")]
    fn move_file(&self, file: FileId, dest_dir: FileId) -> JrpcFutResult<FileMeta>;

    #[pubsub(subscription = "copy_file", subscribe, name = "copy_file")]
    fn copy_file(
        &self,
        m: Self::Metadata,
        sub: pst::Subscriber<Option<Progress>>,
        file: FileId,
        dst_dir: FileId,
        prog_interval: Option<u128>,
    );

    #[pubsub(subscription = "copy_file", unsubscribe, name = "copy_file_c")]
    fn copy_file_c(&self, m: Option<Self::Metadata>, id: ps::SubscriptionId)
        -> JrpcFutResult<bool>;
}

pub struct RpcImpl {}

impl Rpc for RpcImpl {
    type Metadata = std::sync::Arc<ps::Session>;

    fn get_meta(&self, id: FileId) -> JrpcFutResult<FileMeta> {
        Box::pin(async move {
            let m = lib::get_meta(&id).map_err(to_rpc_err).await?;
            Ok(m)
        })
    }

    fn list_meta(&self, id: FileId) -> JrpcFutResult<Vec<FileMeta>> {
        Box::pin(async move {
            let list = fun::list_meta(id).map_err(to_rpc_err).await?;
            Ok(list)
        })
    }

    fn create_file(&self, name: String, dir: FileId) -> JrpcFutResult<FileMeta> {
        Box::pin(async move {
            let m = lib::create_file(&name, &dir)
                .and_then(|id| async move { lib::get_meta(&id).await })
                .map_err(to_rpc_err)
                .await?;
            Ok(m)
        })
    }

    fn create_dir(&self, name: String, dir: FileId) -> JrpcFutResult<FileMeta> {
        Box::pin(async move {
            let m = lib::create_dir(&name, &dir)
                .and_then(|id| async move { lib::get_meta(&id).await })
                .map_err(to_rpc_err)
                .await?;
            Ok(m)
        })
    }

    fn delete_file(&self, id: FileId) -> JrpcFutResult<bool> {
        Box::pin(async move {
            let res = lib::delete_file(&id).map_err(to_rpc_err).await?;
            Ok(res)
        })
    }

    fn delete_dir(&self, id: FileId) -> JrpcFutResult<bool> {
        Box::pin(async move {
            let res = lib::delete_dir(&id).map_err(to_rpc_err).await?;
            Ok(res)
        })
    }

    fn rename(&self, id: FileId, new_name: String) -> JrpcFutResult<FileMeta> {
        Box::pin(async move {
            let m = lib::rename(&id, &new_name)
                .and_then(|id| async move { lib::get_meta(&id).await })
                .map_err(to_rpc_err)
                .await?;
            Ok(m)
        })
    }

    fn move_file(&self, file: FileId, dest_dir: FileId) -> JrpcFutResult<FileMeta> {
        Box::pin(async move {
            let m = lib::move_file(&file, &dest_dir)
                .and_then(|id| async move { lib::get_meta(&id).await })
                .map_err(to_rpc_err)
                .await?;
            Ok(m)
        })
    }

    fn copy_file(
        &self,
        _m: Self::Metadata,
        sub: pst::Subscriber<Option<Progress>>,
        file: FileId,
        dst_dir: FileId,
        prog_interval: Option<u128>,
    ) {
        task::spawn(fun::run(sub, move |sink| async move {
            let res = fun::copy_file(sink, file, dst_dir, prog_interval).await;

            if let Err(_e) = res {
                // TODO: log this error
            }
        }));
    }

    fn copy_file_c(
        &self,
        _m: Option<Self::Metadata>,
        id: ps::SubscriptionId,
    ) -> JrpcFutResult<bool> {
        Box::pin(fun::sub_c(id))
    }
}
