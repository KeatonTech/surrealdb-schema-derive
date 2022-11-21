use surrealdb::sql::{Array, Thing, Value};

use crate::{errors::InvalidValueTypeError, SurrealDbRow, SurrealDbSchemaDeriveQueryError};

pub struct SurrealValue(pub Value);

impl From<Value> for SurrealValue {
    fn from(value: Value) -> Self {
        SurrealValue(value)
    }
}

impl Into<Value> for SurrealValue {
    fn into(self) -> Value {
        self.0
    }
}

pub struct SurrealOption<T>(pub Option<T>);

impl<T> Into<Option<T>> for SurrealOption<T> {
    fn into(self) -> Option<T> {
        self.0
    }
}

impl<T> From<Option<T>> for SurrealOption<T> {
    fn from(option: Option<T>) -> Self {
        SurrealOption(option)
    }
}

// Surreal -> Rust

impl TryFrom<SurrealValue> for bool {
    type Error = SurrealDbSchemaDeriveQueryError;

    fn try_from(value: SurrealValue) -> Result<Self, Self::Error> {
        match value.0 {
            Value::True => Ok(true),
            Value::False => Ok(false),
            _ => Err(SurrealDbSchemaDeriveQueryError::InvalidValueTypeError(InvalidValueTypeError {
                expected_type: "bool".into(),
                received_type: value.0.to_string(),
            })),
        }
    }
}

impl<T> TryFrom<SurrealValue> for SurrealOption<T>
where
    T: TryFrom<SurrealValue>,
{
    type Error = <T as TryFrom<SurrealValue>>::Error;

    fn try_from(value: SurrealValue) -> Result<Self, Self::Error> {
        if let Value::None = value.0 {
            Ok(SurrealOption(None))
        } else {
            Ok(SurrealOption(Some(T::try_from(value)?)))
        }
    }
}

impl<T> TryFrom<SurrealValue> for Vec<T>
where
    T: TryFrom<SurrealValue, Error = SurrealDbSchemaDeriveQueryError>,
{
    type Error = SurrealDbSchemaDeriveQueryError;

    fn try_from(value: SurrealValue) -> Result<Self, Self::Error> {
        let Value::Array(array_val) = value.0 else {
            return Err(SurrealDbSchemaDeriveQueryError::InvalidValueTypeError(InvalidValueTypeError {
                expected_type: "array".into(),
                received_type: value.0.to_string(),
            }));
        };
        println!("{:?}", array_val);

        return Ok(array_val
            .0
            .into_iter()
            .map(|val| SurrealValue(val).try_into())
            .collect::<Result<Vec<_>, _>>()?);
    }
}

macro_rules! impl_surreal_value_try_from {
    ($variant:pat => $map:expr => $type:ty) => {
        impl TryFrom<SurrealValue> for $type {
            type Error = SurrealDbSchemaDeriveQueryError;

            fn try_from(value: SurrealValue) -> Result<Self, Self::Error> {
                if let $variant = value.0 {
                    Ok($map)
                } else {
                    Err(SurrealDbSchemaDeriveQueryError::InvalidValueTypeError(InvalidValueTypeError {
                        expected_type: stringify!($type).into(),
                        received_type: value.0.to_string(),
                    }))
                }
            }
        }
    };

    ($variant:pat => try_into $map:expr => $type:ty) => {
        impl_surreal_value_try_from! {$variant => $map.try_into().map_err(move |_| {
            SurrealDbSchemaDeriveQueryError::InvalidValueTypeError(InvalidValueTypeError {
                expected_type: stringify!($type).into(),
                received_type: "Type mapping failed".into(),
            })
        })? => $type}
    };
}

impl_surreal_value_try_from!(surrealdb::sql::Value::Strand(it) => it.to_string() => String);
impl_surreal_value_try_from!(surrealdb::sql::Value::Number(it) => try_into it.to_usize() => u8);
impl_surreal_value_try_from!(surrealdb::sql::Value::Number(it) => try_into it.to_usize() => u16);
impl_surreal_value_try_from!(surrealdb::sql::Value::Number(it) => try_into it.to_usize() => u32);
impl_surreal_value_try_from!(surrealdb::sql::Value::Number(it) => try_into it.to_usize() => u64);
impl_surreal_value_try_from!(surrealdb::sql::Value::Number(it) => it.to_usize() => usize);
impl_surreal_value_try_from!(surrealdb::sql::Value::Number(it) => try_into it.to_int() => i8);
impl_surreal_value_try_from!(surrealdb::sql::Value::Number(it) => try_into it.to_int() => i16);
impl_surreal_value_try_from!(surrealdb::sql::Value::Number(it) => try_into it.to_int() => i32);
impl_surreal_value_try_from!(surrealdb::sql::Value::Number(it) => it.to_int() => i64);
impl_surreal_value_try_from!(surrealdb::sql::Value::Number(it) => try_into it.to_int() => isize);
impl_surreal_value_try_from!(surrealdb::sql::Value::Number(it) => it.to_float() as f32 => f32);
impl_surreal_value_try_from!(surrealdb::sql::Value::Number(it) => it.to_float() => f64);

