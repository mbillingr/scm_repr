
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
    c.bench_function("cheapair fib 20", |b| b.iter(|| fibonacci(black_box(Scm::from_int(20)))));
}

#[inline(never)]
fn fibonacci(n: Scm) -> Scm {
    if n.as_integer().expect("int") < 2 {
        Scm::from_int(1)
    } else {
        let a = (fibonacci(Scm::from_int(n.as_integer().unwrap() - 1))).as_integer().unwrap();
        let b = (fibonacci(Scm::from_int(n.as_integer().unwrap() - 2))).as_integer().unwrap();
        Scm::from_int(a + b)
    }
}


fn pair_performance(c: &mut Criterion) {
    c.bench_function("cheapair reverse", |b| b.iter(|| reverse(make_list(black_box(10000)))));
}

fn make_list(len: usize) -> Scm {
    let mut list = Scm::nil();
    for i in (0..len).rev() {
        list = cons(Scm::from_int(i as i64), list);
    }
    list
}

fn reverse(list: Scm) -> Scm {
    if is_null(list) {
        Scm::nil()
    } else {
        cons(reverse(cdr(list).expect("pair")), car(list).expect("pair"))
    }
}


const N_TAG_BITS: usize = 2;
const TAG_MASK: usize = 0b_11;
const TAG_POINTER: usize = 0b_00;
const TAG_INTEGER: usize = 0b_01;
const TAG_PAIR: usize = 0b_10;
const TAG_SPECIAL: usize = 0b_11;

const SPECIAL_NIL: usize = 0b_0011;

const MASK_IMMEDIATE: usize = 0b01;  // this works because all immediates have 1 in the lsb

#[derive(Debug, Copy, Clone)]
pub struct Scm {
    value: usize,
}

impl Scm {
    fn new(value: ScmValue) -> Self {
        Scm {
            value: ref_to_addr(Box::leak(Box::new(value)))
        }
    }

    fn nil() -> Self {
        Scm {
            value: SPECIAL_NIL
        }
    }

    fn from_int(value: i64) -> Self {
        Scm {
            value: (value as usize) << N_TAG_BITS | TAG_INTEGER
        }
    }

    fn is_immediate(&self) -> bool {
        self.value & MASK_IMMEDIATE != 0
    }

    fn is_nil(&self) -> bool {
        self.value == SPECIAL_NIL
    }

    fn as_integer(&self) -> Option<i64> {
        if self.value & TAG_MASK == TAG_INTEGER {
            Some((self.value >> N_TAG_BITS) as i64)
        } else {
            None
        }
    }

    fn as_ref(&self) -> Option<&ScmValue> {
        if self.value & TAG_MASK == TAG_POINTER {
            unsafe {
                Some(int_to_ref(self.value))
            }
        } else {
            None
        }
    }

    fn as_pair(&self) -> Option<&(Scm, Scm)> {
        if self.value & TAG_MASK == TAG_PAIR {
            unsafe {
                Some(int_to_ref(self.value - TAG_PAIR))
            }
        } else {
            None
        }
    }
}

unsafe fn int_to_ref<T>(i: usize) -> &'static T {
    &*(i as *const T)
}

fn ref_to_addr<T>(r: &T) -> usize {
    r as *const T as usize
}

#[derive(Debug)]
#[repr(u64)]
pub enum ScmValue {
    Vector(&'static[Scm]),
}

pub fn cons(car: Scm, cdr: Scm) -> Scm {
    let r = Box::leak(Box::new((car, cdr)));
    let addr = r as *const _ as usize;
    debug_assert!(addr & TAG_MASK == 0);
    Scm {
        value: addr + TAG_PAIR
    }
}

pub fn car(scm: Scm) -> Option<Scm> {
    scm.as_pair().map(|p| p.0)
}

pub fn cdr(scm: Scm) -> Option<Scm> {
    scm.as_pair().map(|p| p.1)
}

pub fn is_pair(scm: Scm) -> bool {
    scm.as_pair().is_some()
}

pub fn is_integer(scm: Scm) -> bool {
    scm.as_integer().is_some()
}

pub fn is_null(scm: Scm) -> bool {
    scm.is_nil()
}

#[test]
fn integer_vs_pointers() {
    for i in 0..10 {
        let x = Scm::from_int(i);
        let p = cons(x, x);

        assert!(is_integer(x));
        assert!(!is_pair(x));
        assert!(is_pair(p));
        assert!(!is_integer(p));
    }
}

criterion_group!(benches, integer_performance, pair_performance);
criterion_main!(benches);
