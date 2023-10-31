extern crate downcast;

use downcast::{downcast, Any};
use std::fmt::Debug;

/* Trait */

trait Animal<X: Debug>: Any {
    fn what_am_i(&self);
    fn get_item(&self) -> Option<&X>;
}

downcast!(<X> dyn Animal<X> where X: Debug);

/* Impl */

struct Bird<X>{ item: Option<X> }

impl<X: Debug + 'static> Animal<X> for Bird<X> {
    fn what_am_i(&self){
        println!("Im a bird!")
    }
    fn get_item(&self) -> Option<&X> {
        match self.item {
            Some(ref item) => println!("I'm holding a {:?}! Look, see!", item),
            None => println!("I'm holding nothing!")
        }
        self.item.as_ref()
    }
}

impl<X: Debug + 'static> Bird<X> {
    fn eat_item(&mut self) {
        if self.item.is_some() {
            let item = self.item.take().unwrap();
            println!("I ate the {:?}! I hope it was edible!", item)
        } else {
            println!("I don't have anything to eat!")
        }
    }
}

/* Main */

fn main() {
    let mut animal: Box<dyn Animal<String>> = Box::new(Bird{ item: Some("haselnut".to_owned()) });
    animal.what_am_i();
    {
        let bird = animal.downcast_mut::<Bird<String>>().unwrap();
        bird.get_item();
        bird.eat_item();
    }
    let mut bird = animal.downcast::<Bird<String>>().ok().unwrap();
    bird.get_item();
    bird.eat_item();
}
