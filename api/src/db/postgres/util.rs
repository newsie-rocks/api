//! Utilities

use std::io::Read;

use salvo::prelude::ToSchema;
use serde::{Deserialize, Serialize};
use tokio_postgres::types::{to_sql_checked, FromSql, ToSql};

/// A vector type
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Vector(Vec<f32>);

impl From<Vec<f32>> for Vector {
    fn from(value: Vec<f32>) -> Self {
        Vector(value)
    }
}

impl From<Vector> for Vec<f32> {
    fn from(value: Vector) -> Self {
        value.0
    }
}

impl ToSql for Vector {
    fn to_sql(
        &self,
        _ty: &tokio_postgres::types::Type,
        out: &mut tokio_postgres::types::private::BytesMut,
    ) -> Result<tokio_postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        // NB: a vector value is passed as '[1,2,3]'
        // This code is copied from te crate `pgvector`
        let dim: u16 = self.0.len().try_into()?;
        out.extend(dim.to_be_bytes());
        out.extend(0_u16.to_be_bytes());
        for v in self.0.iter() {
            out.extend(v.to_be_bytes())
        }

        Ok(tokio_postgres::types::IsNull::No)
    }

    fn accepts(ty: &tokio_postgres::types::Type) -> bool
    where
        Self: Sized,
    {
        ty.name() == "vector"
    }

    to_sql_checked!();
}

impl<'a> FromSql<'a> for Vector {
    fn from_sql(
        _ty: &tokio_postgres::types::Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let mut buf = raw;

        let mut buf_u16 = [0x00_u8; 2];
        buf.read_exact(&mut buf_u16)?;
        let dim = u16::from_be_bytes(buf_u16);

        let mut buf_u16 = [0x00_u8; 2];
        buf.read_exact(&mut buf_u16)?;
        let unused = u16::from_be_bytes(buf_u16);
        if unused != 0 {
            return Err("expected unused to be 0".into());
        }

        let mut values = vec![];
        for _i in 0..(dim as usize) {
            let mut buf_f32 = [0x00_u8; 4];
            buf.read_exact(&mut buf_f32)?;
            let v = f32::from_be_bytes(buf_f32);
            values.push(v);
        }
        Ok(Vector(values))
    }

    fn accepts(ty: &tokio_postgres::types::Type) -> bool {
        ty.name() == "vector"
    }
}
