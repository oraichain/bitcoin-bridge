#![no_main]
use libfuzzer_sys::fuzz_target;
use orga::call::Call;
use orga::encoding::{Decode, Encode};
use orga::state::State;

type AppCall = <nomic::app::App as Call>::Call;

fuzz_target!(|data: &[u8]| {
    let mut data = &data[..];
    let mut calls = Vec::new();

    while data.len() >= 2 {
        let len = match u16::decode(data) {
            Ok(len) => len as usize,
            Err(_) => return,
        };

        if data.len() < len {
            break;
        }

        let call = match AppCall::decode(data) {
            Ok(call) => call,
            Err(_) => return,
        };

        calls.push(call);

        if calls.len() > 10 {
            break;
        }
    }

    let mut store = orga::store::Store::new(orga::store::Shared::new(orga::store::MapStore::new()).into());
    let mut app = nomic::app::App::create(store, Default::default());

    for call in calls {
        let res = match app.call(call) {
            Ok(res) => res,
            Err(_) => return,
        };
    }   
});
