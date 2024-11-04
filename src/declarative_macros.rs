#[macro_export]
macro_rules! make_public {
    ($(#[$($attrib_macro:tt)+])* struct $name:ident { $($(#[$($field_attrib_macro:tt)+])* $field_name:ident: $field_type:ty,)* }) => {
        $(#[$($attrib_macro)+])*
        pub struct $name {
            $(
                $(#[$($field_attrib_macro)+])*
                pub $field_name: $field_type,
            )*
        }
    };
}
