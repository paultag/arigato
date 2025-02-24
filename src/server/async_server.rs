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
    JoinSet, Result,
    aio::{RWriter, TReader},
    connection_handler,
};
use crate::{
    raw::Version,
    server::{FileHandles, Filesystem, Requests},
};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, sync::Mutex};

/// `tokio` async 9p server.
pub struct AsyncServer<FilesystemT>
where
    FilesystemT: Filesystem,
    FilesystemT: Send,
    FilesystemT: 'static,
{
    listener: TcpListener,
    msize: u32,

    filesystems: Arc<Mutex<HashMap<String, FilesystemT>>>,
}

/// Server context about the connected peer, instantiated Filesystem,
/// and active state (requests, file descriptors).
pub struct Context<FilesystemT>
where
    FilesystemT: Filesystem,
    FilesystemT: Send,
    FilesystemT: 'static,
{
    // pub(super) join_set: JoinSet,
    pub(super) msize: u32,
    pub(super) version: Version,
    pub(super) peer: SocketAddr,
    pub(super) handles: FileHandles<FilesystemT::File>,
    pub(super) requests: Requests,
    pub(super) filesystems: Arc<Mutex<HashMap<String, FilesystemT>>>,
}

impl<FilesystemT> AsyncServer<FilesystemT>
where
    FilesystemT: Filesystem,
    FilesystemT: Send,
    FilesystemT: 'static,
{
    /// Create a new [AsyncServerBuilder] to construct a new [AsyncServer].
    pub fn builder() -> AsyncServerBuilder<FilesystemT> {
        AsyncServerBuilder::new()
    }

    /// Listen on the configured port, and serve 9p requests.
    pub async fn serve(&self) -> Result<()> {
        let mut join_set = JoinSet::new();

        loop {
            match self.listener.accept().await {
                Ok((socket, addr)) => {
                    socket.set_nodelay(true)?;
                    tracing::info!("new connection: {:?}", addr);
                    let (read, write) = socket.into_split();
                    let tr = TReader::new(Box::pin(read), self.msize);
                    let rw = RWriter::new(Box::pin(write), self.msize);
                    let ctx = Context {
                        // join_set: JoinSet::new(),
                        peer: addr,
                        version: "9P2000.u".parse().unwrap(),
                        msize: self.msize,
                        handles: FileHandles::<FilesystemT::File>::new(),
                        requests: Requests::new(),
                        filesystems: self.filesystems.clone(),
                    };

                    let _ = join_set
                        .build_task()
                        .name(&format!("connection [{addr}]"))
                        .spawn(async move {
                            tracing::debug!("task started [{addr}]");
                            let tr = tr;
                            let rw = rw;
                            if let Err(e) = connection_handler(ctx, rw, tr).await {
                                tracing::warn!("task [{addr}] failed with {e:?}");
                            }
                        });
                }
                Err(e) => {
                    tracing::warn!("failed to establish: {}", e);
                    return Err(e.into());
                }
            }
        }
    }
}

/// Builder-pattern struct to create an [AsyncServer].
pub struct AsyncServerBuilder<FilesystemT>
where
    FilesystemT: Filesystem,
    FilesystemT: Send,
    FilesystemT: 'static,
{
    tcp_listen_address: Option<String>,
    msize: Option<u32>,
    filesystems: HashMap<String, FilesystemT>,
}

impl<FilesystemT> AsyncServerBuilder<FilesystemT>
where
    FilesystemT: Filesystem,
    FilesystemT: Send,
    FilesystemT: 'static,
{
    /// Create a new Builder-pattern struct to create an [AsyncServer].
    fn new() -> Self {
        Self {
            filesystems: HashMap::new(),
            msize: None,
            tcp_listen_address: None,
        }
    }

    /// Set the configured 9p msize (maximum size, in bytes, to use
    /// for a single packet).
    pub fn with_msize(mut self, msize: u32) -> Self {
        self.msize = Some(msize);
        self
    }

    /// Set the IP address and port to listen on.
    pub fn with_tcp_listen_address(mut self, addr: &str) -> Self {
        self.tcp_listen_address = Some(addr.to_owned());
        self
    }

    /// Use the provided Filesystem for the specified filesystem name
    /// (aname).
    pub fn with_filesystem(mut self, name: &str, fs: FilesystemT) -> Self {
        self.filesystems.insert(name.to_owned(), fs);
        self
    }

    /// Build an [AsyncServer].
    pub async fn build(self) -> Result<AsyncServer<FilesystemT>> {
        let listen_address = self.tcp_listen_address.unwrap();
        let listener = TcpListener::bind(listen_address).await?;

        Ok(AsyncServer {
            listener,
            msize: self.msize.unwrap_or(0xFFFFFF00),
            filesystems: Arc::new(Mutex::new(self.filesystems)),
        })
    }
}

// vim: foldmethod=marker
