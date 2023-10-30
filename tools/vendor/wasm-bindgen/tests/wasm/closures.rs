use js_sys::Number;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/closures.js")]
extern "C" {
    fn works_call(a: &dyn Fn());
    fn works_thread(a: &dyn Fn(u32) -> u32) -> u32;

    fn cannot_reuse_call(a: &dyn Fn());
    #[wasm_bindgen(catch)]
    fn cannot_reuse_call_again() -> Result<(), JsValue>;

    fn long_lived_call1(a: &Closure<dyn Fn()>);
    fn long_lived_call2(a: &Closure<dyn FnMut(u32) -> u32>) -> u32;

    fn many_arity_call1(a: &Closure<dyn Fn()>);
    fn many_arity_call2(a: &Closure<dyn Fn(u32)>);
    fn many_arity_call3(a: &Closure<dyn Fn(u32, u32)>);
    fn many_arity_call4(a: &Closure<dyn Fn(u32, u32, u32)>);
    fn many_arity_call5(a: &Closure<dyn Fn(u32, u32, u32, u32)>);
    fn many_arity_call6(a: &Closure<dyn Fn(u32, u32, u32, u32, u32)>);
    fn many_arity_call7(a: &Closure<dyn Fn(u32, u32, u32, u32, u32, u32)>);
    fn many_arity_call8(a: &Closure<dyn Fn(u32, u32, u32, u32, u32, u32, u32)>);
    fn many_arity_call9(a: &Closure<dyn Fn(u32, u32, u32, u32, u32, u32, u32, u32)>);

    #[wasm_bindgen(js_name = many_arity_call1)]
    fn many_arity_call_mut1(a: &Closure<dyn FnMut()>);
    #[wasm_bindgen(js_name = many_arity_call2)]
    fn many_arity_call_mut2(a: &Closure<dyn FnMut(u32)>);
    #[wasm_bindgen(js_name = many_arity_call3)]
    fn many_arity_call_mut3(a: &Closure<dyn FnMut(u32, u32)>);
    #[wasm_bindgen(js_name = many_arity_call4)]
    fn many_arity_call_mut4(a: &Closure<dyn FnMut(u32, u32, u32)>);
    #[wasm_bindgen(js_name = many_arity_call5)]
    fn many_arity_call_mut5(a: &Closure<dyn FnMut(u32, u32, u32, u32)>);
    #[wasm_bindgen(js_name = many_arity_call6)]
    fn many_arity_call_mut6(a: &Closure<dyn FnMut(u32, u32, u32, u32, u32)>);
    #[wasm_bindgen(js_name = many_arity_call7)]
    fn many_arity_call_mut7(a: &Closure<dyn FnMut(u32, u32, u32, u32, u32, u32)>);
    #[wasm_bindgen(js_name = many_arity_call8)]
    fn many_arity_call_mut8(a: &Closure<dyn FnMut(u32, u32, u32, u32, u32, u32, u32)>);
    #[wasm_bindgen(js_name = many_arity_call9)]
    fn many_arity_call_mut9(a: &Closure<dyn FnMut(u32, u32, u32, u32, u32, u32, u32, u32)>);

    fn option_call1(a: Option<&Closure<dyn Fn()>>);
    fn option_call2(a: Option<&Closure<dyn FnMut(u32) -> u32>>) -> u32;
    fn option_call3(a: Option<&Closure<dyn Fn()>>) -> bool;

    #[wasm_bindgen(js_name = many_arity_call1)]
    fn many_arity_stack1(a: &dyn Fn());
    #[wasm_bindgen(js_name = many_arity_call2)]
    fn many_arity_stack2(a: &dyn Fn(u32));
    #[wasm_bindgen(js_name = many_arity_call3)]
    fn many_arity_stack3(a: &dyn Fn(u32, u32));
    #[wasm_bindgen(js_name = many_arity_call4)]
    fn many_arity_stack4(a: &dyn Fn(u32, u32, u32));
    #[wasm_bindgen(js_name = many_arity_call5)]
    fn many_arity_stack5(a: &dyn Fn(u32, u32, u32, u32));
    #[wasm_bindgen(js_name = many_arity_call6)]
    fn many_arity_stack6(a: &dyn Fn(u32, u32, u32, u32, u32));
    #[wasm_bindgen(js_name = many_arity_call7)]
    fn many_arity_stack7(a: &dyn Fn(u32, u32, u32, u32, u32, u32));
    #[wasm_bindgen(js_name = many_arity_call8)]
    fn many_arity_stack8(a: &dyn Fn(u32, u32, u32, u32, u32, u32, u32));
    #[wasm_bindgen(js_name = many_arity_call9)]
    fn many_arity_stack9(a: &dyn Fn(u32, u32, u32, u32, u32, u32, u32, u32));

    fn long_lived_dropping_cache(a: &Closure<dyn Fn()>);
    #[wasm_bindgen(catch)]
    fn long_lived_dropping_call() -> Result<(), JsValue>;

    fn long_lived_option_dropping_cache(a: Option<&Closure<dyn Fn()>>) -> bool;
    #[wasm_bindgen(catch)]
    fn long_lived_option_dropping_call() -> Result<(), JsValue>;

    fn long_fnmut_recursive_cache(a: &Closure<dyn FnMut()>);
    #[wasm_bindgen(catch)]
    fn long_fnmut_recursive_call() -> Result<(), JsValue>;

    fn fnmut_call(a: &mut dyn FnMut());
    fn fnmut_thread(a: &mut dyn FnMut(u32) -> u32) -> u32;

    fn fnmut_bad_call(a: &mut dyn FnMut());
    #[wasm_bindgen(catch)]
    fn fnmut_bad_again(a: bool) -> Result<(), JsValue>;

    fn string_arguments_call(a: &mut dyn FnMut(String));

    fn string_ret_call(a: &mut dyn FnMut(String) -> String);

    fn drop_during_call_save(a: &Closure<dyn Fn()>);
    fn drop_during_call_call();

    fn js_test_closure_returner();

    fn calling_it_throws(a: &Closure<dyn FnMut()>) -> bool;

    fn call_val(f: &JsValue);

    #[wasm_bindgen(js_name = calling_it_throws)]
    fn call_val_throws(f: &JsValue) -> bool;

    fn pass_reference_first_arg_twice(
        a: RefFirstArgument,
        b: &Closure<dyn FnMut(&RefFirstArgument)>,
        c: &Closure<dyn FnMut(&RefFirstArgument)>,
    );
    #[wasm_bindgen(js_name = pass_reference_first_arg_twice)]
    fn pass_reference_first_arg_twice2(
        a: RefFirstArgument,
        b: &mut dyn FnMut(&RefFirstArgument),
        c: &mut dyn FnMut(&RefFirstArgument),
    );
    fn call_destroyed(a: &JsValue);

    fn js_store_forgotten_closure(closure: &Closure<dyn Fn()>);
    fn js_call_forgotten_closure();

    #[wasm_bindgen(js_name = many_arity_call2)]
    fn externref_call(a: &Closure<dyn Fn(JsValue)>);
    #[wasm_bindgen(js_name = many_arity_call2)]
    fn named_externref_call(a: &Closure<dyn Fn(Number)>);
}

