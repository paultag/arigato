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

use super::{Dehydrate, Hydrate, StringError};
use std::{io::Cursor, str::FromStr};

/// Error decoding a Version
#[derive(Debug)]
pub enum VersionError {
    /// 9P version is mismatched.
    MismatchedId,

    /// 9P version variant is mismatched
    MismatchedVariant,

    /// Error turning bytes to unicode.
    StringError(StringError),
}

impl From<StringError> for VersionError {
    fn from(se: StringError) -> Self {
        Self::StringError(se)
    }
}

/// Version is the protocol level, which needs to be negotiated between client
/// and server.
#[derive(Debug, PartialEq, Clone)]
pub struct Version {
    /// Protocol version (9P2000)
    id: String,

    /// Variant (L, u, etc).
    variant: Option<String>,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.variant {
            Some(variant) => write!(f, "{}.{}", self.id, variant),
            None => write!(f, "{}", self.id),
        }
    }
}

impl FromStr for Version {
    type Err = VersionError;

    /// Create a new [Version] from a String.
    fn from_str(v: &str) -> Result<Version, VersionError> {
        // better validation logic here.
        match v.split_once('.') {
            Some((id, variant)) => Ok(Version {
                id: id.to_owned(),
                variant: Some(variant.to_owned()),
            }),
            None => Ok(Version {
                id: v.to_owned(),
                variant: None,
            }),
        }
    }
}

impl Version {
    /// try to negotiate with the peer on a 9p protocol.
    pub fn try_negotiate(&self, other: &Version) -> Result<Version, VersionError> {
        if self.id != other.id {
            return Err(VersionError::MismatchedId);
        }

        if self.variant == other.variant || self.variant.is_none() {
            return Ok(self.clone());
        }

        // TODO: behavior if we want 9P2000.FOO but the peer wants 9P2000;
        // we should negotiate down to 9P2000, but I don't think we actually
        // want to here? This should likely change.

        Err(VersionError::MismatchedVariant)
    }
}

impl<T> Hydrate<T> for Version
where
    T: AsRef<[u8]>,
{
    type Error = VersionError;

    fn hydrate(b: &mut Cursor<T>) -> Result<Self, Self::Error> {
        String::hydrate(b)?.parse()
    }
}

impl Dehydrate for Version {
    type Error = VersionError;

    fn dehydrate(&self, b: &mut Cursor<Vec<u8>>) -> Result<(), Self::Error> {
        let s = self.to_string();
        Ok(s.as_str().dehydrate(b)?)
    }
}

#[cfg(test)]
mod tests {
    use super::{Dehydrate, Hydrate, Version};
    use crate::raw::test_round_trip;
    use std::io::Cursor;

    #[test]
    fn parse() {
        let v: Version = "9P2000".parse().unwrap();
        assert_eq!(v.id, "9P2000");
        assert_eq!(v.variant, None);

        let v: Version = "9P2000.L".parse().unwrap();
        assert_eq!(v.id, "9P2000");
        assert_eq!(v.variant, Some("L".to_owned()));
    }

    #[test]
    fn negotiate_matched() {
        let v: Version = "9P2000".parse().unwrap();
        let v1: Version = "9P2000.L".parse().unwrap();

        assert_eq!(v.try_negotiate(&v).unwrap(), v, "9P2000 + 9P2000 = 9P2000");
        assert_eq!(
            v.try_negotiate(&v1).unwrap(),
            v,
            "9P2000 + 9P2000.L = 9P2000"
        );
        assert!(v1.try_negotiate(&v).is_err(), "9P2000.L + 9P2000 = Error");

        let v2: Version = "9P2001.L".parse().unwrap();
        assert!(v.try_negotiate(&v2).is_err());
        assert!(v1.try_negotiate(&v2).is_err());
        assert!(v2.try_negotiate(&v1).is_err());
        assert!(v2.try_negotiate(&v).is_err());
    }

    test_round_trip!(
        round_trip_version,
        Version,
        Version,
        ("9P2000".parse().unwrap(), "9P2000.L".parse().unwrap())
    );
}

// vim: foldmethod=marker
