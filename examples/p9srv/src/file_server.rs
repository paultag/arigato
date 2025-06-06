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

use super::clean;
use arigato::{
    raw::{Dehydrate, FileType, IoDirection, OpenMode, Qid, Stat},
    server::{
        File as FileTrait, FileError, FileResult, Filesystem as FilesystemTrait,
        OpenFile as OpenFileTrait,
    },
};
use std::{
    fs::Metadata,
    io::{Cursor, Read, Seek, SeekFrom, Write},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    sync::Arc,
};

///
#[derive(Clone, Debug)]
pub struct FileServer {
    root: PathBuf,
    follow_symlinks: bool,
}

///
pub struct FileServerBuilder {
    root: PathBuf,
    follow_symlinks: bool,
}

impl FileServer {
    pub fn builder(root: &Path) -> FileServerBuilder {
        FileServerBuilder {
            root: root.to_owned(),
            follow_symlinks: false,
        }
    }
}

impl FileServerBuilder {
    pub fn follow_symlinks(mut self, follow: bool) -> Self {
        self.follow_symlinks = follow;
        self
    }

    pub fn build(self) -> FileServer {
        let Self {
            root,
            follow_symlinks,
        } = self;

        FileServer {
            root,
            follow_symlinks,
        }
    }
}

///
#[derive(Debug, Clone)]
pub struct File {
    path: PathBuf,
    qid: Qid,

    filesystem: Arc<FileServer>,
}

///
pub enum OpenFile {
    ///
    File(std::fs::File),

    ///
    Cursor(bool, std::io::Cursor<Vec<u8>>),
}

impl OpenFileTrait for OpenFile {
    fn iounit(&self) -> u32 {
        0
    }

    async fn read_at(&mut self, buf: &mut [u8], off: u64) -> FileResult<u32> {
        match self {
            Self::File(file) => {
                file.seek(SeekFrom::Start(off))?;
                Ok(file.read(buf)?.try_into().unwrap())
            }
            Self::Cursor(_, cur) => {
                cur.seek(SeekFrom::Start(off))?;
                Ok(cur.read(buf)?.try_into().unwrap())
            }
        }
    }

    async fn write_at(&mut self, buf: &mut [u8], off: u64) -> FileResult<u32> {
        match self {
            Self::File(file) => {
                file.seek(SeekFrom::Start(off))?;
                Ok(file.write(buf)?.try_into().unwrap())
            }
            Self::Cursor(ro, cur) => {
                if *ro {
                    Err(FileError(1, "EPERM".to_owned()))
                } else {
                    cur.seek(SeekFrom::Start(off))?;
                    Ok(cur.write(buf)?.try_into().unwrap())
                }
            }
        }
    }
}

impl File {
    /// Return a qid for a file off the filesystem metadata.
    fn qid_for_file(meta: &Metadata) -> Qid {
        let ty: FileType = meta.clone().into();
        Qid::new(ty, meta.mtime().try_into().unwrap_or(0), meta.ino())
    }

    /// Create a new File, which can be something like a directory, file, link
    /// or what have you.
    pub fn new(fs: Arc<FileServer>, path: &Path) -> Result<Self, FileError> {
        let path = clean(path);

        if !path.starts_with(&fs.root) {
            // not the right code, but for testing i needed something unique
            return Err(FileError(18, "EXDEV".to_owned()));
        }

        let meta = fs.meta(&path)?;
        let qid = Self::qid_for_file(&meta);
        Ok(Self {
            path: path.to_owned(),
            qid,
            filesystem: fs,
        })
    }

