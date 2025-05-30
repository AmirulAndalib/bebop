// TODO: Create an Array wrapper and a String wrapper to enable the user to use owned types or slices
// TODO: Test "unchecked" feature

use std::collections::HashMap;
use std::convert::TryInto;
use std::hash::Hash;
use std::io::Write;
use std::mem;

pub use error::*;
pub use fixed_sized::*;

use crate::{define_serialize_chained, test_serialization, Date, Guid, SliceWrapper};
// not sure why but this is "unused"
#[allow(unused_imports)]
use crate::collection;

pub mod alignment;
pub mod error;
pub mod fixed_sized;
pub mod testing;

pub type Len = u32;
/// Size of length data
pub const LEN_SIZE: usize = mem::size_of::<Len>();
/// Size of an enum
pub const ENUM_SIZE: usize = 4;

pub trait OwnedRecord: for<'raw> Record<'raw> {}
impl<T> OwnedRecord for T where T: for<'raw> Record<'raw> {}

pub trait OwnedSubRecord: for<'raw> SubRecord<'raw> {}
impl<T> OwnedSubRecord for T where T: for<'raw> SubRecord<'raw> {}

/// Bebop message type which can be serialized and deserialized.
pub trait Record<'raw>: SubRecord<'raw> {
    const OPCODE: Option<u32> = None;

    /// Deserialize this record
    #[inline(always)]
    fn deserialize(raw: &'raw [u8]) -> DeResult<Self> {
        Ok(Self::_deserialize_chained(raw)?.1)
    }

    /// Serialize this record. It is highly recommend to use a buffered writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use std::io::Write;
    /// # // Mock types for documentation example
    /// # struct MyFixedSizeType;
    /// # impl MyFixedSizeType {
    /// #     const SERIALIZED_SIZE: usize = 16;
    /// #     fn serialize<W: Write>(&self, dest: &mut W) -> Result<usize, std::io::Error> {
    /// #         Ok(16)
    /// #     }
    /// # }
    /// // For fixed-size types
    /// let value = MyFixedSizeType;
    /// let mut buf = Vec::with_capacity(MyFixedSizeType::SERIALIZED_SIZE);
    /// let bytes_written = value.serialize(&mut buf).unwrap();
    /// ```
    /// ```rust
    /// # use std::io::Write;
    /// # // Mock types for documentation example
    /// # struct MyVariableSizeType;
    /// # impl MyVariableSizeType {
    /// #     fn serialized_size(&self) -> usize { 32 }
    /// #     fn serialize<W: Write>(&self, dest: &mut W) -> Result<usize, std::io::Error> {
    /// #         Ok(32)
    /// #     }
    /// # }
    /// // For variable-size types
    /// let value = MyVariableSizeType;
    /// let size = value.serialized_size();
    /// let mut buf = Vec::with_capacity(size);
    /// let bytes_written = value.serialize(&mut buf).unwrap();
    /// ```
    #[inline(always)]
    fn serialize<W: Write>(&self, dest: &mut W) -> SeResult<usize> {
        Self::_serialize_chained(self, dest)
    }

    // TODO: support async serialization
    // fn serialize_async<W: AsyncWrite>(&self, dest: &mut W) -> impl Future<Type=SeResult<usize>>;
}

/// Internal trait used to reduce the amount of code that needs to be generated.
pub trait SubRecord<'raw>: Sized {
    const MIN_SERIALIZED_SIZE: usize;
    const EXACT_SERIALIZED_SIZE: Option<usize> = None;

    /// Exact size this will be once serialized in bytes.
    ///
    /// *Warning*: call is recursive and costly to make if not needed.
    fn serialized_size(&self) -> usize;

    /// Should only be called from generated code!
    /// Serialize this record. It is highly recommend to use a buffered writer.
    #[inline]
    fn _serialize_chained<W: Write>(&self, dest: &mut W) -> SeResult<usize> {
        unsafe { Self::_serialize_chained_unaligned(self, dest) }
    }

    // TODO: test performance of this versus a generic Write
    /// Should only be called from generated code!
    /// Serialize this record. It is highly recommend to use a buffered writer.
    ///
    /// This allows the value to be unaligned.
    ///
    /// # Safety
    /// This function assumes that `zelf` is a valid, readable, initialized pointer to a Self
    /// object. `zelf` does not need to be aligned.
    unsafe fn _serialize_chained_unaligned<W: Write>(
        zelf: *const Self,
        dest: &mut W,
    ) -> SeResult<usize>;

    /// Should only be called from generated code!
    /// Deserialize this object as a sub component of a larger message. Returns a tuple of
    /// (bytes_read, deserialized_value).
    fn _deserialize_chained(raw: &'raw [u8]) -> DeResult<(usize, Self)>;
}

