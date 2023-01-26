/// TODO make trait of string
/// Split string by prefix, including the prefix in the result.
pub(crate) fn split_prefix_inclusive<'a>(string: &'a str, prefix: &str) -> Vec<&'a str> {
    let matches = string.match_indices(prefix).map(|(idx, _)| idx);
    let mut start = 0;
    let mut substrings = Vec::new();
    for idx in matches {
        if idx != start {
            substrings.push(&string[start..idx]);
            start = idx;
        }
    }
    substrings.push(&string[start..]);

    substrings
}

/// Finds the file name from a diff. The diff is expected to be of the form
/// "diff --git a/file_name b/file_name".
///
/// If the diff is not of the expected form, then None is returned.
pub(crate) fn get_file_name_from_diff(file_diff: &str) -> Option<&str> {
    let (_, suffix) = file_diff.split_once("diff --git a/")?;
    let (file_name, _) = suffix.split_once(' ')?;
    Some(file_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_prefix_inclusive() {
        let string = include_str!("../tests/data/example_1.diff");
        let pattern = "diff --git ";
        assert_eq!(split_prefix_inclusive(string, pattern).len(), 5);
    }

    #[test]
    fn test_basic_split_prefix_inclusive() {
        let string = "x111x222x333";
        let pattern = "x";
        assert_eq!(split_prefix_inclusive(string, pattern).len(), 3);
        assert_eq!(
            split_prefix_inclusive(string, pattern),
            &["x111", "x222", "x333"]
        );
    }

    #[test]
    fn test_basic_split_prefix_inclusive_2() {
        let string = "x111\nx222\nx333";
        let pattern = "\nx";
        assert_eq!(split_prefix_inclusive(string, pattern).len(), 3);
        assert_eq!(
            split_prefix_inclusive(string, pattern),
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
    }
}
