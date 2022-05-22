// CLI - Cat Lister & Instantiator

use mimicry::*;
use std::str::FromStr;

#[allow(unused)]
#[derive(Debug)]
pub struct BOOL {
    a: bool,
}

impl From<&str> for BOOL {
    fn from(s: &str) -> Self {
        let s = s.trim();

        // Adequate for this example
        if s.to_ascii_lowercase() == "true" {
            BOOL { a: true }
        } else {
            BOOL { a: false }
        }
    }
}

impl FromStr for BOOL {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let a;
        match s.parse::<bool>() {
            Ok(value) => a = value,
            Err(_) => return Err("BOOL failed to parse as a boolean in from_str"),
        };
        Ok(BOOL { a })
    }
}

#[allow(unused)]
#[derive(Debug, Mimic)]
enum CatSelector {
    None,
    Unicolor {
        name: String,
        color: String,
    },
    Mixed {
        name: String,
        // There is no implementation for From<&str> for bool so we have to make one ourselves
        with_stripes: BOOL,
    },
    Tabby {
        fluff_ratio: f32,
    },
    Chungus {
        chung_ratio: usize,
    },
}

use std::io::{self, Write};

pub fn get_input(prompt: &str) -> String {
    let mut input = String::new();

    print!("{} ", prompt);
    let _ = io::stdout().flush().expect("Failed to flush stdout.");

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input.");

    input.trim().to_owned()
}

fn main() {
    println!(
        "Selections:
    1. Unicolor
    2. Mixed
    3. Tabby
    4. Chungus"
    );
    match get_input("Select a Cat (1-4):").parse::<usize>() {
        Ok(n @ 1) | Ok(n @ 2) | Ok(n @ 3) | Ok(n @ 4) => {
            match n {
                1 => {
                    let responses = vec![
                        get_input("Enter the cats name:"),
                        get_input("Enter the cats color (get creative):"),
                    ];
                    let mimic = MimicCatSelector::CatSelectorUnicolor {
                        inner: CatSelectorUnicolor::try_from(responses)
                            .expect("Input not recognized for a unicolor cat"),
                    };

                    if let MimicCatSelector::CatSelectorUnicolor { inner } = mimic {
                        let cat = CatSelector::Unicolor {
                            name: inner.instance.f0,
                            color: inner.instance.f1,
                        };

                        println!("{:?}", cat);
                    };
                }
                2 => {
                    let responses = vec![
                        get_input("Enter the cats name:"),
                        get_input("Does this cat have stripes? (true/false) "),
                    ];
                    let mimic = MimicCatSelector::CatSelectorMixed {
                        inner: CatSelectorMixed::try_from(responses)
                            .expect("Input not recognized for a mixed cat"),
                    };

                    if let MimicCatSelector::CatSelectorMixed { inner } = mimic {
                        let cat = CatSelector::Mixed {
                            name: inner.instance.f0,
                            with_stripes: inner.instance.f1,
                        };

                        println!("{:?}", cat);
                    };
                }
                3 => {
                    let responses = vec![get_input("How fluffy is this cat? (A decimal):")];
                    let mimic = MimicCatSelector::CatSelectorTabby {
                        inner: CatSelectorTabby::try_from(responses)
                            .expect("Input not recognized for a tabby cat"),
                    };

                    if let MimicCatSelector::CatSelectorTabby { inner } = mimic {
                        let cat = CatSelector::Tabby {
                            fluff_ratio: inner.instance.f0,
                        };

                        println!("{:?}", cat);
                    };
                }
                4 => {
                    let responses = vec![get_input("Chungus approximation (A positive integer):")];
                    let mimic = MimicCatSelector::CatSelectorChungus {
                        inner: CatSelectorChungus::try_from(responses)
                            .expect("Input not recognized for a chungus cat"),
                    };

                    if let MimicCatSelector::CatSelectorChungus { inner } = mimic {
                        let cat = CatSelector::Chungus {
                            chung_ratio: inner.instance.f0,
                        };

                        println!("{:?}", cat);
                    };
                }
                _ => {}
            };
        }
        Ok(_) | Err(_) => {}
    }
}
