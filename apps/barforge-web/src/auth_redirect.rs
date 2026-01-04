pub(crate) fn encode_query_value(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b' ' => encoded.push('+'),
            b'/' => encoded.push('/'),
            _ if is_unreserved(byte) => encoded.push(byte as char),
            _ => {
                encoded.push('%');
                encoded.push_str(&format!("{:02X}", byte));
            }
        }
    }
    encoded
}

pub(crate) fn login_redirect_url(target: &str) -> String {
    let encoded = encode_query_value(target);
    format!("/login?redirect_to={encoded}")
}

pub(crate) fn sanitize_redirect_target(redirect_to: Option<&str>) -> Option<&str> {
    redirect_to.filter(|value| is_safe_redirect(value))
}

pub(crate) fn is_safe_redirect(value: &str) -> bool {
    value.starts_with('/') && !value.starts_with("//") && !value.contains("://")
}

fn is_unreserved(byte: u8) -> bool {
    matches!(byte, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~')
}

#[cfg(test)]
mod tests {
    use super::encode_query_value;

    #[test]
    fn encode_query_value_preserves_slashes_and_plus_encodes_spaces() {
        let encoded = encode_query_value("/dashboard with spaces");

        assert_eq!(encoded, "/dashboard+with+spaces");
    }
}
