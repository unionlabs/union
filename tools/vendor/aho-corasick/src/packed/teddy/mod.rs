#[cfg(target_arch = "x86_64")]
pub use crate::packed::teddy::compile::Builder;
#[cfg(not(target_arch = "x86_64"))]
pub use crate::packed::teddy::fallback::Builder;
#[cfg(not(target_arch = "x86_64"))]
pub use crate::packed::teddy::fallback::Teddy;
#[cfg(target_arch = "x86_64")]
pub use crate::packed::teddy::runtime::Teddy;

#[cfg(target_arch = "x86_64")]
mod compile;
#[cfg(target_arch = "x86_64")]
mod runtime;

#[cfg(not(target_arch = "x86_64"))]
mod fallback {
    use crate::packed::pattern::Patterns;
    use crate::Match;

    #[derive(Clone, Debug, Default)]
    pub struct Builder(());

    impl Builder {
        pub fn new() -> Builder {
            Builder(())
        }

        pub fn build(&self, _: &Patterns) -> Option<Teddy> {
            None
        }

        pub fn fat(&mut self, _: Option<bool>) -> &mut Builder {
            self
        }

        pub fn avx(&mut self, _: Option<bool>) -> &mut Builder {
            self
        }
    }

    #[derive(Clone, Debug)]
    pub struct Teddy(());

    impl Teddy {
        pub fn find_at(
            &self,
            _: &Patterns,
            _: &[u8],
            _: usize,
        ) -> Option<Match> {
            None
        }

        pub fn minimum_len(&self) -> usize {
            0
        }

        pub fn heap_bytes(&self) -> usize {
            0
        }
    }
}
