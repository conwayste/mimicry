extern crate mimicry_derive;

pub use mimicry_arg::*;
pub use mimicry_derive::Mimic;

use std::{fmt::Debug, str::FromStr};

pub struct MimicFieldData {
    pub name: &'static str,                // the field name
    pub type_: &'static str,               // the filed type (ie: isize)
    pub type_arguments: Vec<&'static str>, // <A, B, C>
}

pub struct MimicMetadata {
    pub name: &'static str,         // variant name
    pub fields: Vec<MimicFieldData>, // variant fields
}

#[derive(Debug, PartialEq, Default)]
pub struct MimicList<T>
where
    T: FromStr,
{
    output_list: Vec<T>,
}

impl<T> MimicList<T>
where
    T: FromStr,
{
    pub fn new(t: Vec<T>) -> Self {
        MimicList::<T> { output_list: t }
    }
}

impl<T> From<MimicList<T>> for Vec<T>
where
    T: FromStr,
{
    fn from(ibl: MimicList<T>) -> Self {
        ibl.output_list
    }
}

impl<T> FromStr for MimicList<T>
where
    T: FromStr,
{
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let csv = s
            .split(";")
            .filter_map(|s: &str| {
                let t = s.trim();

                if !t.is_empty() {
                    Some(t)
                } else {
                    None
                }
            })
            .collect::<Vec<&str>>();
        if csv.is_empty() {
            return Err("No semi-colon-separated list found in input for MimicList");
        }

        let parsed = match csv
            .iter()
            .map(|item| item.parse::<T>())
            .collect::<Result<Vec<T>, <T as FromStr>::Err>>()
        {
            Ok(p) => p,
            Err(_) => return Err("Failed to parse as MimicList<T>"),
        };

        Ok(MimicList::<T>::new(parsed))
    }
}