#[wasm_bindgen_test]
fn works() {
    let a = Cell::new(false);
    works_call(&|| a.set(true));
    assert!(a.get());

    assert_eq!(works_thread(&|a| a + 1), 3);
}

#[wasm_bindgen_test]
fn cannot_reuse() {
    cannot_reuse_call(&|| {});
    assert!(cannot_reuse_call_again().is_err());
}

#[wasm_bindgen_test]
fn debug() {
    let closure = Closure::wrap(Box::new(|| {}) as Box<dyn FnMut()>);
    assert_eq!(&format!("{:?}", closure), "Closure { ... }");
}

#[wasm_bindgen_test]
fn long_lived() {
    let hit = Rc::new(Cell::new(false));
    let hit2 = hit.clone();
    let a = Closure::new(move || hit2.set(true));
    assert!(!hit.get());
    long_lived_call1(&a);
    assert!(hit.get());

    let hit = Rc::new(Cell::new(false));
    {
        let hit = hit.clone();
        let a = Closure::new(move |x| {
            hit.set(true);
            x + 3
        });
        assert_eq!(long_lived_call2(&a), 5);
    }
    assert!(hit.get());
}

#[wasm_bindgen_test]
fn many_arity() {
    many_arity_call1(&Closure::new(|| {}));
    many_arity_call2(&Closure::new(|a| assert_eq!(a, 1)));
    many_arity_call3(&Closure::new(|a, b| assert_eq!((a, b), (1, 2))));
    many_arity_call4(&Closure::new(|a, b, c| assert_eq!((a, b, c), (1, 2, 3))));
    many_arity_call5(&Closure::new(|a, b, c, d| {
        assert_eq!((a, b, c, d), (1, 2, 3, 4))
    }));
    many_arity_call6(&Closure::new(|a, b, c, d, e| {
        assert_eq!((a, b, c, d, e), (1, 2, 3, 4, 5))
    }));
    many_arity_call7(&Closure::new(|a, b, c, d, e, f| {
        assert_eq!((a, b, c, d, e, f), (1, 2, 3, 4, 5, 6))
    }));
    many_arity_call8(&Closure::new(|a, b, c, d, e, f, g| {
        assert_eq!((a, b, c, d, e, f, g), (1, 2, 3, 4, 5, 6, 7))
    }));
    many_arity_call9(&Closure::new(|a, b, c, d, e, f, g, h| {
        assert_eq!((a, b, c, d, e, f, g, h), (1, 2, 3, 4, 5, 6, 7, 8))
    }));

    let s = String::new();
    many_arity_call_mut1(&Closure::once(move || drop(s)));
    let s = String::new();
    many_arity_call_mut2(&Closure::once(move |a| {
        drop(s);
        assert_eq!(a, 1);
    }));
    let s = String::new();
    many_arity_call_mut3(&Closure::once(move |a, b| {
        drop(s);
        assert_eq!((a, b), (1, 2));
    }));
    let s = String::new();
    many_arity_call_mut4(&Closure::once(move |a, b, c| {
        drop(s);
        assert_eq!((a, b, c), (1, 2, 3));
    }));
    let s = String::new();
    many_arity_call_mut5(&Closure::once(move |a, b, c, d| {
        drop(s);
        assert_eq!((a, b, c, d), (1, 2, 3, 4));
    }));
    let s = String::new();
    many_arity_call_mut6(&Closure::once(move |a, b, c, d, e| {
        drop(s);
        assert_eq!((a, b, c, d, e), (1, 2, 3, 4, 5));
    }));
    let s = String::new();
    many_arity_call_mut7(&Closure::once(move |a, b, c, d, e, f| {
        drop(s);
        assert_eq!((a, b, c, d, e, f), (1, 2, 3, 4, 5, 6));
    }));
    let s = String::new();
    many_arity_call_mut8(&Closure::once(move |a, b, c, d, e, f, g| {
        drop(s);
        assert_eq!((a, b, c, d, e, f, g), (1, 2, 3, 4, 5, 6, 7));
    }));
    let s = String::new();
    many_arity_call_mut9(&Closure::once(move |a, b, c, d, e, f, g, h| {
        drop(s);
        assert_eq!((a, b, c, d, e, f, g, h), (1, 2, 3, 4, 5, 6, 7, 8));
    }));

    many_arity_stack1(&(|| {}));
    many_arity_stack2(&(|a| assert_eq!(a, 1)));
    many_arity_stack3(&(|a, b| assert_eq!((a, b), (1, 2))));
    many_arity_stack4(&(|a, b, c| assert_eq!((a, b, c), (1, 2, 3))));
    many_arity_stack5(&(|a, b, c, d| assert_eq!((a, b, c, d), (1, 2, 3, 4))));
    many_arity_stack6(&(|a, b, c, d, e| assert_eq!((a, b, c, d, e), (1, 2, 3, 4, 5))));
    many_arity_stack7(&(|a, b, c, d, e, f| assert_eq!((a, b, c, d, e, f), (1, 2, 3, 4, 5, 6))));
    many_arity_stack8(
        &(|a, b, c, d, e, f, g| assert_eq!((a, b, c, d, e, f, g), (1, 2, 3, 4, 5, 6, 7))),
    );
    many_arity_stack9(
        &(|a, b, c, d, e, f, g, h| assert_eq!((a, b, c, d, e, f, g, h), (1, 2, 3, 4, 5, 6, 7, 8))),
    );
}

