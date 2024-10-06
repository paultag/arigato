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

//! This module

mod aio;
mod async_server;
mod connection_handler;
mod macros;
mod message_handler;
mod state;
mod traits;

pub use aio::{RReader, RWriter, TReader, TWriter};
pub use traits::{File, FileError, FileResult, Filesystem, OpenFile};

use crate::raw::{RError, TError};

pub use async_server::{AsyncServer, AsyncServerBuilder, Context};
pub use connection_handler::{connection_handler, MessageContext};
pub use message_handler::message_handler;
pub use state::{
    FileHandle, FileHandles, FileHandlesError, Request, Requests, RequestsError, Session,
};

type JoinSet = tokio::task::JoinSet<()>;

type Result<RetT> = std::result::Result<RetT, ServerError>;

/// Possible Errors that may be returned.
#[derive(Debug)]
pub enum ServerError {
    /// Failed to come to an agreement with the client about the 9P
    /// protocol to use.
    FailedToNegotiate,

    /// No filesystem by that name is known by this server.
    NoSuchFilesystem,

    /// Something happened below us. Dunno! Good luck!
    IoError(std::io::Error),

    /// 9p T Error type
    TError(TError),

    /// 9p R Error type
    RError(RError),

    /// Error with state management of incoming and outgoing requests.
    RequestsError(RequestsError),

    /// Error with the file handles management.
    FileHandlesError(FileHandlesError),

    /// Error with an underlying File.
    FileError(FileError),
}

impl From<FileError> for ServerError {
    fn from(fe: FileError) -> Self {
        Self::FileError(fe)
    }
}

impl From<RequestsError> for ServerError {
    fn from(re: RequestsError) -> Self {
        Self::RequestsError(re)
    }
}

impl From<FileHandlesError> for ServerError {
    fn from(fhe: FileHandlesError) -> Self {
        Self::FileHandlesError(fhe)
    }
}

impl From<TError> for ServerError {
    fn from(te: TError) -> Self {
        match te {
            TError::IoError(ioe) => ioe.into(),
            _ => Self::TError(te),
        }
    }
}

impl From<RError> for ServerError {
    fn from(re: RError) -> Self {
        match re {
            RError::IoError(ioe) => ioe.into(),
            _ => Self::RError(re),
        }
    }
}

impl From<std::io::Error> for ServerError {
    fn from(ioe: std::io::Error) -> Self {
        Self::IoError(ioe)
    }
}

// vim: foldmethod=marker
