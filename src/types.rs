use files::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub id: usize,
    pub fun: Function,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Function {
    Create {
        file_type: FileType,
        name: String,
        parent_id: FileId,
    },
    Get {
        id: FileId,
    },
    Rename {
        file: File,
        new_name: String,
    },
    Delete {
        file: File,
    },
    Mime {
        file: File,
    },
    MoveToDir {
        file: File,
        dir_id: FileId,
    },
    List {
        file: File,
    },
    CopyToDir {
        file: File,
        dir_id: FileId,
    },
}

#[derive(Debug, Serialize)]
pub struct Response<T: Serialize> {
    id: usize,
    res: RpcResult<T>,
}

#[derive(Debug, Serialize)]
pub enum RpcResult<T: Serialize> {
    Err(String),
    Ok(T),
}

impl<T: Serialize> Response<T> {
    pub fn new(id: usize, res: RpcResult<T>) -> Self {
        Self { id, res }
    }
}

impl<T: Serialize> From<anyhow::Result<T>> for RpcResult<T> {
    fn from(res: anyhow::Result<T>) -> Self {
        match res {
            Err(e) => RpcResult::Err(format!("{e:?}")),
            Ok(v) => RpcResult::Ok(v),
        }
    }
}
