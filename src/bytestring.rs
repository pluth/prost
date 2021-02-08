use alloc::{string::String, vec::Vec};
use ops::Deref;
use core::{borrow, convert::TryFrom, fmt, hash, ops, str};

use bytes::Bytes;

/// An immutable UTF-8 encoded string with [`Bytes`] as a storage.
#[derive(Clone, Default, Eq, PartialOrd, Ord)]
pub struct ByteString(Bytes);

impl ByteString {
    /// Creates a new empty `ByteString`.
    pub const fn new() -> Self {
        ByteString(Bytes::new())
    }

    /// Get a reference to the underlying `Bytes` object.
    pub fn as_bytes(&self) -> &Bytes {
        &self.0
    }

    pub unsafe fn as_bytes_mut(&mut self) -> &mut Bytes{
        &mut self.0
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Unwraps this `ByteString` into the underlying `Bytes` object.
    pub fn into_bytes(self) -> Bytes {
        self.0
    }

    /// Creates a new `ByteString` from a `&'static str`.
    pub const fn from_static(src: &'static str) -> ByteString {
        Self(Bytes::from_static(src.as_bytes()))
    }

    pub fn as_str(&self) -> &str {
        self.deref()
    }

    /// Creates a new `ByteString` from a Bytes.
    ///
    /// # Safety
    /// This function is unsafe because it does not check the bytes passed to it are valid UTF-8.
    /// If this constraint is violated, it may cause memory unsafety issues with future users of
    /// the `ByteString`, as we assume that `ByteString`s are valid UTF-8. However, the most likely
    /// issue is that the data gets corrupted.
    pub const unsafe fn from_bytes_unchecked(src: Bytes) -> ByteString {
        Self(src)
    }
}

impl PartialEq<str> for ByteString {
    fn eq(&self, other: &str) -> bool {
        &self[..] == other
    }
}

impl<T: AsRef<str>> PartialEq<T> for ByteString {
    fn eq(&self, other: &T) -> bool {
        &self[..] == other.as_ref()
    }
}

impl AsRef<[u8]> for ByteString {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsRef<str> for ByteString {
    fn as_ref(&self) -> &str {
        &*self
    }
}

impl hash::Hash for ByteString {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl ops::Deref for ByteString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        let bytes = self.0.as_ref();
        // SAFETY:
        // UTF-8 validity is guaranteed at during construction.
        unsafe { str::from_utf8_unchecked(bytes) }
    }
}

impl borrow::Borrow<str> for ByteString {
    fn borrow(&self) -> &str {
        &*self
    }
}

impl From<String> for ByteString {
    #[inline]
    fn from(value: String) -> Self {
        Self(Bytes::from(value))
    }
}

impl From<&str> for ByteString {
    #[inline]
    fn from(value: &str) -> Self {
        Self(Bytes::copy_from_slice(value.as_ref()))
    }
}

impl TryFrom<&[u8]> for ByteString {
    type Error = str::Utf8Error;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let _ = str::from_utf8(value)?;
        Ok(ByteString(Bytes::copy_from_slice(value)))
    }
}

impl TryFrom<Vec<u8>> for ByteString {
    type Error = str::Utf8Error;

    #[inline]
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let buf = String::from_utf8(value).map_err(|err| err.utf8_error())?;
        Ok(ByteString(Bytes::from(buf)))
    }
}

impl TryFrom<Bytes> for ByteString {
    type Error = str::Utf8Error;

    #[inline]
    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        let _ = str::from_utf8(value.as_ref())?;
        Ok(ByteString(value))
    }
}

impl TryFrom<bytes::BytesMut> for ByteString {
    type Error = str::Utf8Error;

    #[inline]
    fn try_from(value: bytes::BytesMut) -> Result<Self, Self::Error> {
        let _ = str::from_utf8(&value)?;
        Ok(ByteString(value.freeze()))
    }
}

macro_rules! array_impls {
    ($($len:expr)+) => {
        $(
            impl TryFrom<[u8; $len]> for ByteString {
                type Error = str::Utf8Error;

                #[inline]
                fn try_from(value: [u8; $len]) -> Result<Self, Self::Error> {
                    ByteString::try_from(&value[..])
                }
            }

            impl TryFrom<&[u8; $len]> for ByteString {
                type Error = str::Utf8Error;

                #[inline]
                fn try_from(value: &[u8; $len]) -> Result<Self, Self::Error> {
                    ByteString::try_from(&value[..])
                }
            }
        )+
    }
}

array_impls!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32);

impl fmt::Debug for ByteString {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(fmt)
    }
}

impl fmt::Display for ByteString {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(fmt)
    }
}

#[cfg(feature = "serde")]
mod serde {
    use alloc::string::String;

    use serde::de::{Deserialize, Deserializer};
    use serde::ser::{Serialize, Serializer};

    use super::ByteString;

    impl Serialize for ByteString {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(self.as_ref())
        }
    }

    impl<'de> Deserialize<'de> for ByteString {
        #[inline]
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            String::deserialize(deserializer).map(ByteString::from)
        }
    }
}