#[wasm_bindgen_test]
fn option() {
    let hit = Rc::new(Cell::new(false));
    let hit2 = hit.clone();
    let a = Closure::new(move || hit2.set(true));
    assert!(!hit.get());
    option_call1(Some(&a));
    assert!(hit.get());

    let hit = Rc::new(Cell::new(false));
    {
        let hit = hit.clone();
        let a = Closure::new(move |x| {
            hit.set(true);
            x + 3
        });
        assert_eq!(option_call2(Some(&a)), 5);
    }
    assert!(hit.get());

    assert!(option_call3(None));
}

struct Dropper(Rc<Cell<bool>>);
impl Drop for Dropper {
    fn drop(&mut self) {
        assert!(!self.0.get());
        self.0.set(true);
    }
}

#[wasm_bindgen_test]
fn call_fn_once_twice() {
    let dropped = Rc::new(Cell::new(false));
    let dropper = Dropper(dropped.clone());
    let called = Rc::new(Cell::new(false));

    let c = Closure::once({
        let called = called.clone();
        move || {
            assert!(!called.get());
            called.set(true);
            drop(dropper);
        }
    });

    many_arity_call_mut1(&c);
    assert!(called.get());
    assert!(dropped.get());

    assert!(calling_it_throws(&c));
}

#[wasm_bindgen_test]
fn once_into_js() {
    let dropped = Rc::new(Cell::new(false));
    let dropper = Dropper(dropped.clone());
    let called = Rc::new(Cell::new(false));

    let f = Closure::once_into_js({
        let called = called.clone();
        move || {
            assert!(!called.get());
            called.set(true);
            drop(dropper);
        }
    });

    call_val(&f);
    assert!(called.get());
    assert!(dropped.get());

    assert!(call_val_throws(&f));
}

