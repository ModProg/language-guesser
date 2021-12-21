use serde::{de::Visitor, Deserialize};

/// A struct used for deserializing a map into a Vec of it's keys using serde
pub struct DeserializeKeys(pub Vec<String>);

impl<'de> Deserialize<'de> for DeserializeKeys {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Vis;

        impl<'de> Visitor<'de> for Vis {
            type Value = DeserializeKeys;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a map with string keys.")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut vec = vec![];
                while let Some((key, VoidDeserialize)) = map.next_entry()? {
                    println!("{}", &key);
                    vec.push(key);
                }

                Ok(DeserializeKeys(vec))
            }
        }

        deserializer.deserialize_map(Vis)
    }
}

// A type that can be deserialized from any data but doesn't save it.
// Useful for where something needs to be deserialized to correctly drive a deserializer but isn't
// needed.
pub struct VoidDeserialize;

impl<'de> Deserialize<'de> for VoidDeserialize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Vis;
        impl<'de> Visitor<'de> for Vis {
            type Value = VoidDeserialize;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("anything.")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                // need to empty the map as to not get invalid length errors
                while let Some((VoidDeserialize, VoidDeserialize)) = map.next_entry()? {}

                Ok(VoidDeserialize)
            }

            fn visit_i64<E>(self, _v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(VoidDeserialize)
            }

            fn visit_i128<E>(self, _v: i128) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(VoidDeserialize)
            }

            fn visit_u64<E>(self, _v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(VoidDeserialize)
            }
            fn visit_u128<E>(self, _v: u128) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(VoidDeserialize)
            }
            fn visit_str<E>(self, _v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(VoidDeserialize)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                // need to empty the seq as to not get invalid length errors
                while let Some(VoidDeserialize) = seq.next_element()? {}

                Ok(VoidDeserialize)
            }

            fn visit_char<E>(self, _v: char) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(VoidDeserialize)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(VoidDeserialize)
            }

            fn visit_some<D>(self, _deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Ok(VoidDeserialize)
            }
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(VoidDeserialize)
            }

            fn visit_bytes<E>(self, _v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(VoidDeserialize)
            }

            fn visit_bool<E>(self, _v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(VoidDeserialize)
            }

            fn visit_enum<A>(self, _data: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::EnumAccess<'de>,
            {
                Ok(VoidDeserialize)
            }

            fn visit_newtype_struct<D>(self, _deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Ok(VoidDeserialize)
            }
        }

        deserializer.deserialize_any(Vis)
    }
}
