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

use super::{
    Context, Result, ServerError,
    aio::{RWriter, TReader},
    message_handler,
};
use crate::{
    raw::{R, T, Version},
    server::{FileError, FileHandles, Filesystem, Requests},
};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;

struct ConnectionParams {
    msize: u32,
    version: Version,
}

async fn handshake(
    msize: u32,
    version: &Version,
    rw: &mut RWriter,
    tr: &mut TReader,
) -> Result<ConnectionParams> {
    loop {
        let t = tr.next().await?;
        let tag = t.tag();
        match t {
            T::Version(tag, client_msize, client_version) => {
                tracing::debug!("client version {client_msize} {client_version}");
                let conn_msize = msize.min(client_msize);

                match version.try_negotiate(&client_version) {
                    Ok(conn_version) => {
                        rw.set_msize(conn_msize);
                        tr.set_msize(conn_msize);

                        rw.send(R::Version(tag, conn_msize, conn_version.clone()))
                            .await?;

                        return Ok(ConnectionParams {
                            version: conn_version,
                            msize: conn_msize,
                        });
                    }
                    Err(e) => {
                        rw.send(R::Error(tag, format!("{e:?}"), 0xFFFFFFFF)).await?;
                        return Err(ServerError::FailedToNegotiate);
                    }
                };
            }
            _ => {
                tracing::warn!("dropping unexpected message during handshake (tag={tag})");
            }
        }
    }
}

/// Context about the connected session.
pub struct MessageContext<'a, FilesystemT>
where
    FilesystemT: Filesystem,
    FilesystemT: Send,
    FilesystemT: 'static,
{
    pub(super) peer: SocketAddr,
    pub(super) requests: &'a mut Requests,
    pub(super) handles: &'a mut FileHandles<FilesystemT::File>,
    pub(super) filesystems: Arc<Mutex<HashMap<String, FilesystemT>>>,
    pub(super) msize: u32,
}

/// Handler to manage the reading/writing of R/T messages, and dispatch
/// to internal methods after handshake, etc.
pub async fn connection_handler<FilesystemT>(
    ctx: Context<FilesystemT>,
    mut rw: RWriter,
    mut tr: TReader,
) -> Result<()>
where
    FilesystemT: Filesystem,
    FilesystemT: Send,
    FilesystemT: 'static,
{
    let Context {
        peer,
        msize,
        version,
        mut handles,
        mut requests,
        filesystems,
    } = ctx;

    let ConnectionParams { msize, version } = handshake(msize, &version, &mut rw, &mut tr).await?;

    tracing::info!("connection established with {peer}; version {version}, msize {msize}");

    loop {
        let t = tr.next().await?;
        let tag = t.tag();

        {
            match requests.insert(tag, t.clone()) {
                Ok(_) => {}
                Err(_) => {
                    // what do here? treat it as a flush on the old and send
                    // an error in reply to this?
                    continue;
                }
            };

            let mctx = MessageContext::<FilesystemT> {
                peer,
                requests: &mut requests,
                handles: &mut handles,
                filesystems: filesystems.clone(),
                msize,
            };
            let reply = match message_handler(mctx, t).await {
                Ok(r) => r,
                Err(err) => match err {
                    ServerError::FileError(FileError(errno, desc)) => R::Error(tag, desc, errno),
                    _ => R::Error(tag, format!("{err:?}"), 0xFFFFFFFF),
                },
            };

            tracing::debug!("reply tag={tag}: {:?}", reply);
            match requests.remove(tag) {
                Ok(_request) => {
                    rw.send(reply).await?;
                }
                Err(_) => {
                    tracing::trace!("reply tag={tag} not sent; was it flushed?");
                }
            }
        }
    }
}

// vim: foldmethod=marker
