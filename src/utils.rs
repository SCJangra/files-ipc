use jsonrpc_core as jrpc;

pub fn to_rpc_err(e: anyhow::Error) -> jrpc::Error {
    jrpc::Error {
        code: jrpc::ErrorCode::ServerError(1),
        message: e.to_string(),
        data: Some(jrpc::Value::String(e.root_cause().to_string())),
    }
}
