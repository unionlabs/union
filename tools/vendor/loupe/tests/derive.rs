use loupe::{size_of_val, MemoryUsage, POINTER_BYTE_SIZE};

macro_rules! assert_size_of_val_eq {
    ($expected:expr, $value:expr) => {
        assert_eq!($expected, size_of_val(&$value),);
    };
}

#[test]
fn test_struct_flat() {
    #[derive(MemoryUsage)]
    struct Point {
        x: i32,
        y: i32,
    }

    assert_size_of_val_eq!(8, Point { x: 1, y: 2 });
}

#[test]
fn test_struct_field_ignored() {
    #[derive(MemoryUsage)]
    struct S {
        x: Vec<i32>,
        y: Vec<i32>,
    }

    #[derive(MemoryUsage)]
    #[allow(unused)]
    struct T {
        x: Vec<i32>,
        #[loupe(skip)]
        y: Vec<i32>,
    }

    assert_size_of_val_eq!(
        72,
        S {
            x: vec![1, 2, 3],
            y: vec![1, 2, 3]
        }
    );
    assert_size_of_val_eq!(
        60,
        T {
            x: vec![1, 2, 3],
            y: vec![1, 2, 3]
        }
    );
}

#[test]
fn test_tuple() {
    #[derive(MemoryUsage)]
    struct Tuple(i32, i32);

    assert_size_of_val_eq!(8, Tuple(1, 2));

    #[derive(MemoryUsage)]
    #[repr(transparent)]
    struct Ptr(*const usize);

    assert_size_of_val_eq!(8, Ptr(&1));
}

#[test]
fn test_struct_with_generic() {
    #[derive(MemoryUsage)]
    struct Generic<T>
    where
        T: MemoryUsage,
    {
        x: T,
        y: T,
    }

    assert_size_of_val_eq!(16, Generic { x: 1i64, y: 2i64 });
}

#[test]
fn test_struct_with_inlined_generic() {
    #[derive(MemoryUsage)]
    struct Generic<T: MemoryUsage> {
        x: T,
        y: T,
    }

    assert_size_of_val_eq!(16, Generic { x: 1i64, y: 2i64 });
}

#[test]
fn test_struct_empty() {
    #[derive(MemoryUsage)]
    struct Empty;

    assert_size_of_val_eq!(0, Empty);
}

#[test]
fn test_struct_padding() {
    // This struct is packed in order <x, z, y> because 'y: i32' requires 32-bit
    // alignment but x and z do not. It starts with bytes 'x...yyyy' then adds 'z' in
    // the first place it fits producing 'xz..yyyy' and not 12 bytes 'x...yyyyz...'.
    #[derive(MemoryUsage)]
    struct Padding {
        x: i8,
        y: i32,
        z: i8,
    }

    assert_size_of_val_eq!(8, Padding { x: 1, y: 2, z: 3 });
}

#[test]
fn test_enum() {
    #[derive(MemoryUsage)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[derive(MemoryUsage)]
    enum Things {
        A,
        B(),
        C(i32),
        D { x: i32 },
        E(i32, i32),
        F { x: i32, y: i32 },
        Points(Vec<Point>),
    }

    assert_size_of_val_eq!(32, Things::A);
    assert_size_of_val_eq!(32, Things::B());
    assert_size_of_val_eq!(32, Things::C(1));
    assert_size_of_val_eq!(32, Things::D { x: 1 });
    assert_size_of_val_eq!(32, Things::E(1, 2));
    assert_size_of_val_eq!(32, Things::F { x: 1, y: 2 });

    assert_size_of_val_eq!(8, Point { x: 1, y: 2 });
    assert_size_of_val_eq!(40, vec![Point { x: 1, y: 2 }, Point { x: 3, y: 4 }]);
    assert_size_of_val_eq!(
        48,
        Things::Points(vec![Point { x: 1, y: 2 }, Point { x: 3, y: 4 }])
    );
}

#[test]
fn test_enum_with_generic() {
    #[derive(MemoryUsage)]
    enum Generic<T>
    where
        T: MemoryUsage,
    {
        A(T),
        B(T),
    }

    assert_size_of_val_eq!(16, Generic::<i64>::A(1));
    assert_size_of_val_eq!(16, Generic::<i64>::B(2));
}

#[test]
fn test_enum_with_inlined_generic() {
    #[derive(MemoryUsage)]
    enum Generic<T: MemoryUsage> {
        A(T),
        B(T),
    }

    assert_size_of_val_eq!(16, Generic::<i64>::A(1));
    assert_size_of_val_eq!(16, Generic::<i64>::B(2));
}

#[test]
fn test_enum_variant_ignored() {
    #[derive(MemoryUsage)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[derive(MemoryUsage)]
    enum E {
        A,
        Points(Vec<Point>),
    }

    assert_size_of_val_eq!(24, E::A);
    assert_size_of_val_eq!(
        40,
        E::Points(vec![Point { x: 1, y: 2 }, Point { x: 3, y: 4 }])
    );

    #[derive(MemoryUsage)]
    #[allow(unused)]
    enum F {
        A,
        #[loupe(skip)]
        Points(Vec<Point>),
    }

    assert_size_of_val_eq!(24, F::A);
    assert_size_of_val_eq!(
        24,
        F::Points(vec![Point { x: 1, y: 2 }, Point { x: 3, y: 4 }])
    );
}

#[test]
fn test_ptr() {
    #[derive(MemoryUsage)]
    #[repr(C)]
    struct X(u8);

    #[derive(MemoryUsage)]
    #[repr(transparent)]
    struct P(*const X);

    #[derive(MemoryUsage)]
    struct Q {
        ptr: *const X,
    }

    let x = X(42);
    let ptr = P(&x as *const _);
    assert_size_of_val_eq!(POINTER_BYTE_SIZE, ptr);

    let ptr = Q {
        ptr: &x as *const _,
    };
    assert_size_of_val_eq!(POINTER_BYTE_SIZE, ptr);
}
