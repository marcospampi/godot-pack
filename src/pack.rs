use crate::pack;
use godot::classes::RefCounted;
use godot::prelude::*;
use std::panic;

#[derive(Debug, Clone)]
enum PackError {
    InvalidSequence,
    WriteConversionFailure,
}

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
enum ByteOrder {
    LittleEndian,
    BigEndian,
}

impl ByteOrder {
    #[cfg(target_endian = "little")]
    const NATIVE: ByteOrder = Self::LittleEndian;

    #[cfg(target_endian = "big")]
    const NATIVE: ByteOrder = Self::BigEndian;

    const NETWORK: ByteOrder = Self::BigEndian;
}

#[derive(Debug, Clone)]
struct FieldDescriptior {
    ty: FieldType,
    length: usize,
    offset: usize,
}

#[derive(Debug, Clone)]
struct PackingDescriptor {
    fields: Vec<FieldDescriptior>,
    size: usize,
    order: ByteOrder,
}

impl PackingDescriptor {
    pub(crate) fn sequence_from(seq: &str) -> Result<PackingDescriptor, PackError> {
        let mut order: ByteOrder = ByteOrder::NATIVE;
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
                        order = ByteOrder::NATIVE;
                    }
                    '<' => {
                        order = ByteOrder::LittleEndian;
                    }
                    '>' => {
                        order = ByteOrder::BigEndian;
                    }
                    '!' => {
                        order = ByteOrder::NETWORK;
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
                        return Err(PackError::InvalidSequence);
                    }
                }
                running_length = 0;
            }
        }

        Ok(PackingDescriptor {
            fields,
            size: offset,
            order,
        })
    }
}

#[derive(GodotClass, Debug)]
#[class(no_init,base=RefCounted)]
struct Pack {
    #[var]
    pub original: GString,

    pub(crate) descriptor: PackingDescriptor,
    base: Base<RefCounted>,
}

#[godot_api]
impl Pack {
    #[func]
    fn from(pack_string: GString) -> Gd<Self> {
        let descriptor = PackingDescriptor::sequence_from(&pack_string.to_string()).unwrap();
        Gd::from_init_fn(|base| Self {
            descriptor,
            original: pack_string,
            base,
        })
    }

    #[func]
    pub(crate) fn pack(&self, data: VariantArray) -> PackedByteArray {
        macro_rules! write_variant_as {
            ($variant:expr, $slice:expr, $bounds:expr, $endianess:expr, $T:ty) => {{
                if let Ok(value) = $variant.try_to_relaxed::<$T>() {
                    match $endianess {
                        ByteOrder::BigEndian => {
                            $slice[$bounds].copy_from_slice(&value.to_be_bytes());
                        }
                        ByteOrder::LittleEndian => {
                            $slice[$bounds].copy_from_slice(&value.to_le_bytes());
                        }
                    }
                }
            }};
        }
        godot_script_error!("{:#?}", self);
        let mut output = PackedByteArray::new();
        let endianess = self.descriptor.order.clone();
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
                        slice[bounds].copy_within(bytes);
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

        return output;
    }
    #[func]
    pub(crate) fn unpack(&self) -> VariantArray {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn pack() {
        let pack = Pack::from(GString::from("@fff"));
        let args = varray![1.0, 2.0, 3.0];
        {
            let binded = pack.bind();
            binded.pack(args);
        }
    }
}
