/*
Package sw_emulated implements elliptic curve group operations in (short)
Weierstrass form.

The elliptic curve is the set of points (X,Y) satisfying the equation:

	Y² = X³ + aX + b

over some base field 𝐅p for some constants a, b ∈ 𝐅p.
Additionally, for every curve we also define its generator (base point) G. All
these parameters are stored in the variable of type [CurveParams].

This package implements unified and complete point addition. The method
[Curve.AddUnified] can be used for point additions or in case of points at
infinity. As such, this package does not expose separate Add and Double methods.

The package provides a few curve parameters, see functions [GetSecp256k1Params]
and [GetBN254Params].

Unconventionally, this package uses type parameters to define the base field of
the points and variables to define the coefficients of the curve. This is due to
how the emulated elements are constructed by their type parameters. To unify the
different conventions, we provide the method [GetCurveParams] to allow resolving
a particular curve parameter depending on the type parameter defining the base
field. For now, we only have a single curve defined on every base field, but
this may change in the future with the addition of additional curves.

This package uses field emulation (unlike packages
[github.com/consensys/gnark/std/algebra/native/sw_bls12377] and
[github.com/consensys/gnark/std/algebra/native/sw_bls24315], which use 2-chains). This
allows to use any curve over any native (SNARK) field. The drawback of this
approach is the extreme cost of the operations.
*/
package sw_emulated
