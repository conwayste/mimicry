extern crate mimicry;

use std::{fmt::Debug, str::FromStr, vec};
use strum_macros::Display;

use mimicry::*;

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

fn main() {
//
}