impl<'raw> SubRecord<'raw> for &'raw str {
    const MIN_SERIALIZED_SIZE: usize = LEN_SIZE;

    #[inline]
    fn serialized_size(&self) -> usize {
        self.len() + LEN_SIZE
    }

    define_serialize_chained!(&str => |zelf, dest| serialize_byte_slice(dest, zelf.as_bytes()));

    fn _deserialize_chained(raw: &'raw [u8]) -> DeResult<(usize, Self)> {
        let len = read_len(raw)?;
        if len + LEN_SIZE > raw.len() {
            return Err(DeserializeError::MoreDataExpected(
                len + LEN_SIZE - raw.len(),
            ));
        }
        let raw_str = &raw[LEN_SIZE..len + LEN_SIZE];
        #[cfg(not(feature = "unchecked"))]
        {
            Ok((len + LEN_SIZE, std::str::from_utf8(raw_str)?))
        }
        #[cfg(feature = "unchecked")]
        unsafe {
            Ok((len + LEN_SIZE, std::str::from_utf8_unchecked(raw_str)))
        }
    }
}

#[test]
fn out_of_bounds_string() {
    let buf = [100, 100, 100, 100, 100, 100];
    let result = <&str>::_deserialize_chained(&buf);
    assert!(result.is_err());
}

impl<'raw> SubRecord<'raw> for String {
    const MIN_SERIALIZED_SIZE: usize = <&str>::MIN_SERIALIZED_SIZE;

    #[inline]
    fn serialized_size(&self) -> usize {
        self.len() + LEN_SIZE
    }

    define_serialize_chained!(String => |zelf, dest| serialize_byte_slice(dest, zelf.as_bytes()));

    #[inline]
    fn _deserialize_chained(raw: &'raw [u8]) -> DeResult<(usize, Self)> {
        <&str>::_deserialize_chained(raw).map(|(c, s)| (c, s.to_owned()))
    }
}

test_serialization!(serialization_str, &str, "some random string", 18 + LEN_SIZE);
test_serialization!(serialization_str_long, &str, "some random string that is a bit longer because I had seem some errors that seemed exclusive to longer string values.", 117 + LEN_SIZE);
test_serialization!(serialization_str_empty, &str, "", LEN_SIZE);

impl<'raw, T> SubRecord<'raw> for Vec<T>
where
    T: SubRecord<'raw>,
{
    const MIN_SERIALIZED_SIZE: usize = LEN_SIZE;

    #[inline]
    fn serialized_size(&self) -> usize {
        if let Some(size) = T::EXACT_SERIALIZED_SIZE {
            self.len() * size + LEN_SIZE
        } else {
            self.iter().fold(0, |acc, v| acc + v.serialized_size()) + LEN_SIZE
        }
    }

    define_serialize_chained!(Vec<T> => |zelf, dest| {
        write_len(dest, zelf.len())?;
        let mut i = LEN_SIZE;
        for v in zelf.iter() {
            i += v._serialize_chained(dest)?;
        }
        Ok(i)
    });

    fn _deserialize_chained(raw: &'raw [u8]) -> DeResult<(usize, Self)> {
        let len = read_len(raw)?;
        let mut i = LEN_SIZE;
        let mut v = Vec::with_capacity(len);
        for _ in 0..len {
            let slice = &raw.get(i..).ok_or(DeserializeError::CorruptFrame)?;
            let (read, t) = T::_deserialize_chained(slice)?;
            i += read;
            v.push(t);
        }
        Ok((i, v))
    }
}

