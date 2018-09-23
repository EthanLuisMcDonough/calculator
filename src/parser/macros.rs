#[macro_export]
macro_rules! simple_enum {
    (#[derive($($der:ident),*)] $enum_name:ident { $($field_name:ident),* }) => {
        #[derive($($der),*)]
        pub enum $enum_name {
            $( $field_name , )*
        }

        impl $enum_name {
            pub const VALUES: &'static [$enum_name] = &[ $( $enum_name::$field_name , )* ];
        }
    };
}

macro_rules! internal_mac_var {
    ({ $map:ident } { $i:expr } $v:expr) => {
        $map.insert($i, VariableValue::Constant($v));
    };
    ({ $map:ident } { $i:expr } fn($float:ident, $mode:ident) $b:block) => {{
        fn apply($float: f64, $mode: AngleMode) -> f64 $b
        $map.insert($i, VariableValue::Function(Box::new(|f, m| apply(f, m))));
    }};
    ({ $map:ident } { $i:expr } fn($float:ident) $b:block) => {{
        fn apply($float: f64) -> f64 $b
        $map.insert($i, VariableValue::Function(Box::new(|f, _| apply(f))));
    }};
    ({ $map:ident } { $i:expr } fn(rad ! $float:ident) $b:block) => {{
        fn apply($float: f64) -> f64 $b

        fn rad_apply(arg: f64, mode: AngleMode) -> f64 {
            apply(if mode.is_deg() { arg.to_radians() } else { arg })
        }

        $map.insert($i, VariableValue::Function(Box::new(|f, m| rad_apply(f, m))));
    }};
}

macro_rules! var_map {
    ( $( $k:expr => { $($t:tt)* } ),* ) => {
        {
            let mut map = HashMap::new();
            $(
                internal_mac_var!({ map } {{ stringify![$k] }} $($t)*);
            )*
            map
        }
    };
}
