use jsonrpc_core as jrpc;

pub fn to_rpc_err(e: anyhow::Error) -> jrpc::Error {
    jrpc::Error {
        code: jrpc::ErrorCode::ServerError(1),
        message: e.to_string(),
        data: Some(jrpc::Value::String(e.root_cause().to_string())),
    }
}

#[macro_export]
macro_rules! notify_err {
    ($sink: ident, $err: expr) => {{
        $sink
            .notify(Err($err))
            .map_err(|e| anyhow::anyhow!("Could not notify error '{}'", e))
    }};
}

#[macro_export]
macro_rules! notify_ok {
    ($sink: ident, $val: expr) => {{
        $sink
            .notify(Ok($val))
            .map_err(|e| anyhow::anyhow!("Could not notify result '{}'", e))
    }};
}

#[macro_export]
macro_rules! notify {
    ($sink: ident, $val: expr) => {
        match $val {
            Ok(v) => notify_ok!($sink, v),
            Err(e) => notify_err!($sink, e),
        }
    };
}
