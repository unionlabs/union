extern crate downcast;

// we need to use downcast::Any instead of std::any::Any
use downcast::{downcast, Any};

/* Trait */

trait Animal: Any {
    fn what_am_i(&self);
}

downcast!(dyn Animal);

/* Impl */

struct Bird;

impl Animal for Bird {
    fn what_am_i(&self){
        println!("Im a bird!")
    }
}

impl Bird {
    fn wash_beak(&self) {
        println!("Beak has been washed! What a clean beak!");
    }
}

/* Main */

fn main() {
    let animal: Box<dyn Animal> = Box::new(Bird);
    animal.what_am_i();
    {
        let bird = animal.downcast_ref::<Bird>().unwrap();
        bird.wash_beak();
    }
    let bird: Box<Bird> = animal.downcast::<Bird>().ok().unwrap();
    bird.wash_beak();
}
