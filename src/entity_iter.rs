// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use itertools::PutBack;
use ::{
    CodePair,
    DxfResult,
};
use ::entities::*;

#[doc(hidden)]
pub struct EntityIter<'a, I: 'a + Iterator<Item = DxfResult<CodePair>>> {
    pub iter: &'a mut PutBack<I>,
}

impl<'a, I: 'a + Iterator<Item = DxfResult<CodePair>>> Iterator for EntityIter<'a, I> {
    type Item = Entity;

    fn next(&mut self) -> Option<Entity> {
        match Entity::read(self.iter) {
            Ok(Some(e)) => Some(e),
            Ok(None) | Err(_) => None,
        }
    }
}

impl<'a, I: 'a + Iterator<Item = DxfResult<CodePair>>> EntityIter<'a, I> {
    #[doc(hidden)]
    pub fn read_entities_into_vec(&mut self, entities: &mut Vec<Entity>) -> DxfResult<()> {
        collect_entities(self, entities)
    }
}

#[doc(hidden)]
pub fn collect_entities<I>(iter: &mut I, entities: &mut Vec<Entity>) -> DxfResult<()>
    where I: Iterator<Item = Entity> {

    fn swallow_seqend<I>(iter: &mut PutBack<I>) -> DxfResult<()>
        where I: Iterator<Item = Entity> {

        match iter.next() {
            Some(Entity { specific: EntityType::Seqend(_), .. }) => (),
            Some(ent) => iter.put_back(ent),
            None => (),
        }

        Ok(())
    }

    let mut iter = PutBack::new(iter);
    loop {
        match iter.next() {
            Some(Entity { ref common, specific: EntityType::Insert(ref ins) }) if ins.has_attributes => {
                let mut ins = ins.clone(); // 12 fields
                loop {
                    match iter.next() {
                        Some(Entity { specific: EntityType::Attribute(att), .. }) => ins.attributes.push(att),
                        Some(ent) => {
                            // stop gathering on any non-ATTRIBUTE
                            iter.put_back(ent);
                            break;
                        },
                        None => break,
                    }
                }

                try!(swallow_seqend(&mut iter));

                // and finally keep the INSERT
                entities.push(Entity {
                    common: common.clone(), // 18 fields
                    specific: EntityType::Insert(ins),
                })
            },
            Some(Entity { common, specific: EntityType::Polyline(poly) }) => {
                let mut poly = poly.clone(); // 13 fields
                loop {
                    match iter.next() {
                        Some(Entity { specific: EntityType::Vertex(vertex), .. }) => poly.vertices.push(vertex),
                        Some(ent) => {
                            // stop gathering on any non-VERTEX
                            iter.put_back(ent);
                            break;
                        },
                        None => break,
                    }
                }

                try!(swallow_seqend(&mut iter));

                // and finally keep the POLYLINE
                entities.push(Entity {
                    common: common.clone(), // 18 fields
                    specific: EntityType::Polyline(poly),
                });
            },
            Some(entity) => entities.push(entity),
            None => break,
        }
    }

    Ok(())
}
