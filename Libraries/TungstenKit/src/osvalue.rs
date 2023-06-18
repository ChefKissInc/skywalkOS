// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use alloc::{boxed::Box, string::String, vec::Vec};

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[repr(C)]
pub enum OSValue {
    Bool(bool),
    String(String),
    USize(usize),
    U64(u64),
    U32(u32),
    U16(u16),
    U8(u8),
    ISize(isize),
    I64(i64),
    I32(i32),
    I16(i16),
    I8(i8),
    Vec(Vec<Self>),
    Dictionary(HashMap<String, Self>),
    Tuple(Box<(Self, Self)>),
}

macro_rules! OSValueImplFor {
    ($variant:ident, $target:ty) => {
        impl From<$target> for OSValue {
            fn from(val: $target) -> Self {
                Self::$variant(val)
            }
        }

        impl TryFrom<OSValue> for $target {
            type Error = ();

            fn try_from(val: OSValue) -> Result<Self, Self::Error> {
                match val {
                    OSValue::$variant(d) => Ok(d),
                    _ => Err(()),
                }
            }
        }

        impl<'a> TryFrom<&'a OSValue> for &'a $target {
            type Error = ();

            fn try_from(val: &'a OSValue) -> Result<Self, Self::Error> {
                match val {
                    OSValue::$variant(d) => Ok(d),
                    _ => Err(()),
                }
            }
        }
    };
}

OSValueImplFor!(Bool, bool);
OSValueImplFor!(String, String);
impl From<&str> for OSValue {
    fn from(val: &str) -> Self {
        Self::String(val.into())
    }
}
impl<'a> TryFrom<&'a OSValue> for &'a str {
    type Error = ();

    fn try_from(val: &'a OSValue) -> Result<Self, Self::Error> {
        match val {
            OSValue::String(d) => Ok(d.as_str()),
            _ => Err(()),
        }
    }
}
OSValueImplFor!(USize, usize);
OSValueImplFor!(U64, u64);
OSValueImplFor!(U32, u32);
OSValueImplFor!(U16, u16);
OSValueImplFor!(U8, u8);
OSValueImplFor!(ISize, isize);
OSValueImplFor!(I64, i64);
OSValueImplFor!(I32, i32);
OSValueImplFor!(I16, i16);
OSValueImplFor!(I8, i8);
OSValueImplFor!(Vec, Vec<OSValue>);
OSValueImplFor!(Dictionary, HashMap<String, OSValue>);
impl<A: Into<Self>, B: Into<Self>> From<(A, B)> for OSValue {
    fn from(val: (A, B)) -> Self {
        Self::Tuple((val.0.into(), val.1.into()).into())
    }
}
impl<'a, A: TryFrom<&'a OSValue, Error = ()>, B: TryFrom<&'a OSValue, Error = ()>>
    TryFrom<&'a OSValue> for (A, B)
{
    type Error = ();

    fn try_from(val: &'a OSValue) -> Result<Self, Self::Error> {
        match val {
            OSValue::Tuple(v) => Ok(((&v.0).try_into()?, (&v.1).try_into()?)),
            _ => Err(()),
        }
    }
}
