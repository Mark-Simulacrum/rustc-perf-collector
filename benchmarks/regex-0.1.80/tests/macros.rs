// Convenience macros.

macro_rules! findall {
    ($re:expr, $text:expr) => {{
        $re.find_iter(text!($text)).collect::<Vec<_>>()
    }}
}

// Macros for automatically producing tests.

macro_rules! ismatch {
    ($name:ident, $re:expr, $text:expr, $ismatch:expr) => {
        #[test]
        fn $name() {
            let text = text!($text);
            let re = regex!($re);
            assert!($ismatch == re.is_match(text));
        }
    };
}

macro_rules! mat(
    ($name:ident, $re:expr, $text:expr, $($loc:tt)+) => (
        #[test]
        fn $name() {
            let text = text!($text);
            let expected: Vec<Option<_>> = vec![$($loc)+];
            let r = regex!($re);
            let got: Vec<Option<_>> = match r.captures(text) {
                Some(c) => {
                    assert!(r.is_match(text));
                    assert!(r.shortest_match(text).is_some());
                    c.iter_pos().collect()
                }
                None => vec![None],
            };
            // The test set sometimes leave out capture groups, so truncate
            // actual capture groups to match test set.
            let mut sgot = &got[..];
            if sgot.len() > expected.len() {
                sgot = &sgot[0..expected.len()]
            }
            if expected != sgot {
                panic!("For RE '{}' against '{:?}', \
                        expected '{:?}' but got '{:?}'",
                       $re, text, expected, sgot);
            }
        }
    );
);

macro_rules! matiter(
    ($name:ident, $re:expr, $text:expr) => (
        #[test]
        fn $name() {
            let text = text!($text);
            let expected: Vec<(usize, usize)> = vec![];
            let r = regex!($re);
            let got: Vec<_> = r.find_iter(text).collect();
            if expected != got {
                panic!("For RE '{}' against '{:?}', \
                        expected '{:?}' but got '{:?}'",
                       $re, text, expected, got);
            }
            let captures_got: Vec<_> =
                r.captures_iter(text).map(|c| c.pos(0).unwrap()).collect();
            if captures_got != got {
                panic!("For RE '{}' against '{:?}', \
                        got '{:?}' using find_iter but got '{:?}' \
                        using captures_iter",
                       $re, text, got, captures_got);
            }
        }
    );
    ($name:ident, $re:expr, $text:expr, $($loc:tt)+) => (
        #[test]
        fn $name() {
            let text = text!($text);
            let expected: Vec<_> = vec![$($loc)+];
            let r = regex!($re);
            let got: Vec<_> = r.find_iter(text).collect();
            if expected != got {
                panic!("For RE '{}' against '{:?}', \
                        expected '{:?}' but got '{:?}'",
                       $re, text, expected, got);
            }
            let captures_got: Vec<_> =
                r.captures_iter(text).map(|c| c.pos(0).unwrap()).collect();
            if captures_got != got {
                panic!("For RE '{}' against '{:?}', \
                        got '{:?}' using find_iter but got '{:?}' \
                        using captures_iter",
                       $re, text, got, captures_got);
            }
        }
    );
);

macro_rules! matset {
    ($name:ident, $res:expr, $text:expr, $($match_index:expr),*) => {
        #[test]
        fn $name() {
            let text = text!($text);
            let set = regex_set!($res);
            assert!(set.is_match(text));
            let expected = vec![$($match_index),*];
            let matches = set.matches(text);
            assert!(matches.matched_any());
            let got: Vec<_> = matches.into_iter().collect();
            assert_eq!(expected, got);
        }
    }
}

macro_rules! nomatset {
    ($name:ident, $res:expr, $text:expr) => {
        #[test]
        fn $name() {
            let text = text!($text);
            let set = regex_set!($res);
            assert!(!set.is_match(text));
            let matches = set.matches(text);
            assert!(!matches.matched_any());
            assert_eq!(0, matches.into_iter().count());
        }
    }
}

macro_rules! split {
    ($name:ident, $re:expr, $text:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let re = regex!($re);
            let splitted: Vec<_> = re.split(t!($text)).collect();
            assert_eq!($expected, &*splitted);
        }
    }
}
