//! This integration test tries to run and call the generated wasm.
//! It depends on a Wasm build being available, which you can create with `cargo wasm`.
//! Then running `cargo integration-test` will validate we can properly call into that generated Wasm.
//!
//! You can easily convert unit tests to integration tests.
//! 1. First copy them over verbatum,
//! 2. Then change
//!      let mut deps = mock_dependencies(20, &[]);
//!    to
//!      let mut deps = mock_instance(WASM, &[]);
//! 3. If you access raw storage, where ever you see something like:
//!      deps.storage.get(CONFIG_KEY).expect("no data stored");
//!    replace it with:
//!      deps.with_storage(|store| {
//!          let data = store.get(CONFIG_KEY).expect("no data stored");
//!          //...
//!      });
//! 4. Anywhere you see query(&deps, ...) you must replace it with query(&mut deps, ...)
//! 5. When matching on error codes, you can not use Error types, but rather corresponding ApiError variants.
//!    Note that you don't have backtrace field and can often skip the .. filler:
//!      match res.unwrap_err() {
//!          Error::Unauthorized { .. } => {}
//!          _ => panic!("Must return unauthorized error"),
//!      }
//!    becomes:
//!      match res.unwrap_err() {
//!          ApiError::Unauthorized {} => {}
//!          _ => panic!("Must return unauthorized error"),
//!      }

use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{coins, from_binary, ApiError};

use cosmwasm_vm::testing::{handle, init, mock_instance, query};

use {{crate_name}}::msg::{CountResponse, HandleMsg, InitMsg, QueryMsg};

// This line will test the output of cargo wasm
static WASM: &[u8] = include_bytes!("../../../target/wasm32-unknown-unknown/release/{{crate_name}}.wasm");

#[test]
fn proper_initialization() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InitMsg { count: 17 };
    let env = mock_env(&deps.api, "creator", &coins(1000, "earth"));

    // we can just call .unwrap() to assert this was a success
    let res = init(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(&mut deps, QueryMsg::GetCount {}).unwrap();
    let value: CountResponse = from_binary(&res).unwrap();
    assert_eq!(17, value.count);
}

#[test]
fn increment() {
    let mut deps = mock_instance(WASM, &coins(2, "token"));

    let msg = InitMsg { count: 17 };
    let env = mock_env(&deps.api, "creator", &coins(2, "token"));
    let _res = init(&mut deps, env, msg).unwrap();

    // beneficiary can release it
    let env = mock_env(&deps.api, "anyone", &coins(2, "token"));
    let msg = HandleMsg::Increment {};
    let _res = handle(&mut deps, env, msg).unwrap();

    // should increase counter by 1
    let res = query(&mut deps, QueryMsg::GetCount {}).unwrap();
    let value: CountResponse = from_binary(&res).unwrap();
    assert_eq!(18, value.count);
}

#[test]
fn reset() {
    let mut deps = mock_instance(WASM, &coins(2, "token"));

    let msg = InitMsg { count: 17 };
    let env = mock_env(&deps.api, "creator", &coins(2, "token"));
    let _res = init(&mut deps, env, msg).unwrap();

    // beneficiary can release it
    let unauth_env = mock_env(&deps.api, "anyone", &coins(2, "token"));
    let msg = HandleMsg::Reset { count: 5 };
    let res = handle(&mut deps, unauth_env, msg);
    match res.unwrap_err() {
        ApiError::Unauthorized { .. } => {}
        _ => panic!("Expected unauthorized"),
    }

    // only the original creator can reset the counter
    let auth_env = mock_env(&deps.api, "creator", &coins(2, "token"));
    let msg = HandleMsg::Reset { count: 5 };
    let _res = handle(&mut deps, auth_env, msg).unwrap();

    // should now be 5
    let res = query(&mut deps, QueryMsg::GetCount {}).unwrap();
    let value: CountResponse = from_binary(&res).unwrap();
    assert_eq!(5, value.count);
}
