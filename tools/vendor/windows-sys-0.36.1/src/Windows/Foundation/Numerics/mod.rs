#[repr(C)]
#[doc = "*Required features: `\"Foundation_Numerics\"`*"]
pub struct Matrix3x2 {
    pub M11: f32,
    pub M12: f32,
    pub M21: f32,
    pub M22: f32,
    pub M31: f32,
    pub M32: f32,
}
impl ::core::marker::Copy for Matrix3x2 {}
impl ::core::clone::Clone for Matrix3x2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Foundation_Numerics\"`*"]
pub struct Matrix4x4 {
    pub M11: f32,
    pub M12: f32,
    pub M13: f32,
    pub M14: f32,
    pub M21: f32,
    pub M22: f32,
    pub M23: f32,
    pub M24: f32,
    pub M31: f32,
    pub M32: f32,
    pub M33: f32,
    pub M34: f32,
    pub M41: f32,
    pub M42: f32,
    pub M43: f32,
    pub M44: f32,
}
impl ::core::marker::Copy for Matrix4x4 {}
impl ::core::clone::Clone for Matrix4x4 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Foundation_Numerics\"`*"]
pub struct Plane {
    pub Normal: Vector3,
    pub D: f32,
}
impl ::core::marker::Copy for Plane {}
impl ::core::clone::Clone for Plane {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Foundation_Numerics\"`*"]
pub struct Quaternion {
    pub X: f32,
    pub Y: f32,
    pub Z: f32,
    pub W: f32,
}
impl ::core::marker::Copy for Quaternion {}
impl ::core::clone::Clone for Quaternion {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Foundation_Numerics\"`*"]
pub struct Rational {
    pub Numerator: u32,
    pub Denominator: u32,
}
impl ::core::marker::Copy for Rational {}
impl ::core::clone::Clone for Rational {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Foundation_Numerics\"`*"]
pub struct Vector2 {
    pub X: f32,
    pub Y: f32,
}
impl ::core::marker::Copy for Vector2 {}
impl ::core::clone::Clone for Vector2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Foundation_Numerics\"`*"]
pub struct Vector3 {
    pub X: f32,
    pub Y: f32,
    pub Z: f32,
}
impl ::core::marker::Copy for Vector3 {}
impl ::core::clone::Clone for Vector3 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Foundation_Numerics\"`*"]
pub struct Vector4 {
    pub X: f32,
    pub Y: f32,
    pub Z: f32,
    pub W: f32,
}
impl ::core::marker::Copy for Vector4 {}
impl ::core::clone::Clone for Vector4 {
    fn clone(&self) -> Self {
        *self
    }
}
