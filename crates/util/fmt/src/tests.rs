use crate::Formatting;

#[test]
fn check() {
    for fmt in Formatting::VALUES {
        assert_eq!(*fmt, fmt.code().try_into().unwrap());
        assert_eq!(fmt.raw_name().to_ascii_lowercase(), fmt.name());
        assert_eq!(fmt.raw_name().parse::<Formatting>().unwrap(), *fmt);
        assert_eq!(fmt.to_string().parse::<Formatting>().unwrap(), *fmt);
    }
}
