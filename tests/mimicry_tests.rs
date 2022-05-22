extern crate mimicry;

pub use mimicry::*;

use std::{fmt::Debug, str::FromStr, vec};

#[allow(unused)]
#[derive(Debug, Mimic)]
enum RequestAction {
    None,
    Connect {
        name: String,
        client_version: String,
    },
    Disconnect,
    KeepAlive {
        latest_response_ack: u64,
    },
}

#[test]
fn make_requestaction_connect() {
    let responses = vec!["uno".to_owned(), "1.5.2".to_owned()];
    let rac_mimic = MimicRequestAction::RequestActionConnect {
        inner: RequestActionConnect::try_from(responses).expect("Failed to parse"),
    };

    if let MimicRequestAction::RequestActionConnect { inner } = rac_mimic {
        let _ = RequestAction::Connect {
            name: inner.instance.f0,
            client_version: inner.instance.f1,
        };
    };
}

#[test]
fn make_requestaction_keepalive() {
    let responses = vec!["12763917391823".to_owned()];
    let raka_mimic = MimicRequestAction::RequestActionKeepAlive {
        inner: RequestActionKeepAlive::try_from(responses).expect("Failed to parse"),
    };

    if let MimicRequestAction::RequestActionKeepAlive { inner } = raka_mimic {
        let _ = RequestAction::KeepAlive {
            latest_response_ack: inner.instance.f0,
        };
    };
}

#[test]
fn make_mimicarg9_heterogenous() {
    let responses = vec![
        "19".to_owned(),
        "13".to_owned(),
        "1.5".to_owned(),
        "Denth".to_owned(),
        "-1089189128".to_owned(),
        "13".to_owned(),
        "1.5".to_owned(),
        "".to_owned(),
        "0".to_owned(),
    ];

    if let Ok(fca9) = MimicArg9::<usize, isize, f64, String, i64, i8, f32, String, u8>::try_from(
        responses,
    ) {
            assert_eq!(fca9.f0, 19);
            assert_eq!(fca9.f1, 13);
            assert_eq!(fca9.f2, 1.5);
            assert_eq!(fca9.f3, "Denth".to_owned());
            assert_eq!(fca9.f4, -1089189128);
            assert_eq!(fca9.f5, 13);
            assert_eq!(fca9.f6, 1.5);
            assert_eq!(fca9.f7, "".to_owned());
            assert_eq!(fca9.f8, 0);
    } else {
        panic!("User input was not parsable");
    }
}

#[test]
#[should_panic]
fn make_not_enough_parameters_to_parse() {
    MimicArg2::<String, isize>::try_from(vec!["undersized".to_owned()]).unwrap();
}

#[test]
#[should_panic]
fn make_too_many_parameters_to_parse() {
    MimicArg2::<String, isize>::try_from(vec!["1".to_owned(), "2".to_owned(), "3".to_owned()])
        .unwrap();
}

#[test]
fn make_mimic_list_from_string() {
    use std::iter::zip;

    let a: Vec<isize> = "1; 2; 3".parse::<MimicList<isize>>().unwrap().into();
    let b = vec![1, 2, 3];

    let _ = zip(a, b).map(|(a, b)| {
        assert_eq!(a, b);
    });
}

#[test]
fn make_mimic_list_from_complex_type() {
    use std::iter::zip;

    #[derive(Debug, PartialEq, Eq)]
    struct Foo {
        a: isize,
        b: String,
    }

    impl FromStr for Foo {
        type Err = &'static str;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let list = s.split(",").collect::<Vec<&str>>();
            assert_eq!(list.len(), 2);

            let a = list[0].parse::<isize>().map_err(|_|"Failed to parse first arg as isize")?;
            let b = list[1].parse::<String>().map_err(|_|"Failed to parse second arg as String")?;
            Ok(Foo { a, b })
        }
    }

    let a: Vec<Foo> = "1,2;    2, 3      ; 3,4"
        .parse::<MimicList<Foo>>()
        .unwrap()
        .into();
    let b = vec![
        Foo {
            a: 1,
            b: "2".to_owned(),
        },
        Foo {
            a: 2,
            b: "3".to_owned(),
        },
        Foo {
            a: 3,
            b: "4".to_owned(),
        },
    ];

    let _ = zip(a, b).map(|(a, b)| {
        assert_eq!(a, b);
    });
}

#[test]
fn make_mimic_list_from_mimicarg2() {
    use std::iter::zip;

    let a: Vec<MimicArg2<isize, String>> = "1,2;    2, 3      ; 3,4;"
        .parse::<MimicList<MimicArg2<isize, String>>>()
        .unwrap()
        .into();
    let b = vec![
        MimicArg2 {
            f0: 1,
            f1: "2".to_owned(),
        },
        MimicArg2{
            f0: 2,
            f1: "3".to_owned(),
        },
        MimicArg2{
            f0: 3,
            f1: "4".to_owned(),
        },
    ];

    let _ = zip(a, b).map(|(a, b) : (MimicArg2<isize, String>, MimicArg2<isize, String>)| {
        assert_eq!(a.f0, b.f0);
        assert_eq!(a.f1, b.f1);
    });
}