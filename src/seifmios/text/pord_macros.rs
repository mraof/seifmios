macro_rules! pointer_ord {
    ($s:ident) => {
        // Implement on the type
        impl PartialEq for $s {
            fn eq(&self, other: &Self) -> bool {
                self as *const Self == other as *const Self
            }

            fn ne(&self, other: &Self) -> bool {
                self as *const Self != other as *const Self
            }
        }

        impl Eq for $s {}

        impl PartialOrd for $s {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                (self as *const Self).partial_cmp(&(other as *const Self))
            }

            fn lt(&self, other: &Self) -> bool {
                (self as *const Self) < (other as *const Self)
            }

            fn le(&self, other: &Self) -> bool {
                self as *const Self <= other as *const Self
            }

            fn gt(&self, other: &Self) -> bool {
                self as *const Self > other as *const Self
            }

            fn ge(&self, other: &Self) -> bool {
                self as *const Self >= other as *const Self
            }
        }

        impl Ord for $s {
            fn cmp(&self, other: &Self) -> Ordering {
                (self as *const Self).cmp(&(other as *const Self))
            }
        }
    };
}
