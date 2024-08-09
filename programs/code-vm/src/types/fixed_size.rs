pub trait FixedSize {
    fn to_fixed(&self, size: usize) -> Vec<u8>;
}

impl FixedSize for String {
    fn to_fixed(&self, size: usize) -> Vec<u8> {
        let val = fixed_str(&self, size);
        val.as_bytes().to_vec()
    }
}

/// Returns a padded string with the given length
fn pad_str(s: &str, length: usize) -> String {
    let mut padded = String::from(s);
    while padded.len() < length {
        padded.push('\0');
    }
    padded
}

/// Returns a string with the given length
fn trunc_str(s: &str, length: usize) -> String {
    if s.len() <= length {
        return String::from(s);
    }
    s.chars().take(length).collect()
}

/// Returns a string with the given length
fn fixed_str(s: &str, length: usize) -> String {
    if s.len() >= length {
        return trunc_str(s, length);
    }
    pad_str(s, length)
}