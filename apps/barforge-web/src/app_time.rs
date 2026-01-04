use time::OffsetDateTime;

pub(crate) fn app_now() -> OffsetDateTime {
    #[cfg(test)]
    if let Some(value) = test_now() {
        return value;
    }

    app_now_fallback()
}

fn app_now_fallback() -> OffsetDateTime {
    #[cfg(target_arch = "wasm32")]
    {
        let millis = js_sys::Date::now();
        let seconds = (millis / 1000.0).floor() as i64;
        OffsetDateTime::from_unix_timestamp(seconds).unwrap_or(OffsetDateTime::UNIX_EPOCH)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        OffsetDateTime::now_utc()
    }
}

#[cfg(test)]
mod tests {
    use super::{app_now, clear_test_now, set_test_now};
    use time::OffsetDateTime;
    use time::format_description::well_known::Rfc3339;

    #[test]
    fn uses_test_now_override_when_set() {
        let expected = OffsetDateTime::parse("2025-12-29T00:00:00Z", &Rfc3339).expect("valid time");
        set_test_now(expected);
        let now = app_now();
        clear_test_now();
        assert_eq!(now.unix_timestamp(), expected.unix_timestamp());
    }
}

#[cfg(test)]
fn test_now() -> Option<OffsetDateTime> {
    let guard = TEST_NOW
        .get_or_init(|| std::sync::Mutex::new(None))
        .lock()
        .ok()?;
    *guard
}

#[cfg(test)]
fn set_test_now(value: OffsetDateTime) {
    let mut guard = TEST_NOW
        .get_or_init(|| std::sync::Mutex::new(None))
        .lock()
        .expect("lock");
    *guard = Some(value);
}

#[cfg(test)]
fn clear_test_now() {
    let mut guard = TEST_NOW
        .get_or_init(|| std::sync::Mutex::new(None))
        .lock()
        .expect("lock");
    *guard = None;
}

#[cfg(test)]
static TEST_NOW: std::sync::OnceLock<std::sync::Mutex<Option<OffsetDateTime>>> =
    std::sync::OnceLock::new();
