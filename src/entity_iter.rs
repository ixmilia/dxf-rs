// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use itertools::{
    PutBack,
    put_back,
};
use ::{
    CodePair,
    DxfResult,
};
use ::entities::*;

pub(crate) struct EntityIter<'a, I: 'a + Iterator<Item = DxfResult<CodePair>>> {
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
    pub(crate) fn read_entities_into_vec(&mut self, entities: &mut Vec<Entity>) -> DxfResult<()> {
        collect_entities(self, entities)
    }
}

pub(crate) fn collect_entities<I>(iter: &mut I, entities: &mut Vec<Entity>) -> DxfResult<()>
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

    fn get_mtext<I>(iter: &mut PutBack<I>) -> DxfResult<Option<MText>>
        where I: Iterator<Item = Entity> {

        let m_text = match iter.next() {
            Some(Entity { specific: EntityType::MText(m), .. }) => Some(m),
            Some(ent) => { iter.put_back(ent); None },
            None => None,
        };

        Ok(m_text)
    }

    let mut iter = put_back(iter);
    loop {
        match iter.next() {
            Some(Entity { ref common, specific: EntityType::Attribute(ref att) }) => {
                let mut att = att.clone(); // 27 fields
                match get_mtext(&mut iter) {
                    Ok(Some(m_text)) => att.m_text = m_text,
                    Ok(None) => (),
                    Err(e) => return Err(e),
                }

                entities.push(Entity {
                    common: common.clone(), // 18 fields
                    specific: EntityType::Attribute(att),
                });
            },
            Some(Entity { ref common, specific: EntityType::AttributeDefinition(ref att) }) => {
                let mut att = att.clone(); // 27 fields
                match get_mtext(&mut iter) {
                    Ok(Some(m_text)) => att.m_text = m_text,
                    Ok(None) => (),
                    Err(e) => return Err(e),
                }

                entities.push(Entity {
                    common: common.clone(), // 18 fields
                    specific: EntityType::AttributeDefinition(att),
                });
            },
            Some(Entity { ref common, specific: EntityType::Insert(ref ins) }) if ins.__has_attributes => {
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

                swallow_seqend(&mut iter)?;

                // and finally keep the INSERT
                entities.push(Entity {
                    common: common.clone(), // 18 fields
                    specific: EntityType::Insert(ins),
                });
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

                swallow_seqend(&mut iter)?;

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
