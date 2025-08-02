use crate::number::math;
use super::lua_value::LuaValue;

fn iadd(a: i64, b: i64) -> i64 {
    a + b
}

fn fadd(a: f64, b: f64) -> f64 {
    a + b
}

fn isub(a: i64, b: i64) -> i64 {
    a - b
}
fn fsub(a: f64, b: f64) -> f64 {
    a - b
}
fn imul(a: i64, b: i64) -> i64 {
    a * b
}
fn fmul(a: f64, b: f64) -> f64 {
    a * b
}
fn imod(a: i64, b: i64) -> i64 {
    math::IMod(a, b)
}
fn fmod(a: f64, b: f64) -> f64 {
    math::FMod(a, b)
}
fn pow(a: f64, b: f64) -> f64 {
    a.powf(b)
}
fn div(a: f64, b: f64) -> f64 {
    a / b
}
fn iidiv(a: i64, b: i64) -> i64 {
    math::IFloorDiv(a, b)
}
fn fidiv(a: f64, b: f64) -> f64 {
    math::FFloorDiv(a, b)
}
fn band(a: i64, b: i64) -> i64 {
    a & b
}
fn bor(a: i64, b: i64) -> i64 {
    a | b
}
fn bxor(a: i64, b: i64) -> i64 {
    a ^ b
}
fn shl(a: i64, b: i64) -> i64 {
    math::ShiftLeft(a, b)
}
fn shr(a: i64, b: i64) -> i64 {
    math::ShiftRight(a, b)
}
fn iunm(a: i64, _: i64) -> i64 {
    -a
}
fn funm(a: f64, _: f64) -> f64 {
    -a
}
fn bnot(a: i64, _: i64) -> i64 {
    !a
}

fn inone(_: i64, _: i64) -> i64 {
    0
}
fn fnone(_: f64, _: f64) -> f64 {
    0.0
}

pub const OPERATORS: &'static [(&'static str, fn(i64, i64) -> i64, fn(f64, f64) -> f64)] = &[
    ("__add", iadd, fadd),
    ("__sub", isub, fsub),
    ("__mul", imul, fmul),
    ("__mod", imod, fmod),
    ("__pow", inone, pow),
    ("__div", inone, div),
    ("__idiv", iidiv, fidiv),
    ("__band", band, fnone),
    ("__bor", bor, fnone),
    ("__bxor", bxor, fnone),
    ("__shl", shl, fnone),
    ("__shr", shr, fnone),
    ("__unm", iunm, funm),
    ("__bnot", bnot, fnone),
];

pub fn arith(a: &LuaValue, b: &LuaValue, op: u8) -> Option<LuaValue> {
    let iop = OPERATORS[op as usize].1;
    let fop = OPERATORS[op as usize].2;
    if fop == fnone {
        // bitwise
        if let Some(x) = a.ToInteger() {
            if let Some(y) = b.ToInteger() {
                return Some(LuaValue::Integer(iop(x, y)));
            }
        }
    } else {
        // arith
        if iop != inone {
            if let LuaValue::Integer(x) = a {
                if let LuaValue::Integer(y) = b {
                    return Some(LuaValue::Integer(iop(*x, *y)));
                }
            }
        }
        if let Some(x) = a.ToFloat() {
            if let Some(y) = b.ToFloat() {
                return Some(LuaValue::Number(fop(x, y)));
            }
        }
    }
    None
}