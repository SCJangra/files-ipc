use super::types::*;
use files::{fun as lib, types::*};
use futures::{self as futs, TryStreamExt};
use tokio::task;

pub async fn list_meta(id: FileId) -> AnyResult<Vec<FileMeta>> {
    let mut dirs = vec![];
    let mut files = vec![];

    lib::list_meta(&id)
        .await?
        .map_ok(|m| {
            match m.file_type {
                FileType::Dir => dirs.push(m),
                _ => files.push(m),
            };
        })
        .try_for_each(|_| async move { Ok(()) })
        .await?;

    let sort_dirs = task::spawn_blocking(move || {
        dirs.sort_by(|a, b| a.name.cmp(&b.name));
        dirs
    });

    let sort_files = task::spawn_blocking(move || {
        files.sort_by(|a, b| a.name.cmp(&b.name));
        files
    });

    let (mut d, f) = futs::try_join!(sort_dirs, sort_files)?;
    d.extend(f);

    Ok(d)
}