#[wasm_bindgen_test]
fn long_lived_dropping() {
    let hit = Rc::new(Cell::new(false));
    let hit2 = hit.clone();
    let a = Closure::new(move || hit2.set(true));
    long_lived_dropping_cache(&a);
    assert!(!hit.get());
    assert!(long_lived_dropping_call().is_ok());
    assert!(hit.get());
    drop(a);
    assert!(long_lived_dropping_call().is_err());
}

#[wasm_bindgen_test]
fn long_lived_option_dropping() {
    let hit = Rc::new(Cell::new(false));
    let hit2 = hit.clone();

    let a = Closure::new(move || hit2.set(true));

    assert!(!long_lived_option_dropping_cache(None));
    assert!(long_lived_option_dropping_cache(Some(&a)));

    assert!(!hit.get());
    assert!(long_lived_option_dropping_call().is_ok());
    assert!(hit.get());

    drop(a);
    assert!(long_lived_option_dropping_call().is_err());
}

#[wasm_bindgen_test]
fn long_fnmut_recursive() {
    let a = Closure::new(|| {
        assert!(long_fnmut_recursive_call().is_err());
    });
    long_fnmut_recursive_cache(&a);
    assert!(long_fnmut_recursive_call().is_ok());
}

#[wasm_bindgen_test]
fn fnmut() {
    let mut a = false;
    fnmut_call(&mut || a = true);
    assert!(a);

    let mut x = false;
    assert_eq!(
        fnmut_thread(&mut |a| {
            x = true;
            a + 1
        }),
        3
    );
    assert!(x);
}