test_serialization!(
    serialization_vec_str,
    Vec<&str>,
    vec!["abc", "def", "ghij"],
    10 + LEN_SIZE * 4
);
test_serialization!(
    serialization_vec_layered,
    Vec<Vec<Vec<i64>>>,
    (0..4)
        .map(|_| (0..4).map(|_| (0..16).collect()).collect())
        .collect(),
    8 * 4 * 4 * 16 + LEN_SIZE + LEN_SIZE * 4 + LEN_SIZE * 4 * 4
);
test_serialization!(serialization_vec_empty_str, Vec<&str>, Vec::new(), LEN_SIZE);
test_serialization!(
    serialization_vec_i16,
    Vec<i16>,
    vec![1234, 123, 154, -194, -4234, 432],
    12 + LEN_SIZE
);
test_serialization!(serialization_vec_empty_i16, Vec<i16>, Vec::new(), LEN_SIZE);

#[test]
fn out_of_bounds_array() {
    let buf = [100, 100, 100, 100, 100, 100];
    let result = Vec::<u32>::_deserialize_chained(&buf);
    assert!(result.is_err());
}

#[cfg(feature = "sorted_maps")]
pub trait SubRecordHashMapKey<'raw>: SubRecord<'raw> + Eq + Hash + Ord {}
#[cfg(not(feature = "sorted_maps"))]
pub trait SubRecordHashMapKey<'raw>: SubRecord<'raw> + Eq + Hash {}

#[cfg(feature = "sorted_maps")]
impl<'raw, T> SubRecordHashMapKey<'raw> for T where T: SubRecord<'raw> + Eq + Hash + Ord {}
#[cfg(not(feature = "sorted_maps"))]
impl<'raw, T> SubRecordHashMapKey<'raw> for T where T: SubRecord<'raw> + Eq + Hash {}

impl<'raw, K, V> SubRecord<'raw> for HashMap<K, V>
where
    K: SubRecordHashMapKey<'raw>,
    V: SubRecord<'raw>,
{
    const MIN_SERIALIZED_SIZE: usize = LEN_SIZE;

    #[inline]
    fn serialized_size(&self) -> usize {
        match (K::EXACT_SERIALIZED_SIZE, V::EXACT_SERIALIZED_SIZE) {
            (Some(k_size), Some(v_size)) => (k_size + v_size) * self.len() + LEN_SIZE,
            (Some(k_size), None) => {
                k_size * self.len()
                    + self.values().fold(0, |acc, v| acc + v.serialized_size())
                    + LEN_SIZE
            }
            (None, Some(v_size)) => {
                self.keys().fold(0, |acc, k| acc + k.serialized_size())
                    + v_size * self.len()
                    + LEN_SIZE
            }
            (None, None) => {
                self.keys().fold(0, |acc, k| acc + k.serialized_size())
                    + self.values().fold(0, |acc, v| acc + v.serialized_size())
                    + LEN_SIZE
            }
        }
    }

    define_serialize_chained!(HashMap<K, V> => |zelf, dest| {
        #[cfg(feature = "sorted_maps")]
        use itertools::Itertools as _;

        write_len(dest, zelf.len())?;
        let mut i = LEN_SIZE;

        #[cfg(feature = "sorted_maps")]
        let iter = zelf
            .iter()
            .sorted_unstable_by(|(k1, _), (k2, _)| k1.cmp(k2));
        #[cfg(not(feature = "sorted_maps"))]
        let iter = zelf.iter();

        for (k, v) in iter {
            i += k._serialize_chained(dest)?;
            i += v._serialize_chained(dest)?;
        }
        Ok(i)
    });

    fn _deserialize_chained(raw: &'raw [u8]) -> DeResult<(usize, Self)> {
        let len = read_len(raw)?;
        let mut i = LEN_SIZE;
        let mut m = HashMap::with_capacity(len);
        for _ in 0..len {
            let (read, k) =
                K::_deserialize_chained(raw.get(i..).ok_or(DeserializeError::CorruptFrame)?)?;
            i += read;
            let (read, v) =
                V::_deserialize_chained(raw.get(i..).ok_or(DeserializeError::CorruptFrame)?)?;
            i += read;
            m.insert(k, v);
        }
        Ok((i, m))
    }
}

