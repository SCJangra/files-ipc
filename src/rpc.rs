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
}

pub struct RpcImpl {}

impl Rpc for RpcImpl {
    fn get_meta(&self, id: FileId) -> JrpcFutResult<FileMeta> {
        Box::pin(async move {
            let m = lib::get_meta(&id).map_err(to_rpc_err).await?;
            Ok(m)
        })
    }
}
