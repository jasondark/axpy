#![no_std]

/// Exposes linear combinations of slice-like objects of Copy values to LLVM's auto-vectorizer,
/// a.k.a. write vector expressions as you would in Matlab or Fortran.
///
/// Linear combinations of vectors don't on their own lend themselves to nice optimizations. For
/// example, consider `x+y+z`. Since the operator overloads are binary, this naively maps to
/// two two for-loops: one for `temp = x+y` and another for `result=temp+c*z`. The classic
/// solution is to employ "expression templates" which are effectively values representing lazy
/// operations, to be evaluated when encountering an assignment statement or when otherwise useful.
/// The C++ library Eigen is an excellent library implementing this approach, but as anybody who
/// has used it knows, there is a lot of magic going on that can lead to incomprehensible error
/// messages.
///
/// As a simple alternative (for this restricted set of operations), we provide a macro that
/// converts a linear combination to a canonical Rust representation that is amenable to LLVM's
/// auto-vectorizer. That is, the macro converts statements like `z = a*x + b*y + c*z` to
///     for (z, (x, y)) in z.iter_mut().zip(x.iter().zip(y.iter())) {
///         *z = a * *x + b * *y + c * *z;
///     }
///
/// If `x`, `y`, and `z` are slices, bounds-checks are known to be elided, resulting in fairly
/// optimal code. The value of the macro is that any combination-like expression is generated, e.g.
/// `w = 2.0 * x - z` becomes
///     for (x, (z, w)) in x.iter().zip(z.iter().zip(w.iter_mut())) {
///         *w = 2.0 * *x - *z;
///     }
///
/// In addition to `=`, both `+=` and `-=` are supported. (Technically *any* assignment operator
/// works, e.g. `/=`, but that is an accident of implementation rather than an intended feature.)
/// The assigned variable may appear anywhere in the constructed expression, as the macro is
/// designed to take appropriate care of the mutable borrow. Coefficients may be compatible scalar
/// literals or variables.


#[macro_export]
macro_rules! axpy {
    // point of entry to the macro: we immediately hand the input off to the parser (prefix=!)
    // `+ .` is used as terminal indicator
    [$y:ident $assign:tt $($rest:tt)+] => { axpy![! $y $assign () $($rest)* + .] };


    // parser rules: recursively perform the following transformations to the tokens
    // +? x     =>   1 * x
    // -  x     =>  -1 * x
    // +? a * x =>   a * x
    // -  a * x =>  -a * x
    // implementation note: 3 tokens are required to fully disambiguate the patterns,
    //                      that's why we seemingly peel back unnecessary tokens.
    [! $y:ident $assign:tt ($($parsed:tt)*)   $x:ident + $($rest:tt)+]       => // "x + ..."
        { axpy![! $y $assign ($($parsed)*     0 + $x) + $($rest)*] };
    [! $y:ident $assign:tt ($($parsed:tt)*)   $x:ident - $($rest:tt)+]       => // "x - ..."
        { axpy![! $y $assign ($($parsed)*     0 + $x) - $($rest)*] };
    [! $y:ident $assign:tt ($($parsed:tt)*) + $x:ident + $($rest:tt)+]       => // "+ x + ..."
        { axpy![! $y $assign ($($parsed)*     0 + $x) + $($rest)*] };
    [! $y:ident $assign:tt ($($parsed:tt)*) + $x:ident - $($rest:tt)+]       => // "+ x - ..."
        { axpy![! $y $assign ($($parsed)*     0 + $x) - $($rest)*] };
    [! $y:ident $assign:tt ($($parsed:tt)*) - $x:ident + $($rest:tt)+]       => // "- x + ..."
        { axpy![! $y $assign ($($parsed)*     0 - $x) + $($rest)*] };
    [! $y:ident $assign:tt ($($parsed:tt)*) - $x:ident - $($rest:tt)+]       => // "- x - ..."
        { axpy![! $y $assign ($($parsed)*     0 - $x) - $($rest)*] };
    [! $y:ident $assign:tt ($($parsed:tt)*)   $a:tt * $x:ident $($rest:tt)+] => // "a * x ..."
        { axpy![! $y $assign ($($parsed)*    $a * $x) $($rest)*] };
    [! $y:ident $assign:tt ($($parsed:tt)*) + $a:tt * $x:ident $($rest:tt)+] => // "+ a * x ..."
        { axpy![! $y $assign ($($parsed)*    $a * $x) $($rest)*] };
    [! $y:ident $assign:tt ($($parsed:tt)*) - $a:tt * $x:ident $($rest:tt)+] => // "- a * x ..."
        { axpy![! $y $assign ($($parsed)* (-$a) * $x) $($rest)*] };

    // upon conclusion of parsing, we hand off to iterator construction
    // (prefix=@) and expression constructor (prefix=#)
    [! $y:ident $assign:tt ($($parsed:tt)+) + .] => {
        for (car,cdr) in axpy![@ $y; $y.iter_mut(); $($parsed)*] {
            *car $assign axpy![# $y; car; cdr; () $($parsed)*];
        }
    };


    // iterator construction: we need to emit a zipped
    // iterator for x != y, and do nothing when x = y
    // (since y has already been borrowed mutably)
    [@ $y:ident; $iter:expr; ] => { $iter.map(|x| (x,)) };
    [@ $y:ident; $iter:expr; $a:tt $op:tt $x:ident $($rest:tt)*] => {
        {
            macro_rules! eval {
                ($y $y) => { axpy![@ $y; $iter; $($rest)*] };
                ($x $y) => { $iter.zip(axpy![@ $y; $x.iter(); $($rest)*]) };
            }
            eval!($x $y)
        }
    };


    // within the linear combination expression, we need to replace each vector
    // with the correct combination of obj.1. ... .1.0, e.g. peel back the zip()'s.

    // Base case: when done, emit new expression
    [# $y:ident; $car:ident; $cdr:expr; (+ $($parsed:tt)+)] => { $($parsed)* };

    // Case: + x
    [# $y:ident; $car:ident; $cdr:expr; ($($parsed:tt)*) 0 + $x:ident $($rest:tt)*] => {
        {
            macro_rules! eval {
                ($y $y) => { axpy![# $y; $car; $cdr  ; ($($parsed)* + *$car  ) $($rest)*] };
                ($x $y) => { axpy![# $y; $car; $cdr.1; ($($parsed)* + *$cdr.0) $($rest)*] };
            }
            eval!($x $y)
        }
    };
    // Case: - x
    [# $y:ident; $car:ident; $cdr:expr; ($($parsed:tt)*) 0 - $x:ident $($rest:tt)*] => {
        {
            macro_rules! eval {
                ($y $y) => { axpy![# $y; $car; $cdr  ; ($($parsed)* + - *$car  ) $($rest)*] };
                ($x $y) => { axpy![# $y; $car; $cdr.1; ($($parsed)* + - *$cdr.0) $($rest)*] };
            }
            eval!($x $y)
        }
    };
    // Case: + a * x
    [# $y:ident; $car:ident; $cdr:expr; ($($parsed:tt)*) $a:tt * $x:ident $($rest:tt)*] => {
        {
            macro_rules! eval {
                ($y $y) => { axpy![# $y; $car; $cdr  ; ($($parsed)* + $a * *$car  ) $($rest)*] };
                ($x $y) => { axpy![# $y; $car; $cdr.1; ($($parsed)* + $a * *$cdr.0) $($rest)*] };
            }
            eval!($x $y)
        }
    };

}