test_serialization!(serialization_map_str_str, HashMap<&str, &str>, collection! { "k1" => "v1", "key2" => "value2" }, 14 + LEN_SIZE * 5);
test_serialization!(serialization_map_i16_str, HashMap<i16, &str>, collection! { 123 => "abc", -13 => "def", 843 => "ghij" }, 16 + LEN_SIZE * 4);
test_serialization!(serialization_map_str_i16, HashMap<&str, i16>, collection! { "abc" => 123, "def" => -13, "ghij" => 843 }, 16 + LEN_SIZE * 4);
test_serialization!(serialization_map_i16_i16, HashMap<i16, i16>, collection! { 23 => 432, -543 => 53, -43 => -12 }, 12 + LEN_SIZE);
test_serialization!(serialization_map_i16_i16_empty, HashMap<i16, i16>, HashMap::new(), LEN_SIZE);
test_serialization!(serialization_map_str_str_empty, HashMap<&str, &str>, HashMap::new(), LEN_SIZE);
test_serialization!(serialization_map_str_vec_empty_vec, HashMap<&str, Vec<i32>>, collection! {"abc" => vec![]}, LEN_SIZE * 3 + 3);

impl<'raw> SubRecord<'raw> for Guid {
    const MIN_SERIALIZED_SIZE: usize = Self::SERIALIZED_SIZE;
    const EXACT_SERIALIZED_SIZE: Option<usize> = Some(Self::SERIALIZED_SIZE);

    #[inline]
    fn serialized_size(&self) -> usize {
        Self::SERIALIZED_SIZE
    }

    define_serialize_chained!(*Guid => |zelf, dest| {
        dest.write_all(&zelf.to_ms_bytes())?;
        Ok(16)
    });

    #[inline]
    fn _deserialize_chained(raw: &'raw [u8]) -> DeResult<(usize, Self)> {
        if raw.len() < 16 {
            return Err(DeserializeError::MoreDataExpected(16 - raw.len()));
        }
        Ok((16, Guid::from_ms_bytes(raw[0..16].try_into().unwrap())))
    }
}

test_serialization!(
    serialization_guid,
    Guid,
    Guid::from_be_bytes([
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f
    ]),
    16
);

#[test]
fn out_of_bounds_guid() {
    let buf = [100, 100, 100, 100, 100, 100];
    let result = Guid::_deserialize_chained(&buf);
    assert!(result.is_err());
}

impl<'raw> SubRecord<'raw> for Date {
    const MIN_SERIALIZED_SIZE: usize = Self::SERIALIZED_SIZE;
    const EXACT_SERIALIZED_SIZE: Option<usize> = Some(Self::SERIALIZED_SIZE);

    #[inline]
    fn serialized_size(&self) -> usize {
        Self::SERIALIZED_SIZE
    }

    define_serialize_chained!(*Date => |zelf, dest| zelf.to_ticks()._serialize_chained(dest));

    #[inline]
    fn _deserialize_chained(raw: &'raw [u8]) -> DeResult<(usize, Self)> {
        let (read, date) = u64::_deserialize_chained(raw)?;
        Ok((read, Date::from_ticks(date)))
    }
}

test_serialization!(serialization_date, Date, Date::from_ticks(23462356), 8);

impl<'de> SubRecord<'de> for bool {
    const MIN_SERIALIZED_SIZE: usize = 1;
    const EXACT_SERIALIZED_SIZE: Option<usize> = Some(bool::SERIALIZED_SIZE);

    #[inline]
    fn serialized_size(&self) -> usize {
        bool::SERIALIZED_SIZE
    }

    define_serialize_chained!(*bool => |zelf, dest| {
        dest.write_all(&[if zelf { 1 } else { 0 }])?;
        Ok(1)
    });

    #[inline]
    fn _deserialize_chained(raw: &'de [u8]) -> DeResult<(usize, Self)> {
        if let Some(&b) = raw.first() {
            Ok((1, b > 0))
        } else {
            Err(DeserializeError::MoreDataExpected(1))
        }
    }
}

