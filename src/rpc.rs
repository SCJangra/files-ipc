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
}
