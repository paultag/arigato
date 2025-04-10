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

use crate::{
    raw::{Fid, T, Tag},
    server::File,
};
use std::collections::HashMap;

/// Session being requested. This contains internal state about the connecting
/// user and filesystem requested.
#[derive(Debug, Clone)]
pub struct Session {
    pub(super) uname: String,
    pub(super) aname: String,
}

impl Session {
    /// Create a new Session.
    pub fn new(uname: String, aname: String) -> Self {
        Self { uname, aname }
    }
}

/// Handle to an open File of type FileT -- containing some additional
/// state if it exists (attached Session, any OpenFile type, etc).
#[derive(Clone)]
pub struct FileHandle<FileT>
where
    FileT: File,
    FileT: Send,
{
    pub(super) session: Session,
    pub(super) file: FileT,
    pub(super) of: Option<FileT::OpenFile>,
}

/// Map of all open Files (wrapped in their FileHandle) by file descriptor.
pub struct FileHandles<FileT>
where
    FileT: File,
    FileT: Send,
{
    handles: HashMap<Fid, FileHandle<FileT>>,
}

/// Errors which the FileHandles manager may return.
#[derive(Debug)]
pub enum FileHandlesError {
    /// File Descriptor already exists.
    FidAlreadyExists,

    /// No such file descriptor has been defined yet, or has been
    /// clunked.
    NoSuchFid,
}

impl<FileT> Default for FileHandles<FileT>
where
    FileT: File,
    FileT: Send,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<FileT> FileHandles<FileT>
where
    FileT: File,
    FileT: Send,
{
    /// Create a new FileHandles wrapper.
    pub fn new() -> Self {
        Self {
            handles: HashMap::new(),
        }
    }

    /// Add a new FileT, bound to the provided Session known by the
    /// provided file descriptor.
    pub fn insert(
        &mut self,
        fid: Fid,
        session: Session,
        file: FileT,
    ) -> Result<&FileHandle<FileT>, FileHandlesError> {
        let fh = FileHandle {
            session,
            file,
            of: None,
        };

        if self.handles.contains_key(&fid) {
            return Err(FileHandlesError::FidAlreadyExists);
        }
        self.handles.insert(fid, fh);

        Ok(self.handles.get(&fid).unwrap())
    }

    /// Remove the FileT, known by the provided file descriptor.
    pub fn remove(&mut self, fid: Fid) -> Result<FileHandle<FileT>, FileHandlesError> {
        match self.handles.remove(&fid) {
            Some(fh) => Ok(fh),
            None => Err(FileHandlesError::NoSuchFid),
        }
    }

    /// Get the FileT, known by the provided file descriptor.
    pub fn get(&self, fid: Fid) -> Result<&FileHandle<FileT>, FileHandlesError> {
        match self.handles.get(&fid) {
            Some(fh) => Ok(fh),
            None => Err(FileHandlesError::NoSuchFid),
        }
    }

    /// Get the FileT, known by the provided file descriptor.
    pub fn get_mut(&mut self, fid: Fid) -> Result<&mut FileHandle<FileT>, FileHandlesError> {
        match self.handles.get_mut(&fid) {
            Some(fh) => Ok(fh),
            None => Err(FileHandlesError::NoSuchFid),
        }
    }
}

/// Request type -- opaque handle containing a T type message.
pub struct Request {
    pub(super) t: T,
}

/// Possible Errors from the state code when resolving a tag during a session.
#[derive(Debug)]
pub enum RequestsError {
    /// That tag already exists and is still active.
    TagAlreadyExists,

    /// No such tag exists by that name anymore.
    NoSuchTag,
}

/// All pending requests known to the server.
pub struct Requests {
    requests: HashMap<Tag, Request>,
}

impl Default for Requests {
    fn default() -> Self {
        Self::new()
    }
}

impl Requests {
    /// Create a new Requests state tracking object.
    pub fn new() -> Self {
        Self {
            requests: HashMap::new(),
        }
    }

    /// Insert a new T message under the tag T.
    pub fn insert(&mut self, tag: Tag, t: T) -> Result<(), RequestsError> {
        if self.requests.contains_key(&tag) {
            return Err(RequestsError::TagAlreadyExists);
        }
        self.requests.insert(tag, Request { t });
        Ok(())
    }

    /// Remove the request known to us by the provided Tag.
    pub fn remove(&mut self, tag: Tag) -> Result<Request, RequestsError> {
        match self.requests.remove(&tag) {
            Some(req) => Ok(req),
            None => Err(RequestsError::NoSuchTag),
        }
    }

    /// Get the request known to us by the provided Tag.
    pub fn get(&self, tag: Tag) -> Result<&Request, RequestsError> {
        match self.requests.get(&tag) {
            Some(req) => Ok(req),
            None => Err(RequestsError::NoSuchTag),
        }
    }
}

// vim: foldmethod=marker