test_serialization!(serialization_bool_true, bool, true, 1);
test_serialization!(serialization_bool_false, bool, false, 1);

impl<'raw, T> SubRecord<'raw> for SliceWrapper<'raw, T>
where
    T: FixedSized + SubRecord<'raw>,
{
    const MIN_SERIALIZED_SIZE: usize = LEN_SIZE;

    #[inline]
    fn serialized_size(&self) -> usize {
        self.size() + LEN_SIZE
    }

    define_serialize_chained!(*SliceWrapper<'raw, T> => |zelf, dest| {
        write_len(dest, zelf.len())?;
        match zelf {
            SliceWrapper::Raw(raw) => {
                dest.write_all(raw)?;
                Ok(LEN_SIZE + raw.len())
            }
            SliceWrapper::Cooked(ary) => {
                if cfg!(target_endian = "little") &&
                    mem::size_of::<T>() % mem::align_of::<T>() == 0
                {
                    // special case with no padding and it is stored as little endian so we can
                    // treat it as a raw byte array
                    let b: &[u8] = unsafe {
                        std::slice::from_raw_parts(
                            ary.as_ptr() as *const u8,
                            ary.len() * mem::size_of::<T>(),
                        )
                    };
                    dest.write_all(b)?;
                    Ok(LEN_SIZE + b.len())
                } else {
                    // there is padding in the array so we can't just treat it as raw bytes
                    let mut i = LEN_SIZE;
                    for v in ary {
                        i += v._serialize_chained(dest)?;
                    }
                    Ok(i)
                }
            }
        }
    });

    fn _deserialize_chained(raw: &'raw [u8]) -> DeResult<(usize, Self)> {
        let len = read_len(raw)?;
        let bytes = len * mem::size_of::<T>() + LEN_SIZE;
        if bytes > raw.len() {
            return Err(DeserializeError::MoreDataExpected(bytes - raw.len()));
        }
        Ok((
            bytes,
            if mem::size_of::<T>() % mem::align_of::<T>() == 0
                && raw.as_ptr().align_offset(mem::align_of::<T>()) == 0
            {
                // if the size of T is evenly divisible by the alignment of T AND the start of the
                // array is aligned to T, the representation is already the same as if it were &[T].
                SliceWrapper::from_cooked(unsafe {
                    std::slice::from_raw_parts((raw[LEN_SIZE..bytes]).as_ptr() as *const T, len)
                })
            } else {
                SliceWrapper::from_raw(&raw[LEN_SIZE..bytes])
            },
        ))
    }
}

test_serialization!(
    serialization_slicewrapper_u8_cooked,
    SliceWrapper<u8>,
    SliceWrapper::Cooked(&[1, 2, 3, 4, 5, 6]),
    6 + LEN_SIZE
);

#[test]
fn out_of_bounds_slice_wrapper() {
    let buf = [100, 100, 100, 100, 100, 100];
    let result = SliceWrapper::<u8>::_deserialize_chained(&buf);
    assert!(result.is_err());
}

#[test]
fn serialization_slicewrapper_i16_cooked() {
    let mut buf = Vec::new();
    const LEN: usize = 10 + LEN_SIZE;
    let value: SliceWrapper<i16> = SliceWrapper::Cooked(&[12, 32, 543, 652, -23]);
    assert_eq!(value._serialize_chained(&mut buf).unwrap(), LEN);
    assert_eq!(buf.len(), LEN);
    buf.extend_from_slice(&[0x05, 0x01, 0x00, 0x00, 0x13, 0x42, 0x12]);
    let (read, deserialized) = <SliceWrapper<i16>>::_deserialize_chained(&buf).unwrap();
    assert_eq!(read, LEN);
    assert_eq!(
        deserialized,
        if cfg!(target_endian = "little") {
            SliceWrapper::Cooked(&[12, 32, 543, 652, -23])
        } else {
            SliceWrapper::Raw(&[12, 0, 32, 0, 31, 2, 140, 2, 233, 255])
        }
    );
}

// u8 is a special case among numbers because it has an alignment of 1.
impl<'raw> SubRecord<'raw> for u8 {
    const MIN_SERIALIZED_SIZE: usize = Self::SERIALIZED_SIZE;
    const EXACT_SERIALIZED_SIZE: Option<usize> = Some(Self::SERIALIZED_SIZE);

