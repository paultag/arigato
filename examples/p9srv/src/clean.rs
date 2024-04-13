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

use std::path::{Component, Path, PathBuf};

pub fn clean(path: &Path) -> PathBuf {
    let mut r = Vec::new();
    for c in path.components() {
        match c {
            Component::ParentDir => match r.last() {
                Some(Component::Normal(_)) => {
                    r.pop();
                }
                None | Some(Component::CurDir) | Some(Component::ParentDir) => r.push(c),
                Some(Component::RootDir) => (),
                Some(Component::Prefix(_)) => {
                    // windows, sigh
                    unreachable!();
                }
            },
            Component::CurDir => (),
            c => r.push(c),
        }
    }
    r.iter().collect()
}

#[cfg(test)]
mod test {
    use super::clean;
    use std::path::PathBuf;

    #[test]
    fn clean_paths() {
        for (given, expected) in [
            ("/foo", "/foo"),
            ("/foo/../bar", "/bar"),
            ("/foo/../../", "/"),
            ("foo/../../", ".."),
            ("foo/../bar/", "bar"),
            ("/foo///bar/", "/foo/bar"),
        ] {
            let given: PathBuf = given.parse().unwrap();
            let expected: PathBuf = expected.parse().unwrap();
            let generated = clean(&given);
            assert_eq!(expected, generated);
        }
    }
}

// vim: foldmethod=marker
