mod rpc;
mod utils;

use jsonrpc_core as jrpc;
use jsonrpc_stdio_server::ServerBuilder;
use rpc::Rpc;

#[tokio::main]
async fn main() {
    let mut io = jrpc::MetaIoHandler::<()>::default();
    let rpc = rpc::RpcImpl {};
    io.extend_with(rpc.to_delegate());

    let server = ServerBuilder::new(io).build();
    server.await;
}
