extern crate downcast;
use downcast::{downcast, downcast_sync, Any, AnySync};
use std::sync::Arc;

trait Simple: Any {}
downcast!(dyn Simple);

trait WithParams<T, U>: Any {}
downcast!(<T, U> dyn WithParams<T, U>);
struct Param1;
struct Param2;

struct ImplA {
    data: String
}
impl Simple for ImplA {}
impl WithParams<Param1, Param2> for ImplA {}

struct ImplB;
impl Simple for ImplB {}
impl WithParams<Param1, Param2> for ImplB {}

#[test]
fn simple(){
    let mut a: Box<dyn Simple> = Box::new(ImplA{ data: "data".into() });

    assert_eq!(a.downcast_ref::<ImplA>().unwrap().data, "data");
    assert!(a.downcast_ref::<ImplB>().is_err());

    assert_eq!(a.downcast_mut::<ImplA>().unwrap().data, "data");
    assert!(a.downcast_mut::<ImplB>().is_err());

    assert_eq!(a.downcast::<ImplA>().unwrap().data, "data");
}

#[test]
fn with_params(){
    let mut a: Box<dyn WithParams<Param1, Param2>> = Box::new(ImplA{ data: "data".into() });

    assert_eq!(a.downcast_ref::<ImplA>().unwrap().data, "data");
    assert!(a.downcast_ref::<ImplB>().is_err());

    assert_eq!(a.downcast_mut::<ImplA>().unwrap().data, "data");
    assert!(a.downcast_mut::<ImplB>().is_err());

    assert_eq!(a.downcast::<ImplA>().unwrap().data, "data");
}

trait SimpleSync: AnySync {}
downcast_sync!(dyn SimpleSync);

impl SimpleSync for ImplA {}
impl SimpleSync for ImplB {}

#[test]
fn simple_sync(){
    let a: Arc<dyn SimpleSync> = Arc::new(ImplA{ data: "data".into() });

    assert_eq!(a.downcast_ref::<ImplA>().unwrap().data, "data");
    assert!(a.downcast_ref::<ImplB>().is_err());

    assert_eq!(a.downcast_arc::<ImplA>().unwrap().data, "data");
}

trait WithParamsSync<T, U>: AnySync {}
downcast_sync!(<T, U> dyn WithParamsSync<T, U>);
impl WithParamsSync<Param1, Param2> for ImplA {}
impl WithParamsSync<Param1, Param2> for ImplB {}

#[test]
fn with_params_sync() {
    let a: Arc<dyn WithParamsSync<Param1, Param2>> = Arc::new(ImplA{ data: "data".into() });

    assert_eq!(a.downcast_ref::<ImplA>().unwrap().data, "data");
    assert!(a.downcast_ref::<ImplB>().is_err());

    assert_eq!(a.downcast_arc::<ImplA>().unwrap().data, "data");
}
