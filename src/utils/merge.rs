pub fn merge_optional<T>(
    base: Option<&T>,
    override_val: Option<&T>,
    merge_fn: impl FnOnce(&T, &T) -> T,
) -> Option<T>
where
    T: Clone,
{
    match (base, override_val) {
        (None, None) => None,
        (Some(b), None) => Some(b.clone()),
        (None, Some(o)) => Some(o.clone()),
        (Some(b), Some(o)) => Some(merge_fn(b, o)),
    }
}

pub fn merge_optional_or_default<T>(
    base: Option<&T>,
    override_val: Option<&T>,
    merge_fn: impl FnOnce(&T, &T) -> T,
) -> T
where
    T: Clone + Default,
{
    merge_optional(base, override_val, merge_fn).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_optional_both_none() {
        let result: Option<i32> = merge_optional(None, None, |a, b| a + b);
        assert_eq!(result, None);
    }

    #[test]
    fn test_merge_optional_only_base() {
        let base = 5;
        let result = merge_optional(Some(&base), None, |a, b| a + b);
        assert_eq!(result, Some(5));
    }

    #[test]
    fn test_merge_optional_only_override() {
        let override_val = 10;
        let result = merge_optional(None, Some(&override_val), |a, b| a + b);
        assert_eq!(result, Some(10));
    }

    #[test]
    fn test_merge_optional_both_present() {
        let base = 5;
        let override_val = 10;
        let result = merge_optional(Some(&base), Some(&override_val), |a, b| a + b);
        assert_eq!(result, Some(15));
    }

    #[test]
    fn test_merge_optional_or_default_both_none() {
        let result: i32 = merge_optional_or_default(None, None, |a, b| a + b);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_merge_optional_or_default_with_values() {
        let base = 5;
        let override_val = 10;
        let result = merge_optional_or_default(Some(&base), Some(&override_val), |a, b| a + b);
        assert_eq!(result, 15);
    }
}
