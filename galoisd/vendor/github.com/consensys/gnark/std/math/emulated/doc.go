/*
Package emulated implements operations over any modulus.

# Non-native computation in circuit

Usually, the computations in a SNARK circuit are performed in the 'native' field
of the curve. The native field is defined by the scalar field of the underlying
curve. This package implements non-native arithmetic on top of the native field
to emulate operations in any field and ring.

This package does this by splitting the element into smaller limbs. The
parameters for splitting the limb and defining the modulus are stored in types
implementing [FieldParams] type. The elements are parametrized by those types to
make compile-time distinction between different emulated fields.

This package defines [Element] type which stores the element value in split
limbs. On top of the Element instance, this package defines typical arithmetic
as addition, multiplication and subtraction. If the modulus is a prime (i.e.
defines a finite field), then inversion and division operations are also
possible.

The results of the operations are not always reduced to be less than the
modulus. For consecutive operations it is necessary to manually reduce the value
using [Field.Reduce] method. The number of operations which can be performed
without reduction depends when the operations result starts overflowing the
limbs.

# Element representation

We operate in the scalar field of the SNARK curve (native field). Denote the
modulus of the native field as 'q'. Representing the modulus of the native field
requires 'n' bits. We wish to emulate operations over modulus 'r'. Modulus r may
or may not be a prime. If r is not prime, then we do not have inversion and
division operations (the corresponding methods panic). Let the bitlength of r be
'm'. We note that r may be smaller, larger or equal to q.

To represent an element x ∈ N_r, we choose the limb width 'w' such that

	w ≤ (m-1)/2

and write its integer representation as

	x = ∑_{i=0}^k x_i 2^{w i}.

Here, the variable 'x_i' is the w bits of x starting from 2^{w i}, 'k' is the
number of limbs and is computed as

	k = (n+w-1)/w,   // NB! integer division

and 'i' is the limb index. In this representation the element is represented in
little-endian (least significant limbs first) order. We do not necessarily
require that the limb values x_i are less than 2^w. This may happen if the limb
values are obtained as a result of arithmetic operations between elements. If we
know that the limb values do not overflow 2^w, then we say that the element is
in normal form.

In the implementation, we have two functions for splitting an element into limbs
and composing an element from limbs -- [decompose] and [recompose]. The
[recompose] function also accepts element in non-normal form.

# Elements in non-normal form

When an element is initialized, the limbs are in normal form, i.e. the values of
the limbs have bitwidth strictly less than w. As addition and multiplication are
performed on limbs natively, then the bitwidths of the limbs of the result may
be larger than w. We track the number of bits which may exceed the initial width
of the limbs. We denote the number of such excess bits as 'f' and call it
overflow. The total maximal bitwidth of the limbs is then

	w+f.

Keep in mind that parameter w is global for all emulated elements and f is
individual for every individual element.

To compute the overflow for the operations, we consider the arithmetic
operations which affect the overflow. In this implementation only addition is
done natively (limb-wise addition). When adding two elements, the bitwidth of
the result is up to one bit wider than the width of the widest element.

In the context of overflows, if the overflows of the addends are f_0 and f_1
then the overflow value f' for the sum is computed as

	f' = max(f_0, f_1)+1.

# Multiplication

The complexity of native limb-wise multiplication is k^2. This translates
directly to the complexity in the number of constraints in the constraint
system.

For multiplication, we would instead use polynomial representation of the
elements:

	x = ∑_{i=0}^k x_i 2^{w i}
	y = ∑_{i=0}^k y_i 2^{w i}.

as

	x(X) = ∑_{i=0}^k x_i X^i
	y(X) = ∑_{i=0}^k y_i X^i.

If the multiplication result modulo r is c, then the following holds:

	x * y = c + z*r.

We can check the correctness of the multiplication by checking the following
identity at a random point:

	x(X) * y(X) = c(X) + z(X) * r(X) + (2^w' - X) e(X),

where e(X) is a polynomial used for carrying the overflows of the left- and
right-hand side of the above equation.

# Subtraction

We perform subtraction limb-wise between the elements x and y. However, we have
to ensure than any limb in the result does not result in overflow, i.e.

	x_i ≥ y_i, ∀ 0≤i<k.

As this does not hold in general, then we need to pad x such that every limb x_i
is strictly larger than y_i.

The additional padding 'u' has to be divisible by the emulated modulus r and
every limb u_i must be larger than x_i-y_i. Let f' be the overflow of y. We
first compute the limbs u'_i as

	u'_i = 1 << (w+f'), ∀ 0≤i<k.

Now, as u' is not divisible by r, we need to compensate for it:

	u'' = u' + regroup(r - (u' % r)),

where regroup() regroups the argument so that it is in normal form (i.e. first
applies recompose() and then decompose() method).

We see that u” is now such that it is divisible by r and its every limb is
larger than every limb of b. The subtraction is performed as

	z_i = x_i + u''_i - y_i, ∀ 0≤i<k.

# Equality checking

Equality checking is performed using modular multiplication. To check that a, b
are equal modulo r, we compute

	diff = b-a,

and enforce modular multiplication check using the techniques for modular
multiplication:

	diff * 1 = 0 + k * r.

# Bitwidth enforcement

When element is computed using hints, we need to ensure that every limb is not
wider than k bits. For that, we perform bitwise decomposition of every limb and
check that k lower bits are equal to the whole limb. We omit the bitwidth
enforcement for multiplication as the correctness of the limbs is ensured using
the corresponding system of linear equations.

Additionally, we apply bitwidth enforcement for elements initialized from
integers.

# Modular reduction

To reduce the integer value of element x to be less than the modulus, we compute
the remainder x' of x modulo r using hint, enforce the bitwidth of x' and assert
that

	x' == x

using element equality checking.

# Values computed using hints

We additionally define functions for computing inverse of an element and ratio
of two elements. Both function compute the actual value using hint and then
assert the correctness of the operation using multiplication.

# Constant values

The package currently does not explicitly differentiate between constant and
variable elements. The builder may track some elements as being constants. Some
operations have a fast track path for cases when all inputs are constants. There
is [Field.MulConst], which provides variable by constant multiplication.

# Variable-modulus operations

The package also exposes methods for performing operations with variable
modulus. The modulus is represented as an element and is not required to be
prime. The methods for variable-modulus operations are [Field.ModMul],
[Field.ModAdd], [Field.ModExp] and [Field.ModAssertIsEqual]. The modulus is
passed as an argument to the operation.

The type parameter for the [Field] should be sufficiently big to allow to fit
the inputs and the modulus. Recommended to use predefined [emparams.Mod1e512] or
[emparams.Mod1e4096].
*/
package emulated