// Rust -> Surreal

impl Into<SurrealValue> for bool {
    fn into(self) -> SurrealValue {
        SurrealValue(if self { Value::True } else { Value::False })
    }
}

impl Into<SurrealValue> for String {
    fn into(self) -> SurrealValue {
        SurrealValue(Value::Strand(surrealdb::sql::Strand(self)))
    }
}

impl<T> Into<SurrealValue> for Vec<T>
where
    T: Into<SurrealValue>,
{
    fn into(self) -> SurrealValue {
        SurrealValue(Value::Array(Array(
            self.into_iter()
                .map(|item| item.into() as SurrealValue)
                .map(|item| item.0)
                .collect()
        )))
    }
}

macro_rules! impl_int_into_surreal_value {
    ($type:ty) => {
        impl Into<SurrealValue> for $type {
            fn into(self) -> SurrealValue {
                SurrealValue(surrealdb::sql::Value::Number(surrealdb::sql::Number::Int(
                    self.try_into().expect(&format!(
                        "Value of type {} out of range for surrealdb",
                        stringify!($type)
                    )),
                )))
            }
        }
    };
}

impl_int_into_surreal_value!(u8);
impl_int_into_surreal_value!(u16);
impl_int_into_surreal_value!(u32);
impl_int_into_surreal_value!(u64);
impl_int_into_surreal_value!(usize);
impl_int_into_surreal_value!(i8);
impl_int_into_surreal_value!(i16);
impl_int_into_surreal_value!(i32);
impl_int_into_surreal_value!(i64);
impl_int_into_surreal_value!(isize);

impl Into<SurrealValue> for f32 {
    fn into(self) -> SurrealValue {
        SurrealValue(Value::Number(surrealdb::sql::Number::Float(self as f64)))
    }
}

impl Into<SurrealValue> for f64 {
    fn into(self) -> SurrealValue {
        SurrealValue(Value::Number(surrealdb::sql::Number::Float(self)))
    }
}

impl<T: Into<SurrealValue>> From<SurrealOption<T>> for SurrealValue {
    fn from(option: SurrealOption<T>) -> Self {
        if let Some(value) = option.0 {
            value.into()
        } else {
            SurrealValue(surrealdb::sql::Value::None)
        }
    }
}

pub fn surreal_value_to_row<R: SurrealDbRow>(
    surreal_value: SurrealValue,
) -> Result<R, SurrealDbSchemaDeriveQueryError> {
    let SurrealValue(mut value) = surreal_value;
    let id = if let Value::Object(ref mut object_value) = value {
        if let Some(id_value) = object_value.remove("id") {
            if let Value::Thing(id_thing) = id_value {
                Ok(id_thing.id)
            } else {
                Err(SurrealDbSchemaDeriveQueryError::InvalidValueTypeError(
                    InvalidValueTypeError {
                        expected_type: "Thing".into(),
                        received_type: id_value.to_string(),
                    },
                ))
            }
        } else {
            Err(SurrealDbSchemaDeriveQueryError::RowObjectMissingIdColumn)
        }
    } else {
        Err(SurrealDbSchemaDeriveQueryError::InvalidValueTypeError(
            InvalidValueTypeError {
                expected_type: "Object".into(),
                received_type: value.to_string(),
            },
        ))
    }?;

    let inner_struct = R::Table::try_from(SurrealValue(value))?;
    Ok(R::new(id, inner_struct))
}

pub fn row_to_surreal_value<R: SurrealDbRow>(row: R) -> SurrealValue {
    let id = row.get_reference().get_id();
    let inner_value: SurrealValue = row.into().into();

    if let Value::Object(mut object_value) = inner_value.0 {
        object_value.insert(
            "id".into(),
            Value::Thing(Thing {
                tb: R::get_table_name(),
                id: id,
            }),
        );
        SurrealValue(Value::Object(object_value))
    } else {
        panic!("A generated SurrealDbStruct did not create a surrealdb Object value")
    }
}