#[wasm_bindgen_test]
fn fnmut_bad() {
    let mut x = true;
    let mut hits = 0;
    fnmut_bad_call(&mut || {
        hits += 1;
        if fnmut_bad_again(hits == 1).is_err() {
            return;
        }
        x = false;
    });
    assert_eq!(hits, 1);
    assert!(x);

    assert!(fnmut_bad_again(true).is_err());
}

#[wasm_bindgen_test]
fn string_arguments() {
    let mut x = false;
    string_arguments_call(&mut |s| {
        assert_eq!(s, "foo");
        x = true;
    });
    assert!(x);
}

#[wasm_bindgen_test]
fn string_ret() {
    let mut x = false;
    string_ret_call(&mut |mut s| {
        assert_eq!(s, "foo");
        s.push_str("bar");
        x = true;
        s
    });
    assert!(x);
}

#[wasm_bindgen_test]
fn drop_drops() {
    static mut HIT: bool = false;

    struct A;

    impl Drop for A {
        fn drop(&mut self) {
            unsafe {
                HIT = true;
            }
        }
    }
    let a = A;
    let x: Closure<dyn Fn()> = Closure::new(move || drop(&a));
    drop(x);
    unsafe {
        assert!(HIT);
    }
}

#[wasm_bindgen_test]
fn drop_during_call_ok() {
    static mut HIT: bool = false;
    struct A;
    impl Drop for A {
        fn drop(&mut self) {
            unsafe {
                HIT = true;
            }
        }
    }

    let rc = Rc::new(RefCell::new(None));
    let rc2 = rc.clone();
    let x = 3;
    let a = A;
    let x: Closure<dyn Fn()> = Closure::new(move || {
        // "drop ourselves"
        drop(rc2.borrow_mut().take().unwrap());

        // `A` should not have been destroyed as a result
        unsafe {
            assert!(!HIT);
        }

        // allocate some heap memory to try to paper over our `3`
        drop(String::from("1234567890"));

        // make sure our closure memory is still valid
        assert_eq!(x, 3);

        // make sure `A` is bound to our closure environment.
        drop(&a);
        unsafe {
            assert!(!HIT);
        }
    });
    drop_during_call_save(&x);
    *rc.borrow_mut() = Some(x);
    drop(rc);
    unsafe {
        assert!(!HIT);
    }
    drop_during_call_call();
    unsafe {
        assert!(HIT);
    }
}

#[wasm_bindgen_test]
fn test_closure_returner() {
    type ClosureType = dyn FnMut() -> BadStruct;

    use js_sys::{Object, Reflect};

    js_test_closure_returner();

    #[wasm_bindgen]
    pub struct ClosureHandle(Closure<ClosureType>);

    #[wasm_bindgen]
    pub struct BadStruct {}

    #[wasm_bindgen]
    pub fn closure_returner() -> Result<Object, JsValue> {
        let o = Object::new();

        let some_fn = Closure::wrap(Box::new(move || BadStruct {}) as Box<ClosureType>);
        Reflect::set(
            &o,
            &JsValue::from("someKey"),
            some_fn.as_ref().unchecked_ref(),
        )
        .unwrap();
        Reflect::set(
            &o,
            &JsValue::from("handle"),
            &JsValue::from(ClosureHandle(some_fn)),
        )
        .unwrap();

        Ok(o)
    }
}

#[wasm_bindgen]
pub struct RefFirstArgument {
    contents: u32,
}

