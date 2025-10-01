use godot::classes::RefCounted;
use godot::prelude::*;

#[derive(Debug, Clone)]
enum FieldType {
    String,
    Character,
    Bool,
    Char,
    UnsignedChar,
    Short,
    UnsignedShort,
    Int,
    UnsignedInt,
    Long,
    UnsignedLong,
    LongLong,
    UnsignedLongLong,
    Float,
    Double,
}
#[derive(Clone, Debug)]
enum Endianness {
    LittleEndian,
    BigEndian,
}

impl Endianness {
    #[cfg(target_endian = "little")]
    const NATIVE: Endianness = Self::LittleEndian;

    #[cfg(target_endian = "big")]
    const NATIVE: Endianness = Self::BigEndian;

    const NETWORK: Endianness = Self::BigEndian;
}

#[derive(Debug, Clone)]
struct FieldDescriptior {
    ty: FieldType,
    length: usize,
    offset: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct PackingDescriptor {
    fields: Vec<FieldDescriptior>,
    size: usize,
    endianness: Endianness,
}

impl PackingDescriptor {
    pub(crate) fn sequence_from(seq: &str) -> Result<PackingDescriptor, ()> {
        let mut order: Endianness = Endianness::NATIVE;
        let mut fields: Vec<FieldDescriptior> = vec![];

        let mut running_length: usize = 0;
        let mut offset: usize = 0;

        let mut post_increment = |count: usize| {
            let copy = offset.clone();
            offset = offset + count;
            return copy;
        };
        for c in seq.chars() {
            if c.is_digit(10) {
                running_length = running_length * 10 + c.to_digit(10).unwrap() as usize;
            } else {
                match c {
                    '@' | '=' => {
                        order = Endianness::NATIVE;
                    }
                    '<' => {
                        order = Endianness::LittleEndian;
                    }
                    '>' => {
                        order = Endianness::BigEndian;
                    }
                    '!' => {
                        order = Endianness::NETWORK;
                    }
                    's' => {
                        let length = running_length.clamp(1, u16::MAX as _) as _;
                        fields.push(FieldDescriptior {
                            ty: FieldType::String,
                            length,
                            offset: post_increment(length),
                        })
                    }

                    'x' => {
                        let length = running_length.clamp(1, u16::MAX as _) as _;
                        let _ = post_increment(length);
                    }
                    '?' => {
                        let length = core::mem::size_of::<bool>();
                        fields.push(FieldDescriptior {
                            ty: FieldType::Bool,
                            length: length,
                            offset: post_increment(length),
                        });
                    }
                    'c' => {
                        let length = core::mem::size_of::<u8>();
                        fields.push(FieldDescriptior {
                            ty: FieldType::Character,
                            length: length,
                            offset: post_increment(length),
                        });
                    }
                    'b' => {
                        let length = core::mem::size_of::<i8>();
                        fields.push(FieldDescriptior {
                            ty: FieldType::Char,
                            length: length,
                            offset: post_increment(length),
                        });
                    }
                    'B' => {
                        let length = core::mem::size_of::<u8>();
                        fields.push(FieldDescriptior {
                            ty: FieldType::UnsignedChar,
                            length: length,
                            offset: post_increment(length),
                        });
                    }
                    'h' => {
                        let length = core::mem::size_of::<i16>();
                        fields.push(FieldDescriptior {
                            ty: FieldType::Short,
                            length: length,
                            offset: post_increment(length),
                        });
                    }
                    'H' => {
                        let length = core::mem::size_of::<u16>();
                        fields.push(FieldDescriptior {
                            ty: FieldType::UnsignedShort,
                            length: length,
                            offset: post_increment(length),
                        });
                    }
                    'i' => {
                        let length = core::mem::size_of::<std::ffi::c_int>();
                        fields.push(FieldDescriptior {
                            ty: FieldType::Int,
                            length: length,
                            offset: post_increment(length),
                        });
                    }
                    'I' => {
                        let length = core::mem::size_of::<std::ffi::c_uint>();
                        fields.push(FieldDescriptior {
                            ty: FieldType::UnsignedInt,
                            length: length,
                            offset: post_increment(length),
                        });
                    }
                    'l' => {
                        let length = core::mem::size_of::<i32>();
                        fields.push(FieldDescriptior {
                            ty: FieldType::Long,
                            length: length,
                            offset: post_increment(length),
                        });
                    }
                    'L' => {
                        let length = core::mem::size_of::<u32>();
                        fields.push(FieldDescriptior {
                            ty: FieldType::UnsignedLong,
                            length: length,
                            offset: post_increment(length),
                        });
                    }
                    'q' => {
                        let length = core::mem::size_of::<i64>();
                        fields.push(FieldDescriptior {
                            ty: FieldType::LongLong,
                            length: length,
                            offset: post_increment(length),
                        });
                    }
                    'Q' => {
                        let length = core::mem::size_of::<u64>();
                        fields.push(FieldDescriptior {
                            ty: FieldType::UnsignedLongLong,
                            length: length,
                            offset: post_increment(length),
                        });
                    }
                    'f' => {
                        let length = core::mem::size_of::<f32>();
                        fields.push(FieldDescriptior {
                            ty: FieldType::Float,
                            length: length,
                            offset: post_increment(length),
                        });
                    }
                    'd' => {
                        let length = core::mem::size_of::<f64>();
                        fields.push(FieldDescriptior {
                            ty: FieldType::Double,
                            length: length,
                            offset: post_increment(length),
                        });
                    }
                    _ => {
                        godot_error!("Invalid pattern processed.");
                        return Err(());
                    }
                }
                running_length = 0;
            }
        }

        Ok(PackingDescriptor {
            fields,
            size: offset,
            endianness: order,
        })
    }
}

/// An helper object to pack and unpack binary data using a common format, much alike python's struct library.
/// Use `Pack.from(format)` to construct an instance, follows the format character table:
/// | Character | Meaning / Action                                          | Size (bytes)     |
/// | --------- | --------------------------------------------------------- | ---------------- |
/// | `@` / `=` | Set native endianness                                     | –                |
/// | `<`       | Set little-endian                                         | –                |
/// | `>`       | Set big-endian                                            | –                |
/// | `!`       | Set network endianness (big-endian)                       | –                |
/// | `...s`    | Preceded by `...` digits as length, a null terminated string          | ... or at least one byte |
/// | `...x`       | Preceded by `...` digits as length, padding space                     | ... or at least one byte |
/// | `?`       | Boolean                                                   | 1                |
/// | `c`       | Character (byte)                                          | 1                |
/// | `b`       | Signed 8-bit integer                                      | 1                |
/// | `B`       | Unsigned 8-bit integer                                    | 1                |
/// | `h`       | Signed 16-bit integer                                     | 2                |
/// | `H`       | Unsigned 16-bit integer                                   | 2                |
/// | `i`       | Signed 32-bit integer (`c_int`)                           | 4                |
/// | `I`       | Unsigned 32-bit integer (`c_uint`)                        | 4                |
/// | `l`       | Signed 32-bit integer (long)                              | 4                |
/// | `L`       | Unsigned 32-bit integer (long)                            | 4                |
/// | `q`       | Signed 64-bit integer (long long)                         | 8                |
/// | `Q`       | Unsigned 64-bit integer (long long)                       | 8                |
/// | `f`       | 32-bit floating point                                     | 4                |
/// | `d`       | 64-bit floating point                                     | 8                |
/// | *other*   | Invalid pattern (error)                                   | –                |

#[derive(GodotClass, Debug)]
#[class(no_init,base=RefCounted)]
pub struct Pack {
    #[var]
    pub original: GString,

