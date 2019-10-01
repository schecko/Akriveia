
use tokio_postgres::types::{ FromSql, IsNull, ToSql, Type, };
use std::error::Error;
use crate::ShortAddress;

impl<'a> FromSql<'a> for ShortAddress {
    fn from_sql(_: &Type, raw: &[u8]) -> Result<ShortAddress, Box<dyn Error + Sync + Send>> {
        if raw.len() != 2 {
            return Err("invalid message length".into());
        }
        let mut bytes = [0; 2];
        bytes.copy_from_slice(raw);
        Ok(ShortAddress::new(bytes))
    }

    accepts!(CHAR_ARRAY);
}

impl ToSql for ShortAddress {
    fn to_sql(&self, _: &Type, w: &mut Vec<u8>) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        w.extend_from_slice(self.as_bytes());
        Ok(IsNull::No)
    }

    accepts!(CHAR_ARRAY);
    to_sql_checked!();
}
