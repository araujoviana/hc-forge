pub fn normalize_obs_bucket_name(
    input: &str,
    min_len: usize,
    max_len: usize,
) -> Result<String, String> {
    let bucket = input.trim().to_ascii_lowercase();
    if bucket.len() < min_len || bucket.len() > max_len {
        return Err(format!(
            "OBS bucket name must be {}-{} characters.",
            min_len, max_len
        ));
    }
    if !bucket
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '.' || ch == '-')
    {
        return Err(
            "OBS bucket name can contain only lowercase letters, digits, '.' or '-'.".to_string(),
        );
    }
    if bucket.starts_with('.')
        || bucket.ends_with('.')
        || bucket.starts_with('-')
        || bucket.ends_with('-')
    {
        return Err("OBS bucket name cannot start or end with '.' or '-'.".to_string());
    }
    if bucket.contains("..") {
        return Err("OBS bucket name cannot contain consecutive dots.".to_string());
    }
    if bucket.parse::<std::net::Ipv4Addr>().is_ok() {
        return Err("OBS bucket name cannot be formatted as an IPv4 address.".to_string());
    }
    Ok(bucket)
}

pub fn normalize_obs_object_key(input: &str) -> Result<String, String> {
    let key = input.trim().trim_start_matches('/').to_string();
    if key.is_empty() {
        return Err("OBS object key is required.".to_string());
    }
    Ok(key)
}

pub fn normalize_ssh_session_id(input: &str) -> Result<String, String> {
    let session_id = input.trim();
    if session_id.is_empty() {
        return Err("SSH session ID is required.".to_string());
    }
    Ok(session_id.to_string())
}

pub fn control_char_from_input(input: &str) -> Result<u8, String> {
    let normalized = input.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "c" | "ctrl+c" => Ok(0x03),
        "d" | "ctrl+d" => Ok(0x04),
        "u" | "ctrl+u" => Ok(0x15),
        _ => Err(format!(
            "Unsupported control sequence '{}'. Use Ctrl+C, Ctrl+D, or Ctrl+U.",
            input.trim()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        control_char_from_input, normalize_obs_bucket_name, normalize_obs_object_key,
        normalize_ssh_session_id,
    };

    const OBS_BUCKET_MIN: usize = 3;
    const OBS_BUCKET_MAX: usize = 63;

    #[test]
    fn normalize_obs_bucket_name_accepts_and_lowercases_valid_names() {
        assert_eq!(
            normalize_obs_bucket_name("My-Bucket-001", OBS_BUCKET_MIN, OBS_BUCKET_MAX)
                .expect("bucket"),
            "my-bucket-001"
        );
    }

    #[test]
    fn normalize_obs_bucket_name_rejects_disallowed_formats() {
        assert!(normalize_obs_bucket_name("192.168.0.1", OBS_BUCKET_MIN, OBS_BUCKET_MAX).is_err());
        assert!(normalize_obs_bucket_name("bad_bucket", OBS_BUCKET_MIN, OBS_BUCKET_MAX).is_err());
        assert!(normalize_obs_bucket_name("a..b", OBS_BUCKET_MIN, OBS_BUCKET_MAX).is_err());
        assert!(
            normalize_obs_bucket_name("-starts-with-dash", OBS_BUCKET_MIN, OBS_BUCKET_MAX).is_err()
        );
    }

    #[test]
    fn normalize_obs_bucket_name_enforces_length_bounds() {
        assert!(normalize_obs_bucket_name("ab", OBS_BUCKET_MIN, OBS_BUCKET_MAX).is_err());
        let long_name = "a".repeat(OBS_BUCKET_MAX + 1);
        assert!(normalize_obs_bucket_name(&long_name, OBS_BUCKET_MIN, OBS_BUCKET_MAX).is_err());
        let min_ok = "abc";
        assert_eq!(
            normalize_obs_bucket_name(min_ok, OBS_BUCKET_MIN, OBS_BUCKET_MAX).expect("min len"),
            min_ok
        );
    }

    #[test]
    fn normalize_obs_object_key_trims_leading_slashes() {
        assert_eq!(
            normalize_obs_object_key("/logs/app.log").expect("key"),
            "logs/app.log"
        );
        assert_eq!(
            normalize_obs_object_key("///nested/path.txt").expect("key"),
            "nested/path.txt"
        );
    }

    #[test]
    fn normalize_obs_object_key_requires_non_empty_value() {
        assert!(normalize_obs_object_key("  ").is_err());
        assert!(normalize_obs_object_key("/").is_err());
    }

    #[test]
    fn normalize_ssh_session_id_trims_whitespace() {
        assert_eq!(
            normalize_ssh_session_id("  session-01  ").expect("session id"),
            "session-01"
        );
    }

    #[test]
    fn normalize_ssh_session_id_requires_non_empty_value() {
        assert!(normalize_ssh_session_id("   ").is_err());
    }

    #[test]
    fn control_char_from_input_maps_supported_shortcuts() {
        assert_eq!(control_char_from_input("ctrl+c").expect("ctrl+c"), 0x03);
        assert_eq!(control_char_from_input("c").expect("c"), 0x03);
        assert_eq!(control_char_from_input("D").expect("d"), 0x04);
        assert_eq!(control_char_from_input(" Ctrl+U ").expect("ctrl+u"), 0x15);
    }

    #[test]
    fn control_char_from_input_rejects_unknown_values() {
        assert!(control_char_from_input("ctrl+z").is_err());
    }
}
