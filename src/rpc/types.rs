use jsonrpc_core::{BoxFuture, Result};

pub type JrpcFutResult<T> = BoxFuture<Result<T>>;
