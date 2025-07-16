#[cfg(test)]
mod tests {
    use derive_csv_friendly::CsvFriendly;

    #[derive(CsvFriendly)]
    struct Foo {
        a: String,
        b: Vec<String>,
    }

    #[test]
    fn test_instantiate() {
        FooCsvFriendly {
            a: String::from("foo"),
            b: String::from("bar"),
        };
    }

    #[test]
    fn test_convert_single_val() {
        let foo = Foo {
            a: String::from("foo"),
            b: vec![String::from("bar")],
        };

        let friendly: FooCsvFriendly = foo.into();
        assert_eq!(friendly.a, "foo");
        assert_eq!(friendly.b, "bar");

        let foo_1: Foo = friendly.into();
        assert_eq!(foo_1.a, "foo");
        assert_eq!(foo_1.b, vec![String::from("bar")]);
    }

    #[test]
    fn test_convert_multiple_val() {
        let foo = Foo {
            a: String::from("foo"),
            b: vec![String::from("bar"), String::from("baz")],
        };

        let friendly: FooCsvFriendly = foo.into();
        assert_eq!(friendly.b, "bar|baz");

        let foo_1: Foo = friendly.into();
        assert_eq!(foo_1.b, vec![String::from("bar"), String::from("baz")]);
    }

    #[test]
    fn test_convert_empty_str() {
        let foo = Foo {
            a: String::from("foo"),
            b: vec![String::from("")],
        };

        let friendly: FooCsvFriendly = foo.into();
        assert_eq!(friendly.b, "");

        let foo_1: Foo = friendly.into();
        assert_eq!(foo_1.b, vec![String::from("")]);
    }

    #[test]
    fn test_convert_with_empty_str() {
        let foo = Foo {
            a: String::from("foo"),
            b: vec![String::from(""), String::from("bar"), String::from("")],
        };

        let friendly: FooCsvFriendly = foo.into();
        assert_eq!(friendly.b, "|bar|");

        let foo_1: Foo = friendly.into();
        assert_eq!(foo_1.b, vec![String::from(""), String::from("bar"), String::from("")]);
    }

    #[test]
    #[should_panic]
    fn test_convert_empty() {
        let foo = Foo {
            a: String::from("foo"),
            b: vec![],
        };

        let _friendly: FooCsvFriendly = foo.into();
    }

    #[test]
    #[should_panic]
    fn test_convert_bad_name() {
        let foo = Foo {
            a: String::from("foo"),
            b: vec![String::from("ba|r")],
        };

        let _friendly: FooCsvFriendly = foo.into();
    }
}
