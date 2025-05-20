pub(crate) static HTTP_USER_AGENT: &str =
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

pub(crate) trait SplitPrefixInclusive {
    fn split_prefix_inclusive<'a>(&'a self, prefix: &str) -> Vec<&'a str>;
}

impl SplitPrefixInclusive for str {
    /// Split string by prefix, including the prefix in the result.
    fn split_prefix_inclusive<'a>(&'a self, prefix: &str) -> Vec<&'a str> {
        let matches = self.match_indices(prefix).map(|(idx, _)| idx);
        let mut start = 0;
        let mut substrings = Vec::new();
        for idx in matches {
            if idx != start {
                substrings.push(&self[start..idx]);
                start = idx;
            }
        }
        substrings.push(&self[start..]);

        substrings
    }
}

/// Finds the file name from a diff. The diff is expected to be of the form
/// "diff --git a/file_name b/file_name".
///
/// If the diff is not of the expected form, then None is returned.
pub(crate) fn get_file_name_from_diff(file_diff: &str) -> Option<&str> {
    let (_, suffix) = file_diff.split_once("diff --git ")?;
    let mut parts = suffix.split_whitespace();
    let _old = parts.next()?;
    let new = parts.next()?;
    new.strip_prefix("b/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_prefix_inclusive() {
        let string = include_str!("../tests/data/example_1.diff");
        let pattern = "diff --git ";
        assert_eq!(string.split_prefix_inclusive(pattern).len(), 5);
    }

    #[test]
    fn test_basic_split_prefix_inclusive() {
        let string = "x111x222x333";
        let pattern = "x";
        assert_eq!(string.split_prefix_inclusive(pattern).len(), 3);
        assert_eq!(
            string.split_prefix_inclusive(pattern),
            &["x111", "x222", "x333"]
        );
    }

    #[test]
    fn test_basic_split_prefix_inclusive_2() {
        let string = "x111\nx222\nx333";
        let pattern = "\nx";
        assert_eq!(string.split_prefix_inclusive(pattern).len(), 3);
        assert_eq!(
            string.split_prefix_inclusive(pattern),
            &["x111", "\nx222", "\nx333"]
        );
    }
    #[test]
    fn test_get_file_name_from_diff() {
        assert_eq!(get_file_name_from_diff(""), None);
        assert_eq!(get_file_name_from_diff("asdasdas"), None);
        assert_eq!(get_file_name_from_diff("diff --git a/"), None);
        assert_eq!(get_file_name_from_diff("diff --git b/"), None);
        assert_eq!(
            get_file_name_from_diff(
                &r#"
diff --git a/foo b/foo
new file mode 100644
index 0000000..a51b2a6
--- /dev/null
+++ b/foo
@@ -0,0 +1 @@
+sadasdas
"#[1..]
            ),
            Some("foo")
        );

        assert_eq!(
            get_file_name_from_diff(
                "diff --git a/old_name b/new_name\n--- a/old_name\n+++ b/new_name"
            ),
            Some("new_name")
        );
    }
}
