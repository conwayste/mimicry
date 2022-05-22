# The Initial Thought Experiment (Does Not Compile On Its Own)

```Rust
#[derive(Debug)]
enum RequestAction {
    None,
    Connect {
        name: String,
        client_version: String,
    },
}

#[derive(Default)]
pub struct MimicArg0 {}

#[derive(Default)]
pub struct MimicArg1<T> {
    pub f0: T,
}

#[derive(Default)]
pub struct MimicArg2<T, U> {
    pub f0: T,
    pub f1: U,
}

impl<T> TryFrom<Vec<String>> for MimicArg1<T>
where
    T: Debug + FromStr,
{
    type Error = &'static str;

    fn try_from(input: Vec<String>) -> Result<Self, Self::Error> {
        if let Ok(f0) = input[0].parse::<T>() {
            return Ok(MimicArg1 { f0 });
        }
        Err("Failed to parse `T` in MimicArg2<T>")
    }
}

impl<T, U> TryFrom<Vec<String>> for MimicArg2<T, U>
where
    T: Debug + FromStr,
    U: Debug + FromStr,
{
    type Error = &'static str;

    fn try_from(input: Vec<String>) -> Result<Self, Self::Error> {
        if let Ok(f0) = input[0].parse::<T>() {
            if let Ok(f1) = input[1].parse::<U>() {
                return Ok(MimicArg2 { f0, f1 });
            }
            return Err("Failed to parse `U` in MimicArg2<T,U>");
        }
        Err("Failed to parse `T` in MimicArg2<T,U>")
    }
}

impl<R, S> FromStr for MimicArg2<R, S>
where
    R: FromStr,
    S: FromStr,
{
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let csv = s.split(",").map(|s: &str| s.trim()).collect::<Vec<&str>>();
        if csv.is_empty() {
            return Err("No comma-separated list found in input for MimicList");
        }
        if csv.len() != 2 {
            return Err("Input must contain only 2 values");
        }

        Ok(MimicArg2 {
            f0: csv[0].parse::<R>().map_err(|_| "Failed to parse first arg as R")?,
            f1: csv[1]
                .parse::<S>()
                .map_err(|_| "Failed to parse second arg as S")?,
        })
    }
}

#[derive(Debug)]
pub struct VecISize(Vec<isize>);

impl FromStr for VecISize {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let csv = s.split(",").map(|s: &str| s.trim()).collect::<Vec<&str>>();
        let mut parsed = Vec::<isize>::new();
        for item in csv {
            if let Ok(zero) = item.parse::<isize>() {
                parsed.push(zero);
            } else {
                return Err(format!("Failed to parse list of #self: {}", item));
            }
        }

        Ok(VecISize(parsed))
    }
}

pub struct RequestActionConnect {
    pub meta: MimicMetadata,
    pub command: MimicArg2<String, String>,
}

impl Default for RequestActionConnect {
    fn default() -> Self {
        RequestActionConnect {
            meta: MimicMetadata {
                name: "Connect",
                fields: vec![
                    MimicFieldData {
                        name: "name",
                        type_: "String",
                        type_arguments: vec![],
                    },
                    MimicFieldData {
                        name: "client_version",
                        type_: "String",
                        type_arguments: vec![],
                    },
                ],
            },
            command: MimicArg2::<String, String> {
                f0: "".into(),
                f1: "".into(),
            },
        }
    }
}

fn main () {
    for cmd in MimicRequestAction::iter() {
        match cmd {
            MimicRequestAction::RequestActionConnect {inner } =>
            {
                println!("{}", inner.meta.name);
                for field in inner.meta.fields {
                    println!("\t{}: {}", field.name, field.type_);
                }

                let responses = vec!["uno".to_owned(), "1.5.2".to_owned()];
                let fca2 = MimicArg2::<String, String>::try_from(responses).unwrap();
                println!("{} {}", fca2.zero, fca2.one);

                let responses = vec!["1".to_owned(), "1.5".to_owned()];
                let fca2 = MimicArg2::<usize, f32>::try_from(responses).unwrap();
                println!("{} {}", fca2.zero, fca2.one);

                let responses = vec!["1, 2, 3".to_owned(), "1.5".to_owned()];
                let fca2 = MimicArg2::<VecISize, f32>::try_from(responses).unwrap();
                println!("{:?} {}", fca2.zero, fca2.one);
            }
            MimicRequestAction::RequestActionNone => {},
        }
    }
}
```