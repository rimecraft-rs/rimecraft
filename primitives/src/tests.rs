mod id {
    use crate::Id;

    #[test]
    fn to_str() {
        let id = Id::new("modid", "example_path".to_string());
        assert_eq!(id.to_string(), "modid:example_path");
    }

    #[test]
    fn parse_str() {
        let raw = "modid:example_path";
        let id = Id::parse(raw);
        assert_eq!(id.to_string(), raw);
    }
}
