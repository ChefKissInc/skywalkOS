// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use core::hash::Hash;

use hashbrown::HashMap;
use tungstenkit::{
    osdtentry::{OSDTENTRY_NAME_KEY, TKEXT_MATCH_KEY, TKEXT_PROC_KEY},
    osvalue::OSValue,
    TKInfo,
};

use super::proc::scheduler::Scheduler;
use crate::utils::incr_id::IncrementalIDGen;

fn is_subset<K: Eq + Hash, V: Eq>(a: &HashMap<K, V>, b: &HashMap<K, V>) -> bool {
    if a.len() > b.len() {
        return false;
    }

    a.iter().all(|(k, v)| b.get(k) == Some(v))
}

fn load_tkext(
    ent: &mut super::state::OSDTEntry,
    info: &TKInfo,
    personality: &str,
    payload: &[u8],
    dt_id_gen: &mut IncrementalIDGen,
    scheduler: &mut Scheduler,
) -> super::state::OSDTEntry {
    debug!(
        "TungstenKit extension {} matched <{}> for personality {personality}",
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
                TKEXT_MATCH_KEY.into(),
                (info.identifier.as_str(), personality).into(),
            ),
            (TKEXT_PROC_KEY.into(), thread.pid.into()),
        ]),
        ..Default::default()
    };
    ent.children.push(new.id.into());
    thread.regs.rdi = new.id;
    new
}

pub fn handle_change(scheduler: &mut Scheduler, ent: tungstenkit::osdtentry::OSDTEntry) {
    let state = unsafe { &*super::state::SYS_STATE.get() };

    let dt_index = state.dt_index.as_ref().unwrap();
    let mut dt_id_gen = state.dt_id_gen.as_ref().unwrap().lock();

    let new = {
        let dt_index = dt_index.read();
        let mut ent = dt_index.get::<u64>(&ent.into()).unwrap().lock();
        if ent.children.is_empty() {
            return;
        }
        let tkcache = &state.tkcache.as_ref().unwrap().lock().0;
        tkcache.iter().find_map(|(info, payload)| {
            let identifier: OSValue = info.identifier.as_str().into();
            let attached = ent
                .children
                .iter()
                .filter_map(|id| dt_index.get::<u64>(&id.into()))
                .any(|v| v.lock().properties.get(TKEXT_MATCH_KEY) == Some(&identifier));

            for (personality, matching) in &info.personalities {
                if !attached && is_subset(matching, &ent.properties) {
                    return Some(load_tkext(
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
    };
    if let Some(new) = new {
        dt_index.write().insert(new.id, new.into());
    }
}

pub fn spawn_initial_matches() {
    let state = unsafe { &*super::state::SYS_STATE.get() };

    let dt_index = state.dt_index.as_ref().unwrap();
    let mut dt_id_gen = state.dt_id_gen.as_ref().unwrap().lock();
    let mut scheduler = state.scheduler.as_ref().unwrap().lock();

    let mut newly_matched = vec![];
    for (info, payload) in &state.tkcache.as_ref().unwrap().lock().0 {
        for ent in dt_index.read().values() {
            let mut ent = ent.lock();
            for (personality, matching) in &info.personalities {
                if is_subset(matching, &ent.properties) {
                    let new = load_tkext(
                        &mut ent,
                        info,
                        personality,
                        payload,
                        &mut dt_id_gen,
                        &mut scheduler,
                    );
                    newly_matched.push((new.id, new.into()));
                }
            }
        }
    }
    dt_index.write().extend(newly_matched);
}
