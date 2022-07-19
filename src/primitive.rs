use std::fmt::{Debug, Display};

pub trait Primitive: 'static + Send + Sync + Sized + Debug + Display + PartialEq + Default + Clone {
    const TYPE: PrimitiveType;
}

macro_rules! native_type {
    ($type:ty, $data_type:expr) => {
        impl Primitive for $type {
            const TYPE: PrimitiveType = $data_type;
        }
    };
}

native_type!(bool, PrimitiveType::Bool);
native_type!(u8, PrimitiveType::U8);
native_type!(u16, PrimitiveType::U16);
native_type!(u32, PrimitiveType::U32);
native_type!(u64, PrimitiveType::U64);
native_type!(i8, PrimitiveType::I8);
native_type!(i16, PrimitiveType::I16);
native_type!(i32, PrimitiveType::I32);
native_type!(i64, PrimitiveType::I64);
native_type!(f32, PrimitiveType::F32);
native_type!(f64, PrimitiveType::F64);

#[derive(Debug, PartialEq, Clone)]
pub enum PrimitiveType {
    Bool,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
}
