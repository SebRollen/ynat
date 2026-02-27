macro_rules! setter {
    ($field:ident : $ty:ty) => {
        pub fn $field<T>(mut self, $field: T) -> Self
        where
            T: Into<$ty>,
        {
            self.$field = $field.into();
            self
        }
    };

    ($field:ident . $subfield:ident : $ty:ty) => {
        pub fn $subfield<T>(mut self, $subfield: T) -> Self
        where
            T: Into<$ty>,
        {
            self.$field.$subfield = $subfield.into();
            self
        }
    };

    (opt $field:ident : $ty:ty) => {
        pub fn $field<T>(mut self, $field: T) -> Self
        where
            T: Into<$ty>,
        {
            self.$field = std::option::Option::Some($field.into());
            self
        }
    };

    (opt $field:ident . $subfield:ident : $ty:ty) => {
        pub fn $subfield<T>(mut self, $subfield: T) -> Self
        where
            T: Into<$ty>,
        {
            self.$field.$subfield = Some($subfield.into());
            self
        }
    };
}

pub(crate) use setter;
