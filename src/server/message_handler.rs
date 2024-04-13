// {{{ Copyright (c) Paul R. Tagliamonte <paultag@gmail.com>, 2023-2024
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE. }}}

use super::{MessageContext, Result};
use crate::{
    raw::{FileType, OpenMode, Qid, R, T},
    server::{File, Filesystem, OpenFile, ServerError, Session},
};

///
pub async fn message_handler<'a, FilesystemT>(
    mctx: MessageContext<'a, FilesystemT>,
    t: T,
) -> Result<R>
where
    FilesystemT: Filesystem,
    FilesystemT: Send,
    FilesystemT: 'static,
{
    let MessageContext {
        peer,
        msize,
        handles,
        requests,
        filesystems,
    } = mctx;

    match t {
        T::Version(tag, _, _) => {
            tracing::warn!(
                "Version message sent from {peer} after handshake; this ... is wrong? tag={tag}"
            );
            Ok(R::Error(tag, "EALREADY".to_owned(), 114))
        }
        T::Auth(tag, _, _, _, _) => {
            tracing::debug!("auth request (peer={peer}, tag={tag})");
            Ok(R::Error(tag, "ECONNREFUSED".to_owned(), 111))
        }
        T::Attach(tag, fid, _afid, uname, aname, nuname) => {
            tracing::debug!(
                "attach request (peer={peer}, tag={tag}, fid={fid}, uname={uname}, aname={aname}, nuname={nuname})"
            );

            let filesystems = filesystems.lock().await;
            let fs = match filesystems.get(&aname) {
                Some(fs) => fs,
                None => return Err(ServerError::NoSuchFilesystem),
            };
            let file = fs.attach(&uname, &aname, nuname).await?;
            let qid = file.qid();
            let session = Session::new(uname.clone(), aname.clone());
            handles.insert(fid, session, file)?;
            Ok(R::Attach(tag, qid))
        }
        T::Flush(tag, oldtag) => {
            tracing::debug!("flush request (peer={peer}, tag={tag}, oldtag={oldtag})");
            match requests.remove(oldtag) {
                Ok(req) => {
                    tracing::debug!(
                        "  flush (peer={peer}, tag={tag}, oldtag={oldtag}, t={:?})",
                        req.t
                    );
                }
                _ => {}
            }

            Ok(R::Flush(tag))
        }
        T::Walk(tag, fid, newfid, path) => {
            tracing::debug!("walk request (peer={peer}, tag={tag} from fid={fid}, store to newfid={newfid}, path={path:?})");
            {
                let handle = handles.get(fid)?;
                let session = handle.session.clone();

                tracing::trace!(
                    "walk request (peer={peer}, tag={tag}) session aname={}, uname={}",
                    session.aname,
                    session.uname
                );

                let path: Vec<&str> = path.iter().map(|x| x.as_ref()).collect();
                let (file, files) = handle.file.walk(path.as_slice()).await?;
                let qids: Vec<Qid> = files.iter().map(|x| x.qid()).collect();

                match file {
                    None => {
                        // failed to walk to the file
                        tracing::warn!(
                            "walk failed! file len={} path len={}",
                            files.len(),
                            path.len()
                        );

                        if files.len() == path.len() {
                            return Ok(R::Error(tag, "ENOENT".to_owned(), 2));
                        } else {
                            return Ok(R::Walk(tag, qids));
                        }
                    }
                    Some(file) => {
                        if files.len() != path.len() {
                            tracing::warn!("walk failed but was reported as a success!");
                            return Ok(R::Error(tag, "EINVAL".to_owned(), 22));
                        }
                        tracing::info!("target {:?} is now newfid {}", file.qid(), newfid);
                        handles.insert(newfid, session, file)?;
                    }
                }

                Ok(R::Walk(tag, qids))
            }
        }
        T::Open(tag, fid, mode) => {
            tracing::debug!("open request (peer={peer}, tag={tag}, fid={fid}, mode={mode:?})");
            let handle = handles.get_mut(fid)?;

            let file = &mut handle.file;
            let of = file.open(mode).await?;

            let iounit = of.iounit();
            let qid = file.qid();
            handle.of = Some(of);

            Ok(R::Open(tag, qid, iounit))
        }
        T::Create(tag, fid, name, perm, mode, extension) => {
            tracing::debug!("create request (peer={peer}, tag={tag}, fid={fid}, name={name})");

            let handle = handles.get_mut(fid)?;
            let file = &mut handle.file;

            let mode: OpenMode = mode.into();
            let ty: FileType = perm.into();
            let perm: u16 = (perm & 0o777) as u16;

            tracing::debug!("  tag={tag}, name={name}, ty={ty:?}, mode={mode:?}, perm={perm})");

            let mut f = file.create(&name, perm, ty, mode, &extension).await?;
            let of = f.open(mode).await?;
            handle.of = Some(of);

            Ok(R::Create(tag, f.qid(), 0))
        }
        T::Read(tag, fid, offset, size) => {
            tracing::debug!(
                "read request (peer={peer}, tag={tag}, fid={fid}, offset={offset}, size={size})"
            );
            let handle = handles.get_mut(fid)?;

            // msize here is wrong, buttttt, fine. This is just to cap
            // the upper bound not prevent errors from broken client
            // requests :)
            let mut buf = vec![0u8; size.min(msize) as usize];
            match &mut handle.of {
                Some(ref mut of) => {
                    let n = of.read_at(&mut buf, offset).await? as usize;
                    buf.resize(n, 0u8);
                    Ok(R::Read(tag, buf))
                }
                None => Ok(R::Error(tag, "EBADFD".to_owned(), 77)),
            }
        }
        T::Write(tag, fid, offset, mut buf) => {
            tracing::debug!(
                "write request (peer={peer}, tag={tag}, fid={fid}, offset={offset}, size={})",
                buf.len(),
            );
            let handle = handles.get_mut(fid)?;

            match &mut handle.of {
                Some(ref mut of) => {
                    let n = of.write_at(&mut buf, offset).await?;
                    Ok(R::Write(tag, n))
                }
                None => Ok(R::Error(tag, "EBADFD".to_owned(), 77)),
            }
        }
        T::Clunk(tag, fid) => {
            tracing::debug!("clunk request (peer={peer}, tag={tag}, fid={fid})");
            let _handle = handles.remove(fid)?;
            Ok(R::Clunk(tag))
        }
        T::Remove(tag, fid) => {
            tracing::debug!("remove request (peer={peer}, tag={tag}, fid={fid})");
            let mut handle = handles.remove(fid)?;
            handle.file.unlink().await?;
            Ok(R::Remove(tag))
        }
        T::Stat(tag, fid) => {
            tracing::debug!("stat request (peer={peer}, tag={tag}, fid={fid})");
            let handle = handles.get(fid)?;
            let stat = handle.file.stat().await?;
            Ok(R::Stat(tag, stat))
        }
        T::WStat(tag, fid, stat) => {
            tracing::debug!("wstat request (peer={peer}, tag={tag}, fid={fid}, stat={stat:?})");
            let handle = handles.get_mut(fid)?;
            handle.file.wstat(&stat).await?;
            Ok(R::WStat(tag))
        }
        T::Unknown(ty, tag, _) => {
            tracing::warn!("unknown message from {peer}; ty={ty}, tag={tag}");
            Ok(R::Error(tag, "ENOSYS".to_owned(), 38))
        }
    }
}

// vim: foldmethod=marker
