
//* We represent pointers to Scheme values as static references. This has a few
//* implications:
//*    1. simple implementation
//*    2. naive allocation (=no GC) will leak lots of memory
//*    3. using an explicit garbage collector may be unsound if there is
//*       nothing that prevents such references to be put in e.g. a Box, where
//*       the GC cannot find them; resulting in a dangling reference
//*    4. best strategy is possibly to put the Boehm GC as Rust's global
//*       allocator. Then the GC manages all allocations and Box,Vec,etal
//*       are safe to use.

#[macro_use]
extern crate criterion;

use criterion::Criterion;
use criterion::black_box;


fn integer_performance(c: &mut Criterion) {
    c.bench_function("simple fib 20", |b| b.iter(|| fibonacci(black_box(make_int(20)))));
}

fn fibonacci(n: Scm) -> Scm {
    if as_integer(n).unwrap() < 2 {
        make_int(1)
    } else {
        let a = as_integer(fibonacci(make_int(as_integer(n).unwrap() - 1))).unwrap();
        let b = as_integer(fibonacci(make_int(as_integer(n).unwrap() - 2))).unwrap();
        make_int(a + b)
    }
}


fn pair_performance(c: &mut Criterion) {
    c.bench_function("simple reverse", |b| b.iter(|| reverse(make_list(black_box(10000)))));
}

fn make_list(len: usize) -> Scm {
    let mut list = make_scm(ScmValue::Nil);
    for i in (0..len).rev() {
        list = cons(make_int(i as i64), list);
    }
    list
}

fn reverse(list: Scm) -> Scm {
    if is_null(list) {
        make_scm(ScmValue::Nil)
    } else {
        cons(reverse(cdr(list).unwrap()), car(list).unwrap())
    }
}


pub type Scm = &'static ScmValue;

pub enum ScmValue {
    Nil,
    Integer(i64),
    Pair(Scm, Scm),
}

fn make_scm(value: ScmValue) -> Scm {
    Box::leak(Box::new(value))
}

fn make_int(i: i64) -> Scm {
    make_scm(ScmValue::Integer(i))
}

pub fn cons(car: Scm, cdr: Scm) -> Scm {
    make_scm(ScmValue::Pair(car, cdr))
}

pub fn car(scm: Scm) -> Option<Scm> {
    match scm {
        ScmValue::Pair(car, _) => Some(car),
        _ => None,
    }
}

pub fn cdr(scm: Scm) -> Option<Scm> {
    match scm {
        ScmValue::Pair(_, cdr) => Some(cdr),
        _ => None,
    }
}

pub fn is_pair(scm: Scm) -> bool {
    match scm {
        ScmValue::Pair(_, _) => true,
        _ => false,
    }
}

pub fn as_integer(scm: Scm) -> Option<i64> {
    match scm {
        ScmValue::Integer(i) => Some(*i),
        _ => None,
    }
}

pub fn is_integer(scm: Scm) -> bool {
    match scm {
        ScmValue::Integer(_) => true,
        _ => false,
    }
}

pub fn is_null(scm: Scm) -> bool {
    match scm {
        ScmValue::Nil => true,
        _ => false,
    }
}

#[test]
fn integer_vs_pointers() {
    for i in 0..10 {
        let x = make_int(i);
        let p = cons(x, x);

        assert!(is_integer(x));
        assert!(!is_pair(x));
        assert!(is_pair(p));
        assert!(!is_integer(p));
    }
}


criterion_group!(benches, integer_performance, pair_performance);
criterion_main!(benches);
