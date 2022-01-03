#![no_main]
use libfuzzer_sys::fuzz_target;
use orga::call::Call;
use orga::encoding::{Decode, Encode};
use orga::state::State;

type AppCall = <nomic::app::App as Call>::Call;

fuzz_target!(|data: &[u8]| {
    let call = match AppCall::decode(&mut &data[..]) {
        Ok(call) => call,
        Err(_) => return,
    };

    let mut store = orga::store::Store::new(orga::store::Shared::new(orga::store::MapStore::new()).into());
    let mut app = nomic::app::App::create(store, Default::default());

    let res = match app.call(call) {
        Ok(res) => res,
        Err(_) => return,
    };
});
