# A failed attempt at enforcing vec![T;N] compile-time length checks
```Rust
impl<R, S> TryFrom<&[&str]> for MimicArg2<R, S>
where
    S: FromStr,
    <S as FromStr>::Err: Debug,
    R: FromStr,
    <R as FromStr>::Err: Debug,
{
    type Error = TryFromSliceError;

    fn try_from(arr: &[&str]) -> Result<Self, Self::Error> {
        //<[&str; 3]>::try_from(s).map(Self::from)
        <&[&str; 2]>::try_from(arr).map(Self::from)
    }
}

impl<R, S> From<&[&str; 2]> for MimicArg2<R, S>
where
    S: FromStr,
    <S as FromStr>::Err: Debug,
    R: FromStr,
    <R as FromStr>::Err: Debug,
{
    fn from(arr: &[&str; 2]) -> Self {
        Self::from(*arr)
    }
}

impl<R, S> From<[&str; 2]> for MimicArg2<R, S>
where
    S: FromStr,
    <S as FromStr>::Err: Debug,
    R: FromStr,
    <R as FromStr>::Err: Debug,
{
    fn from(arr: [&str; 2]) -> Self {
        MimicArg2 {
            f0: arr[0].parse::<R>().unwrap(),
            //.map_err(|_| "Failed to parse second arg as R")?,
            f1: arr[1].parse::<S>().unwrap(),
            //.map_err(|_| "Failed to parse second arg as S")?,
        }
    }
}

#[cfg(tests)]
mod tests {
    #[test]
    fn make_mimicry_arg2_from_slice2() {
        let a = ["1", "2"];
        let b = MimicArg2::<isize, String>::try_from(a).unwrap();
        assert_eq!(b.f0, 1);
        assert_eq!(b.f1, String::from("2"));
    }

    #[test]
    fn make_mimicry_arg2_from_vec2toslice() {
        let a = vec!["1", "2"];
        let b = MimicArg2::<isize, String>::try_from(&a[0..2]).unwrap();
        assert_eq!(b.f0, 1);
        assert_eq!(b.f1, String::from("2"));
    }

    #[test]
    fn make_mimicry_arg2_from_vec3toslice() {
        // TODO (one day): Try to get this test case to fail compilation
        let a = vec!["1", "2", "2"];
        let b = MimicArg2::<isize, String>::try_from(&a[0..2]).unwrap();
        assert_eq!(b.f0, 1);
        assert_eq!(b.f1, String::from("2"));
    }
}
```