use super::*;

pub fn encode<A, B>(tag: u32, value: &A, buf: &mut B)
where
    A: BytesAdapter,
    B: BufMut,
{
    encode_key(tag, WireType::LengthDelimited, buf);
    encode_varint(value.len() as u64, buf);
    value.append_to(buf);
}

pub fn merge<A, B>(
    wire_type: WireType,
    value: &mut A,
    buf: &mut B,
    _ctx: DecodeContext,
) -> Result<(), DecodeError>
where
    A: BytesAdapter,
    B: Buf,
{
    check_wire_type(WireType::LengthDelimited, wire_type)?;
    let len = decode_varint(buf)?;
    if len > buf.remaining() as u64 {
        return Err(DecodeError::new("buffer underflow"));
    }
    let len = len as usize;

    // Clear the existing value. This follows from the following rule in the encoding guide[1]:
    //
    // > Normally, an encoded message would never have more than one instance of a non-repeated
    // > field. However, parsers are expected to handle the case in which they do. For numeric
    // > types and strings, if the same field appears multiple times, the parser accepts the
    // > last value it sees.
    //
    // [1]: https://developers.google.com/protocol-buffers/docs/encoding#optional

    // NOTE: The use of BufExt::take() currently prevents zero-copy decoding
    // for bytes fields backed by Bytes when docoding from Bytes. This could
    // be addressed in the future by specialization.
    // See also: https://github.com/tokio-rs/bytes/issues/374
    value.replace_with(buf.take(len));
    Ok(())
}

length_delimited!(impl BytesAdapter);

#[cfg(test)]
mod test {
    use proptest::prelude::*;

    use super::super::test::{check_collection_type, check_type};
    use super::*;

    proptest! {
        #[test]
        fn check_vec(value: Vec<u8>, tag in MIN_TAG..=MAX_TAG) {
            super::test::check_type::<Vec<u8>, Vec<u8>>(value, tag, WireType::LengthDelimited,
                                                        encode, merge, encoded_len)?;
        }

        #[test]
        fn check_bytes(value: Vec<u8>, tag in MIN_TAG..=MAX_TAG) {
            let value = Bytes::from(value);
            super::test::check_type::<Bytes, Bytes>(value, tag, WireType::LengthDelimited,
                                                    encode, merge, encoded_len)?;
        }

        #[test]
        fn check_repeated_vec(value: Vec<Vec<u8>>, tag in MIN_TAG..=MAX_TAG) {
            super::test::check_collection_type(value, tag, WireType::LengthDelimited,
                                               encode_repeated, merge_repeated,
                                               encoded_len_repeated)?;
        }

        #[test]
        fn check_repeated_bytes(value: Vec<Vec<u8>>, tag in MIN_TAG..=MAX_TAG) {
            let value = value.into_iter().map(Bytes::from).collect();
            super::test::check_collection_type(value, tag, WireType::LengthDelimited,
                                               encode_repeated, merge_repeated,
                                               encoded_len_repeated)?;
        }
    }
}
