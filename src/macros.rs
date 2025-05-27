#[macro_export]
/// define an "enum" (struct) of a singular type
/// ```
/// def_const_type_enum! (pub enumName => Type {
///     field1 => value,
///     field2 => value,
/// });
macro_rules! def_const_type_enum {
    ($vis:vis $name:ident => $ty:ty {
        $($variant:ident => $val:expr),+
        $(,)?
    }) => {
        #[non_exhaustive]
        $vis struct $name;

        impl $name {
            $(
                pub const $variant: $ty = $val;
            )+

            #[allow(dead_code)]
            pub const VARIANTS: &'static [$ty] = &[$(Self::$variant),+];
        }
    };
}

#[macro_export]
/// define an "enum" (struct) of mutliple variable types
/// ```
/// def_const_type_enum! (pub enumName {
///     field1: Type => value,
///     field2: Type => value,
/// });
macro_rules! def_varied_type_enum {
    ($vis:vis $name:ident {
        $($variant:ident : $ty:ty => $val:expr),+
        $(,)?
    }) => {
        #[non_exhaustive]
        $vis struct $name;

        impl $name {
            $(
                pub const $variant: $ty = $val;
            )+

            #[allow(dead_code)]
            pub const VARIANTS: &'static [(&'static str, &'static dyn std::any::Any)] = &[
                $((stringify!($variant), &$val as &dyn std::any::Any)),+
            ];
        }
    };
}
