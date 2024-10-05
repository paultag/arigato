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
    raw::{Fid, Tag, T},
    server::File,
};
use std::collections::HashMap;

///
#[derive(Debug, Clone)]
pub struct Session {
    pub(super) uname: String,
    pub(super) aname: String,
}

impl Session {
    ///
    pub fn new(uname: String, aname: String) -> Self {
        Self { uname, aname }
    }
}

///
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

///
pub struct FileHandles<FileT>
where
    FileT: File,
    FileT: Send,
{
    handles: HashMap<Fid, FileHandle<FileT>>,
}

///
#[derive(Debug)]
pub enum FileHandlesError {
    ///
    FidAlreadyExists,

    ///
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
    ///
    pub fn new() -> Self {
        Self {
            handles: HashMap::new(),
        }
    }

    ///
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

        if self.handles.get(&fid).is_some() {
            return Err(FileHandlesError::FidAlreadyExists);
        }
        self.handles.insert(fid, fh);

        Ok(self.handles.get(&fid).unwrap())
    }

    ///
    pub fn remove(&mut self, fid: Fid) -> Result<FileHandle<FileT>, FileHandlesError> {
        match self.handles.remove(&fid) {
            Some(fh) => Ok(fh),
            None => Err(FileHandlesError::NoSuchFid),
        }
    }

    ///
    pub fn get(&self, fid: Fid) -> Result<&FileHandle<FileT>, FileHandlesError> {
        match self.handles.get(&fid) {
            Some(fh) => Ok(fh),
            None => Err(FileHandlesError::NoSuchFid),
        }
    }

    ///
    pub fn get_mut(&mut self, fid: Fid) -> Result<&mut FileHandle<FileT>, FileHandlesError> {
        match self.handles.get_mut(&fid) {
            Some(fh) => Ok(fh),
            None => Err(FileHandlesError::NoSuchFid),
        }
    }
}

///
pub struct Request {
    pub(super) t: T,
}

///
#[derive(Debug)]
pub enum RequestsError {
    ///
    TagAlreadyExists,

    ///
    NoSuchTag,
}

///
pub struct Requests {
    requests: HashMap<Tag, Request>,
}

impl Default for Requests {
    fn default() -> Self {
        Self::new()
    }
}

impl Requests {
    ///
    pub fn new() -> Self {
        Self {
            requests: HashMap::new(),
        }
    }

    ///
    pub fn insert(&mut self, tag: Tag, t: T) -> Result<(), RequestsError> {
        if self.requests.get(&tag).is_some() {
            return Err(RequestsError::TagAlreadyExists);
        }
        self.requests.insert(tag, Request { t });
        Ok(())
    }

    ///
    pub fn remove(&mut self, tag: Tag) -> Result<Request, RequestsError> {
        match self.requests.remove(&tag) {
            Some(req) => Ok(req),
            None => Err(RequestsError::NoSuchTag),
        }
    }

    ///
    pub fn get(&self, tag: Tag) -> Result<&Request, RequestsError> {
        match self.requests.get(&tag) {
            Some(req) => Ok(req),
            None => Err(RequestsError::NoSuchTag),
        }
    }
}

// vim: foldmethod=marker
