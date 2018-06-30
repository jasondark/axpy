#![feature(trace_macros)]
trace_macros!(true);

#[macro_use] extern crate axpy;

#[test]
fn form_basic() {
    let x: [f64; 4] = [1.0, 2.0, 3.0, 4.0];
    let y: [f64; 4] = [4.0, 3.0, 2.0, 1.0];
    let mut z: [f64; 4] = [0., 0., 0., 0.];
    axpy![z = x + y];
    assert_eq!(z, [5f64, 5., 5., 5.]);
}

#[test]
fn form_advanced() {
    let x: [f64; 4] = [1.0, 2.0, 3.0, 4.0];
    let y: [f64; 4] = [4.0, 3.0, 2.0, 1.0];
    let mut z: [f64; 4] = [10., 100., 1000., 10000.];
    axpy![z = 2.*z - x + 3. * y];
    assert_eq!(z, [31f64, 207., 2003., 19999.]);
}
