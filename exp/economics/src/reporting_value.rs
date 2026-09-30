//! Fixed opening reporting valuations for native-denomination claims. No marks or FX.
use crate::model::*;
use std::collections::BTreeMap;

pub(crate) fn value(
    coin: ResourceId,
    values: &BTreeMap<ResourceId, i128>,
    resource: ResourceId,
    quantity: i32,
) -> Result<i128, String> {
    let unit = if resource == coin {
        1
    } else {
        *values
            .get(&resource)
            .filter(|v| **v > 0)
            .ok_or("noncash claims need an explicit positive exchange value")?
    };
    unit.checked_mul(i128::from(quantity))
        .ok_or_else(|| "claim reporting value overflow".into())
}
