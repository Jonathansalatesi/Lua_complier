pub fn IFloorDiv(a: i64, b: i64) -> i64 {
    if (a > 0 && b > 0) || (a < 0 && b < 0) || (a % b == 0) {
        return a / b;
    } else {
        return a / b - 1;
    }
}

pub fn FFloorDiv(a: f64, b: f64) -> f64 {
    (a / b).floor()
}

pub fn IMod(a: i64, b: i64) -> i64 {
    a - IFloorDiv(a, b) * b
}

pub fn FMod(a: f64, b: f64) -> f64 {
    a - ((a / b).floor() * b)
}

pub fn ShiftLeft(a: i64, n: i64) -> i64 {
    if n >= 0 {
        return a << n;
    } else {
        return ShiftRight(a, -n);
    }
}

pub fn ShiftRight(a: i64, n: i64) -> i64 {
    if n >= 0 {
        return (a as u64 >> n) as i64;
    } else {
        return ShiftLeft(a, -n);
    }
}

pub fn FloatToInteger(f: f64) -> (i64, bool) {
    let i = f as i64;
    (i, f == (i as f64))
}

pub fn random() -> usize {
    let ptr = Box::into_raw(Box::new(123));
    ptr as usize
}
