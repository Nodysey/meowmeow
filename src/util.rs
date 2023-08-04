// Borrowed from rust-pretty-bytes
pub fn bytes_to_readable(bytes: f64) -> String
{
    let neg = if bytes.is_sign_positive() {""} else {"-"};
    let bytes = bytes.abs();
    let units = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    if bytes < 1_f64 {
        return format!("{}{} {}", neg, bytes, "B");
    }

    let delimiter = 1000_f64;
    let exponent = std::cmp::min((bytes.ln() / delimiter.ln()).floor() as i32, (units.len() - 1) as i32);
    let readable = format!("{:.2}", bytes / delimiter.powi(exponent)).parse::<f64>().unwrap() * 1_f64;

    let unit = units[exponent as usize];
    return format!("{}{} {}", neg, readable, unit);
}