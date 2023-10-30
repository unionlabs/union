#!/usr/bin/env bash

gen_impl() {
	local len=$1
	cat <<-END
		impl<T: Default> Array for [T; $len] {
		  type Item = T;
		  const CAPACITY: usize = $len;

		  #[inline(always)]
		  #[must_use]
		  fn as_slice(&self) -> &[T] {
		    &*self
		  }

		  #[inline(always)]
		  #[must_use]
		  fn as_slice_mut(&mut self) -> &mut [T] {
		    &mut *self
		  }

		  #[inline(always)]
		  fn default() -> Self {
		    [
		$(for ((i = 0; i < $len; i += 6))
		do
			echo -n '     '
			for ((j = 0; j < 6 && j + i < $len; j++))
			do
				echo -n ' T::default(),'
			done
			echo
		done)
		    ]
		  }
		}

		END
}

cat <<-END
	// Generated file, to regenerate run
	//     ./gen-array-impls.sh > src/array/generated_impl.rs
	// from the repo root

	use super::Array;

	$(for ((i = 0; i <= 33; i++)); do gen_impl $i; done)

	$(for ((i = 64; i <= 4096; i *= 2)); do gen_impl $i; done)
END

# vim: noet
