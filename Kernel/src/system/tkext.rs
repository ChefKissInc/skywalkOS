// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use alloc::borrow::ToOwned;
use core::hash::Hash;

use hashbrown::HashMap;

fn is_subset<K: Eq + Hash, V: Eq>(a: &HashMap<K, V>, b: &HashMap<K, V>) -> bool {
    if a.len() > b.len() {
        return false;
    }

    a.iter().all(|(k, v)| b.get(k) == Some(v))
}

pub fn spawn_new_matches() {
    let state = unsafe { &*super::state::SYS_STATE.get() };

    let dt_index = state.dt_index.as_ref().unwrap();
    let mut dt_id_gen = state.dt_id_gen.as_ref().unwrap().lock();
    let mut scheduler = state.scheduler.as_ref().unwrap().lock();

    let mut newly_matched = vec![];
    for (info, payload) in &state.tkcache.as_ref().unwrap().lock().0 {
        for ent in dt_index.read().values() {
            let mut ent = ent.lock();

            if !is_subset(&info.matching_props, &ent.properties) {
                continue;
            }

            debug!(
                "Loading TungstenKit extension {} <{}> (matched <{}>)",
                info.name, info.identifier, ent.id
            );
            let id = dt_id_gen.next();
            let new = super::state::OSDTEntry {
                id,
                parent: Some(ent.id.into()),
                properties: HashMap::from([
                    ("Name".to_owned(), info.name.clone().into()),
                    ("TKExtMatch".to_owned(), info.identifier.clone().into()),
                ]),
                children: vec![],
            };
            ent.children.push(id.into());
            newly_matched.push((id, spin::Mutex::new(new)));
            let thread = scheduler.spawn_proc(payload);
            thread.regs.rdi = id;
        }
    }
    dt_index.write().extend(newly_matched);
}