    #[inline]
    fn serialized_size(&self) -> usize {
        Self::SERIALIZED_SIZE
    }

    #[inline]
    unsafe fn _serialize_chained_unaligned<W: Write>(
        zelf: *const Self,
        dest: &mut W,
    ) -> SeResult<usize> {
        dest.write_all(&[*zelf])?;
        Ok(1)
    }

    #[inline]
    fn _deserialize_chained(raw: &'raw [u8]) -> DeResult<(usize, Self)> {
        if raw.is_empty() {
            Err(DeserializeError::MoreDataExpected(1))
        } else {
            Ok((1, raw[0]))
        }
    }
}

macro_rules! impl_record_for_num {
    ($t:ty) => {
        impl<'raw> SubRecord<'raw> for $t {
            const MIN_SERIALIZED_SIZE: usize = Self::SERIALIZED_SIZE;
            const EXACT_SERIALIZED_SIZE: Option<usize> = Some(Self::SERIALIZED_SIZE);

            #[inline]
            fn serialized_size(&self) -> usize {
                Self::SERIALIZED_SIZE
            }

            define_serialize_chained!(*$t => |zelf, dest| {
                dest.write_all(&zelf.to_le_bytes())?;
                Ok(mem::size_of::<$t>())
            });

            #[inline]
            fn _deserialize_chained(raw: &'raw [u8]) -> DeResult<(usize, Self)> {
                if raw.len() < mem::size_of::<$t>() {
                    return Err(DeserializeError::MoreDataExpected(
                        mem::size_of::<$t>() - raw.len(),
                    ));
                }
                Ok((
                    mem::size_of::<$t>(),
                    <$t>::from_le_bytes(raw[0..mem::size_of::<$t>()].try_into().unwrap()),
                ))
            }
        }
    };
}

// no signed byte type at this time
impl_record_for_num!(u16);
impl_record_for_num!(i16);
impl_record_for_num!(u32);
impl_record_for_num!(i32);
impl_record_for_num!(f32);
impl_record_for_num!(u64);
impl_record_for_num!(i64);
impl_record_for_num!(f64);

/// Read a 4-byte length value from the front of the raw data.
///
/// This should only be called from within an auto-implemented deserialize function or for byte
/// hacking.
#[inline(always)]
pub fn read_len(raw: &[u8]) -> DeResult<usize> {
    Ok(Len::_deserialize_chained(raw)?.1 as usize)
}

#[test]
fn read_len_test() {
    let buf = [23, 51, 0, 0, 2, 5];
    assert_eq!(read_len(&buf).unwrap(), 13079);
    assert_eq!(read_len(&buf[1..]).unwrap(), 33554483);
    assert_eq!(read_len(&buf[2..]).unwrap(), 84017152);
}

/// Write a 4-byte length value to the writer.
///
/// This should only be called from within an auto-implemented deserialize function or for byte
/// hacking.
#[inline(always)]
pub fn write_len<W: Write>(dest: &mut W, len: usize) -> SeResult<()> {
    if len > u32::MAX as usize {
        Err(SerializeError::LengthExceeds32Bits)
    } else {
        (len as u32)._serialize_chained(dest)?;
        Ok(())
    }
}

#[test]
fn write_len_test() {
    let mut buf = vec![0, 12, 4];
    write_len(&mut buf, 0).unwrap();
    write_len(&mut buf, 123).unwrap();
    write_len(&mut buf, 87543).unwrap();
    assert_eq!(buf.len(), 3 + LEN_SIZE * 3);
    assert_eq!(buf[3..7], [0, 0, 0, 0]);
    assert_eq!(buf[7..11], [123, 0, 0, 0]);
    assert_eq!(buf[11..], [247, 85, 1, 0]);
}

#[inline(always)]
fn serialize_byte_slice<W: Write>(dest: &mut W, raw: &[u8]) -> SeResult<usize> {
    write_len(dest, raw.len())?;
    dest.write_all(raw)?;
    Ok(LEN_SIZE + raw.len())
}