#[wasm_bindgen_test]
fn reference_as_first_argument_builds_at_all() {
    #[wasm_bindgen]
    extern "C" {
        fn ref_first_arg1(a: &dyn Fn(&JsValue));
        fn ref_first_arg2(a: &mut dyn FnMut(&JsValue));
        fn ref_first_arg3(a: &Closure<dyn Fn(&JsValue)>);
        fn ref_first_arg4(a: &Closure<dyn FnMut(&JsValue)>);
        fn ref_first_custom1(a: &dyn Fn(&RefFirstArgument));
        fn ref_first_custom2(a: &mut dyn FnMut(&RefFirstArgument));
        fn ref_first_custom3(a: &Closure<dyn Fn(&RefFirstArgument)>);
        fn ref_first_custom4(a: &Closure<dyn FnMut(&RefFirstArgument)>);
    }

    Closure::wrap(Box::new(|_: &JsValue| ()) as Box<dyn Fn(&JsValue)>);
    Closure::wrap(Box::new(|_: &JsValue| ()) as Box<dyn FnMut(&JsValue)>);
    Closure::once(|_: &JsValue| ());
    Closure::once_into_js(|_: &JsValue| ());
    Closure::wrap(Box::new(|_: &RefFirstArgument| ()) as Box<dyn Fn(&RefFirstArgument)>);
    Closure::wrap(Box::new(|_: &RefFirstArgument| ()) as Box<dyn FnMut(&RefFirstArgument)>);
    Closure::once(|_: &RefFirstArgument| ());
    Closure::once_into_js(|_: &RefFirstArgument| ());
}

#[wasm_bindgen_test]
fn reference_as_first_argument_works() {
    let a = Rc::new(Cell::new(0));
    let b = {
        let a = a.clone();
        Closure::once(move |x: &RefFirstArgument| {
            assert_eq!(a.get(), 0);
            assert_eq!(x.contents, 3);
            a.set(a.get() + 1);
        })
    };
    let c = {
        let a = a.clone();
        Closure::once(move |x: &RefFirstArgument| {
            assert_eq!(a.get(), 1);
            assert_eq!(x.contents, 3);
            a.set(a.get() + 1);
        })
    };
    pass_reference_first_arg_twice(RefFirstArgument { contents: 3 }, &b, &c);
    assert_eq!(a.get(), 2);
}

#[wasm_bindgen_test]
fn reference_as_first_argument_works2() {
    let a = Cell::new(0);
    pass_reference_first_arg_twice2(
        RefFirstArgument { contents: 3 },
        &mut |x: &RefFirstArgument| {
            assert_eq!(a.get(), 0);
            assert_eq!(x.contents, 3);
            a.set(a.get() + 1);
        },
        &mut |x: &RefFirstArgument| {
            assert_eq!(a.get(), 1);
            assert_eq!(x.contents, 3);
            a.set(a.get() + 1);
        },
    );
    assert_eq!(a.get(), 2);
}

#[wasm_bindgen_test]
fn call_destroyed_doesnt_segfault() {
    struct A(i32, i32);
    impl Drop for A {
        fn drop(&mut self) {
            assert_eq!(self.0, self.1);
        }
    }

    let a = A(1, 1);
    let a = Closure::wrap(Box::new(move || drop(&a)) as Box<dyn Fn()>);
    let b = a.as_ref().clone();
    drop(a);
    call_destroyed(&b);

    let a = A(2, 2);
    let a = Closure::wrap(Box::new(move || drop(&a)) as Box<dyn FnMut()>);
    let b = a.as_ref().clone();
    drop(a);
    call_destroyed(&b);

    let a = A(1, 1);
    let a = Closure::wrap(Box::new(move |_: &JsValue| drop(&a)) as Box<dyn Fn(&JsValue)>);
    let b = a.as_ref().clone();
    drop(a);
    call_destroyed(&b);

    let a = A(2, 2);
    let a = Closure::wrap(Box::new(move |_: &JsValue| drop(&a)) as Box<dyn FnMut(&JsValue)>);
    let b = a.as_ref().clone();
    drop(a);
    call_destroyed(&b);
}

#[wasm_bindgen_test]
fn forget_works() {
    let a = Closure::wrap(Box::new(|| {}) as Box<dyn Fn()>);
    js_store_forgotten_closure(&a);
    a.forget();
    js_call_forgotten_closure();
}

#[wasm_bindgen_test]
fn named_externref_no_duplicate_adapter() {
    externref_call(&Closure::new(|a| assert_eq!(a, 1)));
    named_externref_call(&Closure::new(|a| assert_eq!(a, 1)));
}
