use crate::types::{
    Condition, FieldCondition, Filter, Payload, Range as RangeCondition, VectorElementType,
};
use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::Rng;
use serde_json::{json, Value};
use std::ops::Range;

const ADJECTIVE: &[&str] = &[
    "jobless",
    "rightful",
    "breakable",
    "impartial",
    "shocking",
    "faded",
    "phobic",
    "overt",
    "like",
    "wide-eyed",
    "broad",
];

const NOUN: &[&str] = &[
    "territory",
    "jam",
    "neck",
    "chicken",
    "cap",
    "kiss",
    "veil",
    "trail",
    "size",
    "digestion",
    "rod",
    "seed",
];

const INT_RANGE: Range<i64> = 0..500;
pub const LON_RANGE: Range<f64> = -180.0..180.0;
pub const LAT_RANGE: Range<f64> = -90.0..90.0;

pub const STR_KEY: &str = "kvd";
pub const INT_KEY: &str = "int";
pub const FLT_KEY: &str = "flt";
pub const FLICKING_KEY: &str = "flicking";
pub const GEO_KEY: &str = "geo";

pub fn random_keyword<R: Rng + ?Sized>(rnd_gen: &mut R) -> String {
    let random_adj = ADJECTIVE.choose(rnd_gen).unwrap();
    let random_noun = NOUN.choose(rnd_gen).unwrap();
    format!("{} {}", random_adj, random_noun)
}

pub fn random_keyword_payload<R: Rng + ?Sized>(rnd_gen: &mut R, num_values: usize) -> Value {
    if num_values > 1 {
        Value::Array(
            (0..num_values)
                .map(|_| Value::String(random_keyword(rnd_gen)))
                .collect(),
        )
    } else {
        Value::String(random_keyword(rnd_gen))
    }
}

pub fn random_int_payload<R: Rng + ?Sized>(rnd_gen: &mut R, num_values: usize) -> Vec<i64> {
    (0..num_values)
        .map(|_| rnd_gen.gen_range(INT_RANGE))
        .collect_vec()
}

pub fn random_geo_payload<R: Rng + ?Sized>(rnd_gen: &mut R, num_values: usize) -> Vec<Value> {
    (0..num_values)
        .map(|_| {
            json!( {
                "lon": rnd_gen.gen_range(LON_RANGE),
                "lat": rnd_gen.gen_range(LAT_RANGE),
            })
        })
        .collect_vec()
}

pub fn random_vector<R: Rng + ?Sized>(rnd_gen: &mut R, size: usize) -> Vec<VectorElementType> {
    (0..size).map(|_| rnd_gen.gen()).collect()
}

pub fn random_field_condition<R: Rng + ?Sized>(rnd_gen: &mut R) -> Condition {
    let kv_or_int: bool = rnd_gen.gen();
    if kv_or_int {
        Condition::Field(FieldCondition::new_match(
            STR_KEY.to_string(),
            random_keyword(rnd_gen).into(),
        ))
    } else {
        Condition::Field(FieldCondition::new_range(
            INT_KEY.to_string(),
            RangeCondition {
                lt: None,
                gt: None,
                gte: Some(rnd_gen.gen_range(INT_RANGE) as f64),
                lte: Some(rnd_gen.gen_range(INT_RANGE) as f64),
            },
        ))
    }
}

pub fn random_must_filter<R: Rng + ?Sized>(rnd_gen: &mut R, num_conditions: usize) -> Filter {
    let must_conditions = (0..num_conditions)
        .map(|_| random_field_condition(rnd_gen))
        .collect_vec();

    Filter {
        should: None,
        must: Some(must_conditions),
        must_not: None,
    }
}

pub fn random_filter<R: Rng + ?Sized>(rnd_gen: &mut R) -> Filter {
    let mut rnd1 = rand::thread_rng();

    let should_conditions = (0..=2)
        .take_while(|_| rnd1.gen::<f64>() > 0.6)
        .map(|_| random_field_condition(rnd_gen))
        .collect_vec();

    let should_conditions_opt = if !should_conditions.is_empty() {
        Some(should_conditions)
    } else {
        None
    };

    let must_conditions = (0..=2)
        .take_while(|_| rnd1.gen::<f64>() > 0.6)
        .map(|_| random_field_condition(rnd_gen))
        .collect_vec();

    let must_conditions_opt = if !must_conditions.is_empty() {
        Some(must_conditions)
    } else {
        None
    };

    Filter {
        should: should_conditions_opt,
        must: must_conditions_opt,
        must_not: None,
    }
}

pub fn generate_diverse_payload<R: Rng + ?Sized>(rnd_gen: &mut R) -> Payload {
    let payload: Payload = if rnd_gen.gen_range(0.0..1.0) < 0.5 {
        json!({
            STR_KEY: random_keyword_payload(rnd_gen, 2),
            INT_KEY: random_int_payload(rnd_gen, 2),
            FLT_KEY: rnd_gen.gen_range(0.0..10.0),
            GEO_KEY: random_geo_payload(rnd_gen, 2)
        })
        .into()
    } else {
        json!({
            STR_KEY: random_keyword_payload(rnd_gen, 2),
            INT_KEY: random_int_payload(rnd_gen, 2),
            FLT_KEY: rnd_gen.gen_range(0.0..10.0),
            GEO_KEY: random_geo_payload(rnd_gen, 2),
            FLICKING_KEY: random_int_payload(rnd_gen, 2)
        })
        .into()
    };

    payload
}
