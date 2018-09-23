#[test]
fn eval_radian_mode() {
    use super::{eval_math, to_fixed, AngleMode};

    let fixed = 7u32;

    let tests = vec![
        ("3 + 3 ^ 2", Ok(12.0)),
        ("5 + 3 -", Err("Incomplete expression".into())),
        ("3floor(2.4) + 2", Ok(8.0)),
        ("  1 + 2*   3 ^4+   5 ", Ok(168.0)),
        ("3pi3pi", Ok(88.8264396)),
        ("8E3 / 100 + 30", Ok(110.0)),
        ("sqrt(4E4) / (3 - 1)", Ok(100.0)),
        ("3 .20", Err("Unexpected character \'.\' at index 2".into())),
        ("7 + (3) + 3e2", Ok(26.3096910)),
        ("1 + abs(3 + 2 * -20 - 2) + 3 / 2", Ok(41.5)),
        ("3 + abs - 2", Err("Unexpected operator -".into())),
        ("3 + () / 2", Err("Empty parentheses".into())),
        ("3 + (4 + ((3)) * 3", Err("Incomplete expression".into())),
        ("((((((((3))))) + 4))) - 1 * 2", Ok(5.0)),
        ("((5((2 / 3)3))sqrt(4) + 2) ^ 2 + 2sin(pi) ^ 3", Ok(484.0)),
        ("3+-4*2", Ok(-5.0)),
        ("0*0---e", Ok(-2.7182818)),
        ("-2^2", Ok(-4.0)),
        ("-2*-2", Ok(4.0)),
        ("-(3 + 2)4", Ok(-20.0)),
        ("3-*2", Err("Unexpected character \'*\' at index 2".into())),
        ("3*--abs(-2)", Ok(6.0)),
    ];

    for (expr, result) in tests.into_iter() {
        assert_eq!(
            eval_math(expr, AngleMode::Rad).map(|f| to_fixed(f, fixed)),
            result
        );
    }
}
