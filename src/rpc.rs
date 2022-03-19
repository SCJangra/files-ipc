mod fun;
mod types;

use super::utils::*;
use types::*;

use files::{fun as lib, types::*};
use futures::TryFutureExt;
use jsonrpc_derive::rpc;

#[rpc(server)]
pub trait Rpc {
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
}

pub struct RpcImpl {}

impl Rpc for RpcImpl {
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

    fn create_file(&self, name: String, dir: FileId) -> JrpcFutResult<FileId> {
        Box::pin(async move {
            let id = lib::create_file(&name, &dir).map_err(to_rpc_err).await?;
            Ok(id)
        })
    }

    fn create_dir(&self, name: String, dir: FileId) -> JrpcFutResult<FileId> {
        Box::pin(async move {
            let id = lib::create_dir(&name, &dir).map_err(to_rpc_err).await?;
            Ok(id)
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
}
