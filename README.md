# AXPY

An macro-based alternative to expression templates for efficient n-ary linear combinations of slice-like objects, i.e. objects that implement `.iter()` and `.iter_mut()`. Compiled with optimizations, resulting source code elides bound checks and will be auto-vectorized by LLVM.

## Examples

    #[macro_use] extern crate axpy;
    fn test(a: f64, x: &[f64], y: &[f64], z: &mut [f64]) {
        // some random expression
        axpy![z = a * x + z - 2.*y];

        // this becomes:
        // for (z, (x, y)) in z.iter_mut().zip(x.iter().zip(y.iter())) {
        //     *z = a * *x + *z - 2. * *y;
        // }
    }

Virtually any "reasonable" linear combination of any number of vectors (up to the compiler macro recursion limit) is permitted, along with other assignment statements, e.g. `+=` or `-=` in addition to `=`. The assigned variable may freely appear anywhere in the expression, permitting in-place modifications without auxiliary variables. Refer to the source code for more information -- as far as macro code goes, it is fairly well commented.

## License

Licensed under
* Apache License 2.0, or
* MIT License, or
* BSD 2-Clause License,
at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be tri-licensed as above, without any additional terms or conditions.

## Acknowledgments

* [static-cond](https://github.com/durka/static-cond) was how I learned to do token equality matching, which was used in this code to permit the assigned variable appearing throughout the expression.

