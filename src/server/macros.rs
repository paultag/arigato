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

///
#[macro_export]
macro_rules! gen_file_enum {
    (enum $name:ident {
        $( $child:ident($path:path) )+,
    }
    enum $open_file_name:ident
    ) => {
        /// Macro-generated wrapper type that contains multiple underlying
        /// Plan 9 File traited types.
        pub enum $name {
            $(
                $child($path),
            )+
        }

        /// Macro-generated wrapper type that contains multiple underlying
        /// Plan 9 OpenFile traited types.
        pub enum $open_file_name {
            $(
                $child(<$path as $crate::server::File>::OpenFile),
            )+
        }

        impl $crate::server::File for $name {
            type OpenFile = $open_file_name;

            async fn stat(&self) -> $crate::server::FileResult<$crate::raw::Stat> {
                match self {
                    $(
                        Self::$child(slf) => slf.stat().await
                    )+
                }
            }

            async fn wstat(
                &mut self,
                stat: &$crate::raw::Stat
            ) -> $crate::server::FileResult<()> {
                match self {
                    $(
                        Self::$child(slf) => slf.wstat(stat).await
                    )+
                }
            }

            async fn walk(&self, path: &[&str]) -> $crate::server::FileResult<(Option<Self>, Vec<Self>)> {
                match self {
                    $(
                        Self::$child(slf) => {
                            let (node, path) = slf.walk(path).await?;
                            Ok((
                                node.map(Self::$child),
                                path.into_iter().map(Self::$child).collect(),
                            ))
                        },
                    )+
                }
            }

            async fn unlink(&mut self) -> $crate::server::FileResult<()> {
                match self {
                    $(
                        Self::$child(slf) => slf.unlink().await
                    )+
                }
            }

            async fn create(
                &mut self,
                name: &str,
                perm: u16,
                ty: FileType,
                mode: OpenMode,
                extension: &str,
            ) -> $crate::server::FileResult<Self> {
                match self {
                    $(
                        Self::$child(slf) => Ok(Self::$child(slf.create(name, perm, ty, mode, extension).await?))
                    )+
                }
            }

            async fn open(&mut self, mode: OpenMode) -> $crate::server::FileResult<$open_file_name> {
                match self {
                    $(
                        Self::$child(slf) => Ok($open_file_name::$child(slf.open(mode).await?))
                    )+
                }
            }

            fn qid(&self) -> Qid {
                match self {
                    $(
                        Self::$child(slf) => slf.qid()
                    )+
                }
            }

        }

        impl $crate::server::OpenFile for $open_file_name {
           fn iounit(&self) -> u32 {
                match self {
                    $(
                        Self::$child(slf) => slf.iounit()
                    )+
                }
           }

           async fn read_at(&mut self, buf: &mut [u8], off: u64) -> $crate::server::FileResult<u32> {
                match self {
                    $(
                        Self::$child(slf) => slf.read_at(buf, off).await
                    )+
                }
           }

           async fn write_at(&mut self, buf: &mut [u8], off: u64) -> $crate::server::FileResult<u32> {
                match self {
                    $(
                        Self::$child(slf) => slf.write_at(buf, off).await
                    )+
                }
           }
        }

    };
}

// vim: foldmethod=marker
