#[cfg(any(feature = "scrypto_std", feature = "scrypto_alloc"))]
use scrypto_types::primitives::{Address, BID, H256, RID, U256};

use crate::model::*;
use crate::rust::boxed::Box;
use crate::rust::collections::*;
use crate::rust::string::String;
use crate::rust::vec;
use crate::rust::vec::Vec;

/// A data structure that can be described using SBOR types.
pub trait Describe {
    fn describe() -> Type;
}

macro_rules! describe_basic_type {
    ($type:ident, $sbor_type:expr) => {
        impl Describe for $type {
            fn describe() -> Type {
                $sbor_type
            }
        }
    };
}

describe_basic_type!(bool, Type::Bool);
describe_basic_type!(i8, Type::I8);
describe_basic_type!(i16, Type::I16);
describe_basic_type!(i32, Type::I32);
describe_basic_type!(i64, Type::I64);
describe_basic_type!(i128, Type::I128);
describe_basic_type!(u8, Type::U8);
describe_basic_type!(u16, Type::U16);
describe_basic_type!(u32, Type::U32);
describe_basic_type!(u64, Type::U64);
describe_basic_type!(u128, Type::U128);

describe_basic_type!(isize, Type::I32);
describe_basic_type!(usize, Type::U32);

describe_basic_type!(str, Type::String);
describe_basic_type!(String, Type::String);

#[cfg(any(feature = "scrypto_std", feature = "scrypto_alloc"))]
describe_basic_type!(H256, Type::H256);
#[cfg(any(feature = "scrypto_std", feature = "scrypto_alloc"))]
describe_basic_type!(U256, Type::U256);
#[cfg(any(feature = "scrypto_std", feature = "scrypto_alloc"))]
describe_basic_type!(Address, Type::Address);
#[cfg(any(feature = "scrypto_std", feature = "scrypto_alloc"))]
describe_basic_type!(BID, Type::BID);
#[cfg(any(feature = "scrypto_std", feature = "scrypto_alloc"))]
describe_basic_type!(RID, Type::RID);

impl<T: Describe> Describe for Option<T> {
    fn describe() -> Type {
        let ty = T::describe();
        Type::Option {
            value: Box::new(ty),
        }
    }
}

impl<T: Describe> Describe for Box<T> {
    fn describe() -> Type {
        let ty = T::describe();
        Type::Box {
            value: Box::new(ty),
        }
    }
}

impl<T: Describe, const N: usize> Describe for [T; N] {
    fn describe() -> Type {
        let ty = T::describe();
        Type::Array {
            element: Box::new(ty),
            length: N as u16,
        }
    }
}

macro_rules! describe_tuple {
    ($($name:ident)+) => {
        impl<$($name: Describe),+> Describe for ($($name,)+) {
            fn describe() -> Type {
                let mut elements = vec!();
                $(elements.push($name::describe());)+
                Type::Tuple { elements }
            }
        }
    };
}

describe_tuple! { A B }
describe_tuple! { A B C }
describe_tuple! { A B C D }
describe_tuple! { A B C D E }
describe_tuple! { A B C D E F }
describe_tuple! { A B C D E F G }
describe_tuple! { A B C D E F G H }
describe_tuple! { A B C D E F G H I }
describe_tuple! { A B C D E F G H I J }

impl<T: Describe> Describe for Vec<T> {
    fn describe() -> Type {
        let ty = T::describe();
        Type::Vec {
            element: Box::new(ty),
        }
    }
}

impl<T: Describe> Describe for BTreeSet<T> {
    fn describe() -> Type {
        let ty = T::describe();
        Type::TreeSet {
            element: Box::new(ty),
        }
    }
}

impl<K: Describe, V: Describe> Describe for BTreeMap<K, V> {
    fn describe() -> Type {
        let k = K::describe();
        let v = V::describe();
        Type::TreeMap {
            key: Box::new(k),
            value: Box::new(v),
        }
    }
}

impl<T: Describe> Describe for HashSet<T> {
    fn describe() -> Type {
        let ty = T::describe();
        Type::HashSet {
            element: Box::new(ty),
        }
    }
}

impl<K: Describe, V: Describe> Describe for HashMap<K, V> {
    fn describe() -> Type {
        let k = K::describe();
        let v = V::describe();
        Type::HashMap {
            key: Box::new(k),
            value: Box::new(v),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::rust::boxed::Box;
    use crate::rust::string::String;
    use crate::rust::vec;

    use crate::model::*;
    use crate::*;

    #[test]
    pub fn test_basic_types() {
        assert_eq!(Type::Bool, bool::describe());
        assert_eq!(Type::I8, i8::describe());
        assert_eq!(Type::I16, i16::describe());
        assert_eq!(Type::I32, i32::describe());
        assert_eq!(Type::I64, i64::describe());
        assert_eq!(Type::I128, i128::describe());
        assert_eq!(Type::U8, u8::describe());
        assert_eq!(Type::U16, u16::describe());
        assert_eq!(Type::U32, u32::describe());
        assert_eq!(Type::U64, u64::describe());
        assert_eq!(Type::U128, u128::describe());
        assert_eq!(Type::String, String::describe());
    }

    #[test]
    pub fn test_option() {
        assert_eq!(
            Type::Option {
                value: Box::new(Type::String)
            },
            Option::<String>::describe(),
        );
    }

    #[test]
    pub fn test_array() {
        assert_eq!(
            Type::Array {
                element: Box::new(Type::U8),
                length: 3,
            },
            <[u8; 3]>::describe(),
        );
    }

    #[test]
    pub fn test_tuple() {
        assert_eq!(
            Type::Tuple {
                elements: vec![Type::U8, Type::U128]
            },
            <(u8, u128)>::describe(),
        );
    }
}
