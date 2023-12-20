mod multi_constructor;

pub(crate) type ConstructorVec<T> = Vec<T>;

struct Context;

struct It {
    i: u32,
    limit: u32,
}

impl multi_constructor::ContextIter for It {
    type Context = Context;
    type Output = u32;
    fn next(&mut self, _ctx: &mut Self::Context) -> Option<u32> {
        if self.i >= self.limit {
            None
        } else {
            let i = self.i;
            self.i += 1;
            Some(i)
        }
    }
}

impl multi_constructor::Context for Context {
    type etor_C_iter = It;
    fn etor_C(&mut self, value: u32) -> Option<It> {
        Some(It { i: 0, limit: value })
    }

    type ctor_B_iter = multi_constructor::ContextIterWrapper<u32, std::vec::IntoIter<u32>, Context>;
    fn ctor_B(&mut self, value: u32) -> Option<Self::ctor_B_iter> {
        Some((0..value).rev().collect::<Vec<_>>().into_iter().into())
    }
}

struct IterWithContext<'a, Item, I: multi_constructor::ContextIter<Output = Item, Context = Context>> {
    ctx: &'a mut Context,
    it: I,
}

impl<'a, Item, I: multi_constructor::ContextIter<Output = Item, Context = Context>> Iterator for IterWithContext<'a, Item, I> {
    type Item = Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.it.next(self.ctx)
    }
}

fn main() {
    let mut ctx = Context;
    let l1 = multi_constructor::constructor_A(&mut ctx, 10).unwrap();
    let l2 = multi_constructor::constructor_D(&mut ctx, 5).unwrap();
    let l1 = IterWithContext { ctx: &mut ctx, it: l1 }.collect::<Vec<_>>();
    let l2 = IterWithContext { ctx: &mut ctx, it: l2 }.collect::<Vec<_>>();
    println!("l1 = {:?} l2 = {:?}", l1, l2);
}
