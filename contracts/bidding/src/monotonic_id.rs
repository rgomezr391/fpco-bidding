#[macro_export]
macro_rules! impl_monotonic_id {
    ($name:ident, $key:literal, $doc:expr) => {
        #[doc = $doc]
        #[derive(
            Clone,
            Copy,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            serde::Serialize,
            serde::Deserialize,
            schemars::JsonSchema,
            Debug,
        )]
        pub struct $name(pub u32);

        
        impl $name {
            #[allow(dead_code)]
            const COUNTER: cw_storage_plus::Item<u32> =
                cw_storage_plus::Item::new($key);

            pub fn new(id: u32) -> Self {
                Self(id)
            }

            #[allow(dead_code)]
            pub fn next(storage: &mut dyn cosmwasm_std::Storage) -> cosmwasm_std::StdResult<Self> {
                let id = Self::COUNTER.load(storage).unwrap_or_default();
                Self::COUNTER.save(storage, &(id + 1))?;

                Ok(Self(id))
            }
        }

        impl<'a> cw_storage_plus::PrimaryKey<'a> for $name {
            type Prefix = ();
            type SubPrefix = ();
            type Suffix = u64;
            type SuperSuffix = u64;

            #[inline]
            fn key(&self) -> Vec<cw_storage_plus::Key> {
                use cw_storage_plus::IntKey;
                vec![cw_storage_plus::Key::Val32(self.0.to_cw_bytes())]

                
            }
        }

        impl<'a> cw_storage_plus::Prefixer<'a> for $name {
            fn prefix(&self) -> Vec<cw_storage_plus::Key> {
                use cw_storage_plus::IntKey;
                vec![cw_storage_plus::Key::Val32(self.0.to_cw_bytes())]
            }
        }

        impl cw_storage_plus::KeyDeserialize for $name {
            type Output = Self;
            const KEY_ELEMS: u16 = 1;

            fn from_vec(value: Vec<u8>) -> cosmwasm_std::StdResult<Self::Output> {
                Self::from_slice(value.as_slice())
            }

            fn from_slice(value: &[u8]) -> StdResult<Self::Output> {
                Ok(Self(u32::from_cw_bytes(slice_to_array(value)?)))
            }
        }

        impl std::fmt::Display for $name {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(&self.0, f)
            }
        }
    };
}
