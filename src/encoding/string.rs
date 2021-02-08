use crate::bytestring::ByteString;
use super::BytesAdapter;

use super::*;

pub trait StringAdapter: Default + Sized + 'static {
    type Bytes: BytesAdapter;
    unsafe fn bytes_mut(&mut self) -> &mut Self::Bytes;
    fn as_bytes(&self) -> &[u8];
    fn len(&self) -> usize;
    fn clear(&mut self);
}

impl StringAdapter for ByteString {
    type Bytes = ::bytes::Bytes;

    unsafe fn bytes_mut(&mut self) -> &mut Self::Bytes {
        self.as_bytes_mut()
    }

    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }

    fn len(&self) -> usize {
        self.bytes().len()
    }

    fn clear(&mut self) {
        self.clear()
    }
}

impl StringAdapter for String {
    type Bytes = Vec<u8>;

    unsafe fn bytes_mut(&mut self) -> &mut Self::Bytes {
        self.as_mut_vec()
    }

    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn clear(&mut self) {
        self.clear()
    }
}

pub fn encode<A, B>(tag: u32, value: &A, buf: &mut B)
where
    B: BufMut,
    A: StringAdapter,
{
    encode_key(tag, WireType::LengthDelimited, buf);
    encode_varint(value.len() as u64, buf);
    buf.put_slice(value.as_bytes());
}

pub fn merge<A, B>(
    wire_type: WireType,
    value: &mut A,
    buf: &mut B,
    ctx: DecodeContext,
) -> Result<(), DecodeError>
where
    B: Buf,
    A: StringAdapter
{
    // ## Unsafety
    //
    // `string::merge` reuses `bytes::merge`, with an additional check of utf-8
    // well-formedness. If the utf-8 is not well-formed, or if any other error occurs, then the
    // string is cleared, so as to avoid leaking a string field with invalid data.
    //
    // This implementation uses the unsafe `String::as_mut_vec` method instead of the safe
    // alternative of temporarily swapping an empty `String` into the field, because it results
    // in up to 10% better performance on the protobuf message decoding benchmarks.
    //
    // It's required when using `String::as_mut_vec` that invalid utf-8 data not be leaked into
    // the backing `String`. To enforce this, even in the event of a panic in `bytes::merge` or
    // in the buf implementation, a drop guard is used.
    unsafe {
        struct DropGuard<'a, A: StringAdapter>(&'a mut A);
        impl<'a, A: StringAdapter> Drop for DropGuard<'a, A> {
            #[inline]
            fn drop(&mut self) {
                self.0.clear();
            }
        }

        let drop_guard = DropGuard(value);
        bytes::merge(wire_type, drop_guard.0.bytes_mut(), buf, ctx)?;
        match str::from_utf8(drop_guard.0.as_bytes()) {
            Ok(_) => {
                // Success; do not clear the bytes.
                mem::forget(drop_guard);
                Ok(())
            }
            Err(_) => Err(DecodeError::new(
                "invalid string value: data is not UTF-8 encoded",
            )),
        }
    }
}

length_delimited!(impl StringAdapter);

#[cfg(test)]
mod test {
    use proptest::prelude::*;

    use super::super::test::{check_collection_type, check_type};
    use super::*;

    proptest! {
        #[test]
        fn check(value: String, tag in MIN_TAG..=MAX_TAG) {
            super::test::check_type::<String, String>(value, tag, WireType::LengthDelimited,
                                    encode, merge, encoded_len)?;
        }
        #[test]
        fn check_repeated(value: Vec<String>, tag in MIN_TAG..=MAX_TAG) {
            super::test::check_collection_type(value, tag, WireType::LengthDelimited,
                                               encode_repeated, merge_repeated,
                                               encoded_len_repeated)?;
        }
    }
}