    pub(crate) descriptor: PackingDescriptor,
    base: Base<RefCounted>,
}

#[godot_api]
impl Pack {
    /// Constructs an instance.
    #[func]
    pub fn from(format: GString) -> Option<Gd<Self>> {
        if let Ok(descriptor) = PackingDescriptor::sequence_from(&format.to_string()) {
            Some(Gd::from_init_fn(|base| Self {
                descriptor,
                original: format,
                base,
            }))
        } else {
            None
        }
    }
    /// Packs a variant array into either a `PackedByteArray` or `nil` if erroers.
    #[func]
    pub fn pack(&self, data: VariantArray) -> Variant {
        match self.pack_impl(data) {
            Ok(result) => Variant::from(result),
            Err(_) => return Variant::nil(),
        }
    }

    pub(crate) fn pack_impl(&self, data: VariantArray) -> Result<PackedByteArray, ()> {
        macro_rules! write_variant_as {
            ($variant:expr, $slice:expr, $bounds:expr, $endianess:expr, $T:ty) => {{
                if let Ok(value) = $variant.try_to_relaxed::<$T>() {
                    match $endianess {
                        Endianness::BigEndian => {
                            $slice[$bounds].copy_from_slice(&value.to_be_bytes());
                        }
                        Endianness::LittleEndian => {
                            $slice[$bounds].copy_from_slice(&value.to_le_bytes());
                        }
                    }
                } else {
                    return Err(());
                }
            }};
        }
        let mut output = PackedByteArray::new();
        let endianess = self.descriptor.endianness.clone();
        output.resize(self.descriptor.size);
        output.fill(0u8);
        {
            let slice = output.as_mut_slice();
            for (variant, descriptor) in data.iter_shared().zip(self.descriptor.fields.iter()) {
                let bounds = (descriptor.offset)..(descriptor.offset + descriptor.length);
                match descriptor.ty {
                    FieldType::String => {
                        let string = variant.to_string();
                        let bytes = string.as_bytes();
                        let min_size = usize::min(bytes.len(), descriptor.length);
                        slice[bounds][..min_size].copy_from_slice(&bytes[..min_size]);
                    }
                    FieldType::Character => {
                        let string = variant.to_string().as_bytes().first().cloned();
                        if let Some(first) = string {
                            slice[bounds].copy_from_slice(&[first]);
                        }
                    }
                    FieldType::Bool => {
                        if let Ok(value) = variant.try_to_relaxed::<bool>() {
                            slice[bounds].copy_from_slice(&[value as u8]);
                        }
                    }
                    FieldType::Char => {
                        write_variant_as!(variant, slice, bounds, endianess, i8);
                    }
                    FieldType::UnsignedChar => {
                        write_variant_as!(variant, slice, bounds, endianess, u8);
                    }
                    FieldType::Short => {
                        write_variant_as!(variant, slice, bounds, endianess, i16);
                    }
                    FieldType::UnsignedShort => {
                        write_variant_as!(variant, slice, bounds, endianess, u16);
                    }
                    FieldType::Long | FieldType::Int => {
                        write_variant_as!(variant, slice, bounds, endianess, i32);
                    }
                    FieldType::UnsignedInt | FieldType::UnsignedLong => {
                        write_variant_as!(variant, slice, bounds, endianess, u32);
                    }
                    FieldType::LongLong => {
                        write_variant_as!(variant, slice, bounds, endianess, i64);
                    }
                    FieldType::UnsignedLongLong => {
                        write_variant_as!(variant, slice, bounds, endianess, u64);
                    }
                    FieldType::Float => {
                        write_variant_as!(variant, slice, bounds, endianess, f32);
                    }
                    FieldType::Double => {
                        write_variant_as!(variant, slice, bounds, endianess, f64);
                    }
                }
            }
        }

        Ok(output)
    }
    /// Unpacks a `PackedByteArray` into either a `VariantArray` or `nil` if erroers.
    #[func]
    pub fn unpack(&self, data: PackedByteArray) -> Variant {
        match self.unpack_impl(data) {
            Ok(result) => result.to_variant(),
            Err(()) => Variant::nil(),
        }
    }
    pub(crate) fn unpack_impl(&self, data: PackedByteArray) -> Result<VariantArray, ()> {
        macro_rules! read_variant_from {
            ($result:expr, $data:expr, $bounds:expr, $endianness:expr, $T:ty) => {{
                let mut bytes = [0u8; core::mem::size_of::<$T>()];
                bytes.copy_from_slice(&$data[$bounds]);
                let extracted = match $endianness {
                    Endianness::BigEndian => <$T>::from_be_bytes(bytes),
                    Endianness::LittleEndian => <$T>::from_le_bytes(bytes),
                };
                $result.push(&extracted.to_variant());
            }};
        }
        if data.len() != self.descriptor.size {
            return Err(());
        }
        let data = data.as_slice();
        let mut result = VariantArray::new();
        let endianness = self.descriptor.endianness.clone();
        for field in &self.descriptor.fields {
            let bounds = (field.offset)..(field.offset + field.length);
            match field.ty {
                FieldType::String => {
                    let string = str::from_utf8(&data[bounds])
                        .map(|s| GString::from(s))
                        .unwrap();
                    result.push(&string.to_variant());
                }
                FieldType::Character => {
                    let value = data[field.offset];
                    let mut str = String::new();
                    str.push(char::from(value));
                    result.push(&str.to_variant());
                }
                FieldType::Bool => {
                    let value = data[field.offset] != 0;
                    result.push(&value.to_variant());
                }
                FieldType::Char => {
                    read_variant_from!(result, data, bounds, endianness, i8);
                }
                FieldType::UnsignedChar => {
                    read_variant_from!(result, data, bounds, endianness, u8);
                }
                FieldType::Short => {
                    read_variant_from!(result, data, bounds, endianness, i16);
                }
                FieldType::UnsignedShort => {
                    read_variant_from!(result, data, bounds, endianness, u16);
                }
                FieldType::Int | FieldType::Long => {
                    read_variant_from!(result, data, bounds, endianness, i32);
                }
                FieldType::UnsignedInt | FieldType::UnsignedLong => {
                    read_variant_from!(result, data, bounds, endianness, u32);
                }
                FieldType::LongLong => {
                    read_variant_from!(result, data, bounds, endianness, i64);
                }
                FieldType::UnsignedLongLong => {
                    read_variant_from!(result, data, bounds, endianness, u64);
                }
                FieldType::Float => {
                    read_variant_from!(result, data, bounds, endianness, f32);
                }
                FieldType::Double => {
                    read_variant_from!(result, data, bounds, endianness, f64);
                    //let mut bytes = [0u8; core::mem::size_of::<f64>()];
                    //bytes.copy_from_slice(&data[bounds]);
                    //let extracted = match endianness {
                    //    Endianness::BigEndian => f64::from_be_bytes(bytes),
                    //    Endianness::LittleEndian => f64::from_le_bytes(bytes),
                    //};
                    //result.push(&extracted.to_variant());
                }
            }
        }
        Ok(result)
    }
}
