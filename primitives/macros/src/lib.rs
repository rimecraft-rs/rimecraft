/// Creates a new [`rimecraft_primitives::Id`].
#[macro_export]
macro_rules! id {
    ($ns:expr, $p:expr) => {
        {
            rimecraft_primitives::Id::new($ns.to_string(), $p.to_string())
        }
    };
    ($ns:expr, $p:expr, $( $pp:expr ),*) => {
        {
            let mut __temp_path = String::new();
            __temp_path.push_str($p.as_ref());
            $(
                __temp_path.push('/');
                __temp_path.push_str($pp.as_ref());
            )*
            rimecraft_primitives::Id::new($ns.to_string(), __temp_path)
        }
    };
    ($n:expr) => {
        {
            rimecraft_primitives::Id::parse($n.as_ref())
        }
    }
}
