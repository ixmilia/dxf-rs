use crate::code_pair_put_back::CodePairPutBack;
use crate::drawing::AUTO_REPLACE_HANDLE;
use crate::entities::*;
use crate::DxfResult;

use itertools::{put_back, PutBack};

pub(crate) struct EntityIter<'a> {
    pub iter: &'a mut CodePairPutBack,
}

impl<'a> Iterator for EntityIter<'a> {
    type Item = Entity;

    fn next(&mut self) -> Option<Entity> {
        match Entity::read(self.iter) {
            Ok(Some(e)) => Some(e),
            Ok(None) | Err(_) => None,
        }
    }
}

impl<'a> EntityIter<'a> {
    pub(crate) fn read_entities_into_vec(&mut self, entities: &mut Vec<Entity>) -> DxfResult<()> {
        collect_entities(self, entities)
    }
}

pub(crate) fn collect_entities<I>(iter: &mut I, entities: &mut Vec<Entity>) -> DxfResult<()>
where
    I: Iterator<Item = Entity>,
{
    fn swallow_seqend<I>(iter: &mut PutBack<I>) -> DxfResult<()>
    where
        I: Iterator<Item = Entity>,
    {
        match iter.next() {
            Some(Entity {
                specific: EntityType::Seqend(_),
                ..
            }) => (),
            Some(ent) => {
                iter.put_back(ent);
            }
            None => (),
        }

        Ok(())
    }

    fn mtext<I>(iter: &mut PutBack<I>) -> DxfResult<Option<MText>>
    where
        I: Iterator<Item = Entity>,
    {
        let m_text = match iter.next() {
            Some(Entity {
                specific: EntityType::MText(m),
                ..
            }) => Some(m),
            Some(ent) => {
                iter.put_back(ent);
                None
            }
            None => None,
        };

        Ok(m_text)
    }

    let mut iter = put_back(iter);
    loop {
        match iter.next() {
            Some(Entity {
                ref common,
                specific: EntityType::Attribute(ref att),
            }) => {
                let mut att = att.clone(); // 27 fields
                match mtext(&mut iter) {
                    Ok(Some(m_text)) => att.m_text = m_text,
                    Ok(None) => (),
                    Err(e) => return Err(e),
                }

                entities.push(Entity {
                    common: common.clone(), // 18 fields
                    specific: EntityType::Attribute(att),
                });
            }
            Some(Entity {
                ref common,
                specific: EntityType::AttributeDefinition(ref att),
            }) => {
                let mut att = att.clone(); // 27 fields
                match mtext(&mut iter) {
                    Ok(Some(m_text)) => att.m_text = m_text,
                    Ok(None) => (),
                    Err(e) => return Err(e),
                }

                entities.push(Entity {
                    common: common.clone(), // 18 fields
                    specific: EntityType::AttributeDefinition(att),
                });
            }
            Some(Entity {
                ref common,
                specific: EntityType::Insert(ref ins),
            }) if ins.__has_attributes => {
                let mut ins = ins.clone(); // 12 fields
                loop {
                    match iter.next() {
                        Some(Entity {
                            specific: EntityType::Attribute(att),
                            ..
                        }) => ins
                            .__attributes_and_handles
                            .push((att, AUTO_REPLACE_HANDLE)),
                        Some(ent) => {
                            // stop gathering on any non-ATTRIBUTE
                            iter.put_back(ent);
                            break;
                        }
                        None => break,
                    }
                }

                swallow_seqend(&mut iter)?;

                // and finally keep the INSERT
                entities.push(Entity {
                    common: common.clone(), // 18 fields
                    specific: EntityType::Insert(ins),
                });
            }
            Some(Entity {
                common,
                specific: EntityType::Polyline(poly),
            }) => {
                let mut poly = poly.clone(); // 13 fields
                loop {
                    match iter.next() {
                        Some(Entity {
                            specific: EntityType::Vertex(vertex),
                            ..
                        }) => poly
                            .__vertices_and_handles
                            .push((vertex, AUTO_REPLACE_HANDLE)),
                        Some(ent) => {
                            // stop gathering on any non-VERTEX
                            iter.put_back(ent);
                            break;
                        }
                        None => break,
                    }
                }

                swallow_seqend(&mut iter)?;

                // and finally keep the POLYLINE
                entities.push(Entity {
                    common: common.clone(), // 18 fields
                    specific: EntityType::Polyline(poly),
                });
            }
            Some(entity) => entities.push(entity),
            None => break,
        }
    }

    Ok(())
}
