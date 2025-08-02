use super::math::FloatToInteger;

pub fn ParseInteger(s: &str) -> (i64, bool) {
    if let Ok(val) = s.parse::<i64>() {
        return (val, true);
    } else if let Ok(val) = s.parse::<f64>() {
        return FloatToInteger(val);
    } else {
        return (0, false);
    }
}

pub fn ParseFloat(s: &str) -> (f64, bool) {
    if let Ok(val) = s.parse::<f64>() {
        return (val, true);
    } else {
        return (0.0, false);
    }
}