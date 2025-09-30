use crate::pack;
use godot::classes::RefCounted;
use godot::prelude::*;
use std::panic;

#[derive(Debug, Clone)]
enum PackError {
    InvalidSequence,
}

#[derive(Debug, Clone)]
enum Packing {
    String { length: usize },
    Pad { length: usize },
    Bool,
    Char,
    SignedChar,
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

impl Packing {
    fn sequence_from(seq: &str) -> Result<Vec<Packing>, PackError> {
        let mut result = vec![];
        let mut length = 0;
        for c in seq.chars() {
            if c.is_digit(10) {
                length = length * 10 + c.to_digit(10).unwrap() as usize;
            } else {
                match c {
                    's' => result.push(Packing::String {
                        length: length.clamp(1, u16::MAX as _),
                    }),
                    'x' => result.push(Packing::Pad {
                        length: length.clamp(1, u16::MAX as _),
                    }),
                    '?' => result.push(Packing::Bool),
                    'c' => result.push(Packing::Char),
                    'b' => result.push(Packing::SignedChar),
                    'B' => result.push(Packing::UnsignedChar),
                    'h' => result.push(Packing::Short),
                    'H' => result.push(Packing::UnsignedShort),
                    'i' => result.push(Packing::Int),
                    'I' => result.push(Packing::UnsignedInt),
                    'l' => result.push(Packing::Long),
                    'L' => result.push(Packing::UnsignedLong),
                    'q' => result.push(Packing::LongLong),
                    'Q' => result.push(Packing::UnsignedLongLong),
                    'f' => result.push(Packing::Float),
                    'd' => result.push(Packing::Double),
                    _ => {
                        return Err(PackError::InvalidSequence);
                    }
                }
                length = 0;
            }
        }

        Ok(result)
    }

    fn compute_size_from(seq: &[Packing]) -> usize {
        return seq.iter().fold(0, |acc, next| acc + next.size());
    }
    fn size(&self) -> usize {
        match self {
            Packing::String { length } => *length,
            Packing::Pad { length } => *length,
            Packing::Bool => core::mem::size_of::<bool>(),
            Packing::Char => core::mem::size_of::<u8>(),
            Packing::SignedChar => core::mem::size_of::<i8>(),
            Packing::UnsignedChar => core::mem::size_of::<u8>(),
            Packing::Short => core::mem::size_of::<i16>(),
            Packing::UnsignedShort => core::mem::size_of::<u16>(),
            Packing::Int => core::mem::size_of::<i32>(),
            Packing::UnsignedInt => core::mem::size_of::<u32>(),
            Packing::Long => core::mem::size_of::<i32>(),
            Packing::UnsignedLong => core::mem::size_of::<u32>(),
            Packing::LongLong => core::mem::size_of::<i64>(),
            Packing::UnsignedLongLong => core::mem::size_of::<u64>(),
            Packing::Float => core::mem::size_of::<f32>(),
            Packing::Double => core::mem::size_of::<f64>(),
        }
    }
}

#[derive(GodotClass)]
#[class(no_init,base=RefCounted)]
struct Pack {
    pack_string: GString,
    size: usize,
    seq: Vec<Packing>,
    base: Base<RefCounted>,
}

#[godot_api]
impl Pack {
    #[func]
    fn from(pack_string: GString) -> Gd<Self> {
        let packing_seq = Packing::sequence_from(&pack_string.to_string()).unwrap();
        let size = Packing::compute_size_from(&packing_seq);
        Gd::from_init_fn(|base| Self {
            pack_string: pack_string,
            size: size,
            seq: packing_seq,
            base,
        })
    }

    #[func]
    fn pack(&self, data: VariantArray) -> PackedByteArray {
        let mut vector = vec![0; self.size];

        let mut data_iterator = 0;

        let targets = self
            .seq
            .iter()
            .scan(0 as usize, |state, next| Some(*state + next.size()))
            .zip(self.seq.iter())
            .filter_map(|(offset, pack)| {
                if let Packing::Pad { length: _ } = pack {
                    return None;
                }
                return Some((offset, pack));
            })
            .collect::<Vec<(usize, Packing)>>();

        return PackedByteArray::from(vector);
    }
    #[func]
    fn unpack(&self) -> VariantArray {}
}
