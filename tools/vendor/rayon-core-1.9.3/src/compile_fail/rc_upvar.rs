/*! ```compile_fail,E0277

use std::rc::Rc;

fn main() {
    let r = Rc::new(22);
    rayon_core::join(|| r.clone(), || r.clone());
    //~^ ERROR
}

``` */
