use super::*;

#[cfg(test)]
mod tests {
    use anyhow::Error;

    use super::*;

    fn match_result(result: Result<bool, Error>, expected: bool) {
        match result {
            Ok(r) => assert_eq!(r, expected),
            Err(_) => panic!(""),
        }
    }

    #[test]
    fn match_single_character() {
        let result = match_pattern("apple", "a");
        match_result(result, true);
    }

    #[test]
    fn match_digit() {
        let result = match_pattern("apple123", "\\d");
        match_result(result, true);
    }

    #[test]
    fn match_no_digit() {
        let result = match_pattern("apple", "\\d");
        match_result(result, false);
    }

    #[test]
    fn match_alphanumeric() {
        let result = match_pattern("apple", "\\w");
        match_result(result, true);
    }

    #[test]
    fn match_no_alphanumeric() {
        let result = match_pattern("$!?", "\\w");
        match_result(result, false);
    }

    #[test]
    fn match_positive_match_group() {
        let result = match_pattern("apple", "[abc]");
        match_result(result, true);
    }

    #[test]
    fn match_no_positive_match_group() {
        let result = match_pattern("apple", "[bcd]");
        match_result(result, false);
    }

    #[test]
    fn match_negative_match_group() {
        let result = match_pattern("apple", "[^abc]");
        match_result(result, true);
    }

    #[test]
    fn match_no_negative_match_group() {
        let result = match_pattern("cab", "[^abc]");
        match_result(result, false);
    }

    #[test]
    fn match_combined_character_classes() {
        let result = match_pattern("3 dogs", "\\d \\w\\w\\ws");
        match_result(result, true);
    }

    #[test]
    fn match_start_anchor() {
        let result = match_pattern("log", "^log");
        match_result(result, true);
    }

    #[test]
    fn match_no_start_anchor() {
        let result = match_pattern("slog", "^log");
        match_result(result, false);
    }

    #[test]
    fn match_end_anchor() {
        let result = match_pattern("dog", "dog$");
        match_result(result, true);
    }

    #[test]
    fn match_no_end_anchor() {
        let result = match_pattern("dogs", "dog$");
        match_result(result, false);
    }

    #[test]
    fn match_one_or_more_times() {
        let result = match_pattern("SaaS", "a+");
        match_result(result, true);
    }

    #[test]
    fn match_no_one_or_more_times() {
        let result = match_pattern("dog", "a+");
        match_result(result, false);
    }

    #[test]
    fn match_zero_or_one_time() {
        let result = match_pattern("dog", "dogs?");
        match_result(result, true);
    }
}
