use chrono::{DateTime, Local, TimeDelta, Timelike};
use flagset::FlagSet;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro128Plus;
use tracing::debug;

use crate::{
    records::{Check, CheckFlag, TARGETS},
    store::Store,
};

pub const N: usize = 30_000;
pub const DEFAULT_SEED: u64 = 1686429357;
pub const BLAKE3_HASH_OF_DEFAULT_DATASET: &str =
    "2d6c3542a60645c48c3a0023026a370a18c4d5d3b529a738be7b0a5e10ee5e9f";

pub fn base_time() -> DateTime<Local> {
    let utc = DateTime::from_timestamp(1686429357, 0)
        .unwrap()
        .with_second(0)
        .unwrap();
    utc.into()
}

pub fn default_dataset() -> Store {
    let a = generate_dataset(DEFAULT_SEED);
    let hash = a.get_hash().to_string();
    if hash != BLAKE3_HASH_OF_DEFAULT_DATASET {
        panic!("the hash of the generated default dataset is wrong.\n{hash}\nis not\n{BLAKE3_HASH_OF_DEFAULT_DATASET}")
    }
    a
}

pub fn generate_dataset(seed: u64) -> Store {
    let mut rng: Xoshiro128Plus = Xoshiro128Plus::seed_from_u64(seed);
    let mut buf = Vec::new();
    let base_time = base_time();
    let mut r: u32 = rng.gen();
    let mut time;
    debug!("first r: {r}");
    for idx in 0..N {
        time = base_time + TimeDelta::minutes(idx as i64);

        r = rng.gen();
        buf.push(Check::new(
            time,
            FlagSet::from(CheckFlag::TypeIcmp)
                | if !success(r, idx) {
                    CheckFlag::Unreachable
                } else {
                    CheckFlag::Success
                },
            Some((r % 100) as u16),
            TARGETS[idx % TARGETS.len()].parse().unwrap(),
        ));
        r = rng.gen();
        buf.push(Check::new(
            time,
            FlagSet::from(CheckFlag::TypeIcmp)
                | if !success(r, idx) {
                    CheckFlag::Unreachable
                } else {
                    CheckFlag::Success
                },
            Some((r % 100) as u16),
            TARGETS[idx % TARGETS.len()].parse().unwrap(),
        ));
        r = rng.gen();
        buf.push(Check::new(
            time,
            FlagSet::from(CheckFlag::TypeHTTP)
                | if !success(r, idx) {
                    CheckFlag::Unreachable
                } else {
                    CheckFlag::Success
                },
            Some((r % 100) as u16),
            TARGETS[idx % TARGETS.len()].parse().unwrap(),
        ));
        r = rng.gen();
        buf.push(Check::new(
            time,
            FlagSet::from(CheckFlag::TypeHTTP)
                | if !success(r, idx) {
                    CheckFlag::Unreachable
                } else {
                    CheckFlag::Success
                },
            Some((r % 100) as u16),
            TARGETS[idx % TARGETS.len()].parse().unwrap(),
        ));
    }
    debug!("last r: {r}");

    buf.sort();
    Store::from_raw_in_mem(buf)
}

fn success(r: u32, idx: usize) -> bool {
    !(2020usize..2280).contains(&idx) && !(15020usize..15080).contains(&idx) && r % 4000 != 1
}

#[cfg(test)]
mod tests {
    use super::default_dataset;

    #[test]
    #[should_panic]
    fn test_cant_save_default_dataset() {
        let virtual_store = default_dataset();
        virtual_store.save().expect("could not save")
    }
}
