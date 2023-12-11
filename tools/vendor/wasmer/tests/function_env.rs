use macro_wasmer_universal_test::universal_test;
#[cfg(feature = "js")]
use wasm_bindgen_test::*;

use wasmer::*;

#[universal_test]
fn data_and_store_mut() -> Result<(), String> {
    let mut store = Store::default();
    let global_mut = Global::new_mut(&mut store, Value::I32(10));
    struct Env {
        value: i32,
        global: Global,
    }
    let env = FunctionEnv::new(
        &mut store,
        Env {
            value: 0i32,
            global: global_mut,
        },
    );
    let mut envmut = env.into_mut(&mut store);

    let (mut data, mut storemut) = envmut.data_and_store_mut();

    assert_eq!(
        data.global.ty(&mut storemut),
        GlobalType {
            ty: Type::I32,
            mutability: Mutability::Var
        }
    );
    assert_eq!(data.global.get(&mut storemut), Value::I32(10));
    data.value = data.global.get(&mut storemut).unwrap_i32() + 10;

    data.global
        .set(&mut storemut, Value::I32(data.value))
        .unwrap();

    assert_eq!(data.global.get(&mut storemut), Value::I32(data.value));

    Ok(())
}
