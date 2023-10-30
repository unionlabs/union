extern crate downcast;

// careful: do not combine downcast_sync! with downcast::Any, you will get `size not known at compile time` errors
use downcast::{downcast_sync, AnySync};
use std::sync::Arc;

/* Trait */

trait Service: AnySync {
    fn what_am_i(&self);
}

downcast_sync!(dyn Service);

/* Impl */

struct Database {}

impl Service for Database {
    fn what_am_i(&self){
        println!("I'm a database!");
    }
}

impl Database {
    fn purge_data(&self) {
        println!("Database has been purged! Goodbye, data!")
    }
}

fn main(){
    let service: Arc<dyn Service> = Arc::new(Database{});
    service.what_am_i();
    {
        let db = service.downcast_ref::<Database>().unwrap();
        db.purge_data();
    }
    let db: Arc<Database> = service.downcast_arc::<Database>().ok().unwrap();
    db.purge_data();
}

