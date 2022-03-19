mod rpc;
mod utils;

use jsonrpc_core as jrpc;
use jsonrpc_ipc_server::{RequestContext, ServerBuilder};
use jsonrpc_pubsub::Session;

use rpc::Rpc;

fn main() {
    let mut io = jrpc::MetaIoHandler::default();
    let rpc = rpc::RpcImpl {};
    io.extend_with(rpc.to_delegate());

    let server_builder = ServerBuilder::with_meta_extractor(io, |request: &RequestContext| {
        std::sync::Arc::new(Session::new(request.sender.clone()))
    });
    let server = server_builder
        .start("/tmp/files")
        .expect("Unable to start TCP server");

    server.wait()
}
