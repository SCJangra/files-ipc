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
    fn create_file(&self, name: String, dir: FileId) -> JrpcFutResult<FileId>;

    #[rpc(name = "create_dir")]
    fn create_dir(&self, name: String, dir: FileId) -> JrpcFutResult<FileId>;

    #[rpc(name = "delete_file")]
    fn delete_file(&self, id: FileId) -> JrpcFutResult<bool>;

    #[rpc(name = "delete_dir")]
    fn delete_dir(&self, id: FileId) -> JrpcFutResult<bool>;

    #[rpc(name = "rename")]
    fn rename(&self, id: FileId, new_name: String) -> JrpcFutResult<FileId>;

    #[rpc(name = "mv")]
    fn mv(&self, file: FileId, dest_dir: FileId) -> JrpcFutResult<FileId>;

    #[rpc(name = "get_mime")]
    fn get_mime(&self, file: FileId) -> JrpcFutResult<String>;

    #[pubsub(subscription = "rename_all", subscribe, name = "rename_all")]
    fn rename_all(
        &self,
        m: Self::Metadata,
        sub: pst::Subscriber<Option<Progress>>,
        rn: Vec<(FileMeta, String)>,
    );

    #[pubsub(subscription = "rename_all", unsubscribe, name = "rename_all_c")]
    fn rename_all_c(
        &self,
        m: Option<Self::Metadata>,
        id: ps::SubscriptionId,
    ) -> JrpcFutResult<bool>;

    #[pubsub(subscription = "copy_all", subscribe, name = "copy_all")]
    fn copy_all(
        &self,
        m: Self::Metadata,
        sub: pst::Subscriber<Option<CopyProg>>,
        files: Vec<FileMeta>,
        dst_dir: FileMeta,
        prog_interval: u128,
    );

    #[pubsub(subscription = "copy_all", unsubscribe, name = "copy_all_c")]
    fn copy_all_c(&self, m: Option<Self::Metadata>, id: ps::SubscriptionId) -> JrpcFutResult<bool>;

    #[pubsub(subscription = "mv_all", subscribe, name = "mv_all")]
    fn mv_all(
        &self,
        m: Self::Metadata,
        sub: pst::Subscriber<Option<Progress>>,
        files: Vec<FileMeta>,
        dst_dir: FileMeta,
    );

    #[pubsub(subscription = "mv_all", unsubscribe, name = "mv_all_c")]
    fn mv_all_c(&self, m: Option<Self::Metadata>, id: ps::SubscriptionId) -> JrpcFutResult<bool>;

    #[pubsub(subscription = "delete_all", subscribe, name = "delete_all")]
    fn delete_all(
        &self,
        m: Self::Metadata,
        sub: pst::Subscriber<Option<Progress>>,
        files: Vec<FileMeta>,
    );

    #[pubsub(subscription = "delete_all", unsubscribe, name = "delete_all_c")]
    fn delete_all_c(
        &self,
        m: Option<Self::Metadata>,
        id: ps::SubscriptionId,
    ) -> JrpcFutResult<bool>;
}

pub struct RpcImpl {}

impl Rpc for RpcImpl {
    type Metadata = std::sync::Arc<ps::Session>;

    fn get_meta(&self, id: FileId) -> JrpcFutResult<FileMeta> {
        Box::pin(async move { lib::get_meta(&id).map_err(to_rpc_err).await })
    }

    fn list_meta(&self, id: FileId) -> JrpcFutResult<Vec<FileMeta>> {
        Box::pin(async move { fun::list_meta(id).map_err(to_rpc_err).await })
    }

    fn create_file(&self, name: String, dir: FileId) -> JrpcFutResult<FileId> {
        Box::pin(async move { lib::create_file(&name, &dir).map_err(to_rpc_err).await })
    }

    fn create_dir(&self, name: String, dir: FileId) -> JrpcFutResult<FileId> {
        Box::pin(async move { lib::create_dir(&name, &dir).map_err(to_rpc_err).await })
    }

    fn delete_file(&self, id: FileId) -> JrpcFutResult<bool> {
        Box::pin(async move { lib::delete_file(&id).map_err(to_rpc_err).await })
    }

    fn delete_dir(&self, id: FileId) -> JrpcFutResult<bool> {
        Box::pin(async move { lib::delete_dir(&id).map_err(to_rpc_err).await })
    }

    fn rename(&self, id: FileId, new_name: String) -> JrpcFutResult<FileId> {
        Box::pin(async move { lib::rename(&id, &new_name).map_err(to_rpc_err).await })
    }

    fn mv(&self, file: FileId, dest_dir: FileId) -> JrpcFutResult<FileId> {
        Box::pin(async move { lib::mv(&file, &dest_dir).map_err(to_rpc_err).await })
    }

    fn get_mime(&self, file: FileId) -> JrpcFutResult<String> {
        Box::pin(async move { lib::get_mime(&file).map_err(to_rpc_err).await })
    }

    fn rename_all(
        &self,
        _m: Self::Metadata,
        sub: pst::Subscriber<Option<Progress>>,
        rn: Vec<(FileMeta, String)>,
    ) {
        task::spawn(fun::run(sub, move |sink| async move {
            let res = fun::rename_all(sink, rn).await;

            if let Err(_e) = res {
                // TODO: log this error
            }
        }));
    }

    fn rename_all_c(
        &self,
        _m: Option<Self::Metadata>,
        id: ps::SubscriptionId,
    ) -> JrpcFutResult<bool> {
        Box::pin(fun::sub_c(id))
    }

    fn copy_all(
        &self,
        _m: Self::Metadata,
        sub: pst::Subscriber<Option<CopyProg>>,
        files: Vec<FileMeta>,
        dst: FileMeta,
        prog_interval: u128,
    ) {
        task::spawn(fun::run(sub, move |sink| async move {
            let res = fun::copy_all(sink, files, dst, prog_interval).await;

            if let Err(_e) = res {
                // TODO: log this error
            }
        }));
    }

    fn copy_all_c(
        &self,
        _m: Option<Self::Metadata>,
        id: ps::SubscriptionId,
    ) -> JrpcFutResult<bool> {
        Box::pin(fun::sub_c(id))
    }

    fn mv_all(
        &self,
        _m: Self::Metadata,
        sub: pst::Subscriber<Option<Progress>>,
        files: Vec<FileMeta>,
        dst: FileMeta,
    ) {
        task::spawn(fun::run(sub, move |sink| async move {
            let res = fun::mv_all(sink, files, dst).await;

            if let Err(_e) = res {
                // TODO: log this error
            }
        }));
    }

    fn mv_all_c(&self, _m: Option<Self::Metadata>, id: ps::SubscriptionId) -> JrpcFutResult<bool> {
        Box::pin(fun::sub_c(id))
    }

    fn delete_all(
        &self,
        _m: Self::Metadata,
        sub: pst::Subscriber<Option<Progress>>,
        files: Vec<FileMeta>,
    ) {
        task::spawn(fun::run(sub, move |sink| async move {
            let res = fun::delete_all(sink, files).await;

            if let Err(_e) = res {
                // TODO: log this error
            }
        }));
    }

    fn delete_all_c(
        &self,
        _m: Option<Self::Metadata>,
        id: ps::SubscriptionId,
    ) -> JrpcFutResult<bool> {
        Box::pin(fun::sub_c(id))
    }
}
