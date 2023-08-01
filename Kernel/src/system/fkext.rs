// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use alloc::vec::Vec;
use core::hash::Hash;

use fireworkkit::{
    osdtentry::{FKEXT_MATCH_KEY, FKEXT_PROC_KEY, OSDTENTRY_NAME_KEY},
    FKInfo,
};
use hashbrown::HashMap;

use super::proc::scheduler::Scheduler;
use crate::utils::incr_id::IncrementalIDGen;

fn is_subset<K: Eq + Hash, V: Eq>(a: &HashMap<K, V>, b: &HashMap<K, V>) -> bool {
    if a.len() > b.len() {
        return false;
    }

    a.iter().all(|(k, v)| b.get(k) == Some(v))
}

fn load_fkext(
    ent: &mut super::state::OSDTEntry,
    info: &FKInfo,
    personality: &str,
    payload: &[u8],
    dt_id_gen: &mut IncrementalIDGen,
    scheduler: &mut Scheduler,
) -> (u64, spin::Mutex<super::state::OSDTEntry>) {
    debug!(
        "FireworkKit extension {} matched <{}> for personality {personality}",
        info.identifier, ent.id
    );
    let thread = scheduler.spawn_proc(payload);
    let new = super::state::OSDTEntry {
        id: dt_id_gen.next(),
        parent: Some(ent.id.into()),
        properties: HashMap::from([
            (
                OSDTENTRY_NAME_KEY.into(),
                info.identifier.split('.').last().unwrap().into(),
            ),
            (
                FKEXT_MATCH_KEY.into(),
                (info.identifier.as_str(), personality).into(),
            ),
            (FKEXT_PROC_KEY.into(), thread.pid.into()),
        ]),
        ..Default::default()
    };
    ent.children.push(new.id.into());
    thread.regs.rdi = new.id;
    (new.id, new.into())
}

pub fn handle_change(scheduler: &mut Scheduler, ent: fireworkkit::osdtentry::OSDTEntry) {
    let state = unsafe { &*super::state::SYS_STATE.get() };

    let dt_index = state.dt_index.as_ref().unwrap();
    let mut dt_id_gen = state.dt_id_gen.as_ref().unwrap().lock();

    let new: Vec<_> = {
        let dt_index = dt_index.read();
        let mut ent = dt_index.get::<u64>(&ent.into()).unwrap().lock();
        let fkcache = &state.fkcache.as_ref().unwrap().lock().0;
        fkcache
            .iter()
            .filter_map(|(info, payload)| {
                for (personality, matching) in &info.personalities {
                    let match_ = (info.identifier.as_str(), personality.as_str()).into();
                    let attached = ent
                        .children
                        .iter()
                        .filter_map(|id| dt_index.get::<u64>(&id.into()))
                        .any(|v| v.lock().properties.get(FKEXT_MATCH_KEY) == Some(&match_));
                    if !attached && is_subset(matching, &ent.properties) {
                        return Some(load_fkext(
                            &mut ent,
                            info,
                            personality,
                            payload,
                            &mut dt_id_gen,
                            scheduler,
                        ));
                    }
                }
                None
            })
            .collect()
    };

    dt_index.write().extend(new);
}

pub fn spawn_initial_matches() {
    let state = unsafe { &*super::state::SYS_STATE.get() };

    let dt_index = state.dt_index.as_ref().unwrap();
    let mut dt_id_gen = state.dt_id_gen.as_ref().unwrap().lock();
    let mut scheduler = state.scheduler.as_ref().unwrap().lock();

    let mut newly_matched = vec![];
    for ((info, payload), mut ent) in iproduct!(
        &state.fkcache.as_ref().unwrap().lock().0,
        dt_index.read().values()
    )
    .map(|(info, ent)| (info, ent.lock()))
    {
        for (personality, matching) in &info.personalities {
            if is_subset(matching, &ent.properties) {
                let new = load_fkext(
                    &mut ent,
                    info,
                    personality,
                    payload,
                    &mut dt_id_gen,
                    &mut scheduler,
                );
                newly_matched.push(new);
            }
        }
    }
    dt_index.write().extend(newly_matched);
}
