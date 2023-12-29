mod multi_extractor;

#[derive(Clone)]
pub enum A {
    B,
    C,
}

struct It {
    i: u32,
    arg: u32,
}

impl multi_extractor::ContextIter for It {
    type Context = Context;
    type Output = (A, u32);
    fn next(&mut self, _ctx: &mut Self::Context) -> Option<Self::Output> {
        if self.i >= 32 {
            None
        } else {
            let idx = self.i;
            self.i += 1;
            let a = if self.arg & (1u32 << idx) != 0 {
                A::B
            } else {
                A::C
            };
            Some((a, idx))
        }
    }
}

struct Context;
impl multi_extractor::Context for Context {
    type e1_etor_iter = It;
    fn e1_etor(&mut self, arg0: u32) -> Option<It> {
        Some(It { i: 0, arg: arg0 })
    }
}

fn main() {
    let mut ctx = Context;
    let x = multi_extractor::constructor_Rule(&mut ctx, 0xf0);
    let y = multi_extractor::constructor_Rule(&mut ctx, 0);
    println!("x = {:?} y = {:?}", x, y);
}