    async fn open_dir(&mut self, om: OpenMode) -> FileResult<OpenFile> {
        match om.direction() {
            IoDirection::Read => {}
            _ => return Err(FileError(1, "EPERM".to_owned())),
        }

        let mut ent = Cursor::new(vec![]);
        for dirent in std::fs::read_dir(&self.path)? {
            let stat = Self::new(self.filesystem.clone(), &dirent?.path())?
                .stat()
                .await?;
            match stat.dehydrate(&mut ent) {
                Ok(_) => {}
                Err(_) => return Err(FileError(22, "EINVAL".to_owned())),
            }
        }
        Ok(OpenFile::Cursor(true, ent))
    }

    async fn open_file(&mut self, om: OpenMode) -> FileResult<OpenFile> {
        match om.direction() {
            IoDirection::Read => {}
            _ => return Err(FileError(1, "EPERM".to_owned())),
        }

        Ok(OpenFile::File(std::fs::File::open(&self.path)?))
    }
}

impl FileServer {
    fn meta(&self, path: &Path) -> FileResult<Metadata> {
        Ok(if self.follow_symlinks {
            std::fs::metadata(path)?
        } else {
            std::fs::symlink_metadata(path)?
        })
    }
}

impl FileTrait for File {
    type OpenFile = OpenFile;

    async fn stat(&self) -> FileResult<Stat> {
        // use the cached qid here rather than reworking the qid based on
        // the filesystem again; this may be a mistake.

        let qid = self.qid.clone();
        let ty = qid.ty;

        let meta = self.filesystem.meta(&self.path)?;
        let mut sb = Stat::builder(
            self.path.file_name().and_then(|x| x.to_str()).unwrap_or(""),
            qid,
        )
        .with_mtime(meta.mtime().try_into().unwrap_or(0))
        .with_atime(meta.atime().try_into().unwrap_or(0))
        .with_mode(meta.mode())
        .with_nuid(meta.uid())
        .with_ngid(meta.gid())
        .with_nmuid(meta.uid())
        .with_size(meta.size());

        if ty == FileType::Link {
            sb = sb.with_extension(
                &std::fs::read_link(&self.path)?
                    .into_os_string()
                    .into_string()
                    // best I can do is EBADMSG here; not sure how else
                    // to spell "your fs is not unicode"
                    .map_err(|_| FileError(74, "EBADMSG".to_owned()))?,
            );
        }

        Ok(sb.build())
    }

    async fn wstat(&mut self, _s: &Stat) -> FileResult<()> {
        Err(FileError(1, "EPERM".to_owned()))
    }

    async fn walk(&self, path: &[&str]) -> FileResult<(Option<Self>, Vec<Self>)> {
        if path.is_empty() {
            return Ok((Some(self.clone()), vec![]));
        }

        let mut my_path = self.path.clone();

        let mut walked_path = vec![];
        for part in path {
            my_path.push(part);
            walked_path.push(match Self::new(self.filesystem.clone(), &my_path) {
                Ok(v) => v,
                Err(_) => {
                    return Ok((None, walked_path));
                }
            });
        }

        Ok((
            Self::new(self.filesystem.clone(), &my_path).ok(),
            walked_path,
        ))
    }

    async fn unlink(&mut self) -> FileResult<()> {
        Err(FileError(1, "EPERM".to_owned()))
    }

    async fn create(
        &mut self,
        _: &str,
        _: u16,
        _: FileType,
        _: OpenMode,
        _: &str,
    ) -> FileResult<Self> {
        Err(FileError(1, "EPERM".to_owned()))
    }

    async fn open(&mut self, om: OpenMode) -> FileResult<Self::OpenFile> {
        match self.qid.ty {
            FileType::File => self.open_file(om).await,
            FileType::Dir => self.open_dir(om).await,
            _ => Err(FileError(1, "EPERM".to_owned())),
        }
    }

    fn qid(&self) -> Qid {
        self.qid.clone()
    }
}

impl FilesystemTrait for FileServer {
    type File = File;

    async fn attach(&self, _: &str, _: &str, _: u32) -> FileResult<Self::File> {
        File::new(Arc::new(self.clone()), &self.root)
    }
}

// vim: foldmethod=marker
