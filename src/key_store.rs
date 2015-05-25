// Copyright 2015 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.0.  This, along with the
// Licenses can be found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

use sodiumoxide::crypto::sign;
use lru_time_cache::LruCache;
use std::collections::{BTreeMap, BTreeSet};

const NAME_CAPACITY: usize = 1000;

// FIXME: We only work with KeyData and not PublicKey directly
// because PublicKey doesn't derive from Ord in the current version of
// sodiumdioxide library. Once that library is bumped to version 0.0.6
// or above, we should be able to get rid of KeyData and the re-wrapping.
type KeyData   = [u8; sign::PUBLICKEYBYTES];
type Map<A, B> = BTreeMap<A,B>;
type Set<A>    = BTreeSet<A>;

pub struct KeyStore<Name> where Name: Eq + PartialOrd + Ord + Clone {
    quorum_size: usize,
    //              +--- To                +--- Sender
    //              V                      V
    cache: LruCache<Name, Map<KeyData, Set<Name>>>,
}

impl<Name> KeyStore<Name> where Name: Eq + PartialOrd + Ord + Clone {
    pub fn new(quorum_size: usize) -> KeyStore<Name> {
        KeyStore{ quorum_size: quorum_size
                , cache: LruCache::with_capacity(NAME_CAPACITY)
        }
    }

    pub fn add_key(&mut self, to: Name, sender: Name, key: sign::PublicKey) {
        let new_map = || { Map::<KeyData, Set<Name>>::new() };
        let new_set = || { Set::<Name>::new() };

        self.cache.entry(to).or_insert_with(new_map)
                  .entry(key.0).or_insert_with(new_set)
                  .insert(sender);
    }

    /// Returns a vector of keys belonging to `to`, for whom we've received the key
    /// from at least a quorum size of unique senders.
    pub fn get_accumulated_keys(&mut self, to: &Name) -> Vec<sign::PublicKey> {
        // Create temp variable to workaround a borrow checker bug
        // http://blog.ezyang.com/2013/12/two-bugs-in-the-borrow-checker-every-rust-developer-should-know-about/
        let quorum = self.quorum_size;

        self.cache.get(to)
            .iter().flat_map(|keys| Self::pick_where_quorum_reached(keys, quorum))
            .cloned().map(sign::PublicKey)
            .collect::<_>()
    }

    fn pick_where_quorum_reached<'a>(keys: &'a Map<KeyData, Set<Name>>, quorum: usize)
    -> Vec<&'a KeyData> {
        keys.iter().filter_map(|(key, sender_set)| {
            return if sender_set.len() >= quorum { Some(key) } else { None };
        }).collect::<_>()
    }
}

