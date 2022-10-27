mod api;
mod types;

use std::{
    io::{BufRead, Write},
    thread,
};

use anyhow::{anyhow, Result};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use types::*;

fn main() -> Result<()> {
    let (req_sender, req_receiver) = unbounded_channel::<Request>();
    let (res_sender, res_receiver) = unbounded_channel::<Vec<u8>>();

    let read_thread = thread::spawn(|| read_requests(req_sender));
    let write_thread = thread::spawn(|| write_responses(res_receiver));
    let req_handler_thread = thread::spawn(|| req_handler(req_receiver, res_sender));

    read_thread
        .join()
        .map_err(|e| anyhow!("could not join `read_thread` {e:?}"))??;
    write_thread
        .join()
        .map_err(|e| anyhow!("could not join `write_thread` {e:?}"))??;
    req_handler_thread
        .join()
        .map_err(|e| anyhow!("could not join `req_handler_thread` {e:?}"))??;

    Ok(())
}

fn req_handler(
    mut req_receiver: UnboundedReceiver<Request>,
    res_sender: UnboundedSender<Vec<u8>>,
) -> Result<()> {
    tokio::runtime::Runtime::new()?.block_on(async move {
        loop {
            let req = req_receiver
                .recv()
                .await
                .ok_or_else(|| anyhow!("request channel closed"))?;

            tokio::spawn(api::do_request(req, res_sender.clone()));
        }

        #[allow(unreachable_code)]
        anyhow::Ok(())
    })
}

fn read_requests(req_sender: UnboundedSender<Request>) -> Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut line = String::new();
    let mut line_reader = std::io::BufReader::new(stdin);
    let mut res_writer = std::io::BufWriter::new(stdout);

    loop {
        line_reader.read_line(&mut line)?;
        let req = match serde_json::from_str(line.as_str()) {
            Ok(req) => req,
            Err(e) => {
                let res = Response::<()>::new(0, Err(anyhow!("{e}")).into());
                let res = serde_json::to_vec(&res)?;
                res_writer.write_all(&res)?;
                res_writer.write_all(b"\n")?;
                res_writer.flush()?;
                continue;
            }
        };
        req_sender.send(req)?;

        line.clear();
    }
}

fn write_responses(mut res_receiver: UnboundedReceiver<Vec<u8>>) -> Result<()> {
    let stdout = std::io::stdout();
    let mut writer = std::io::BufWriter::new(stdout);

    loop {
        let res = res_receiver
            .blocking_recv()
            .ok_or_else(|| anyhow!("response channel closed"))?;

        writer.write_all(&res)?;
        writer.write_all(b"\n")?;

        writer.flush()?;
    }
}
