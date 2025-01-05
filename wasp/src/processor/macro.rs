use crate::parser::{Element, ElementValue};


pub(super) struct Macro {
    pub sub_count: usize,
    pub source: Vec<Element>,
}


pub(super) struct CurrentMacro {
    subs: Vec<Element>,
    elems: Vec<Element>,
}


impl CurrentMacro {
    pub fn new(mut elems: Vec<Element>, subs: Vec<Element>) -> Self {
        elems.reverse();
        Self { elems, subs }
    }
}


impl Iterator for CurrentMacro {
    type Item = Element;

    fn next(&mut self) -> Option<Self::Item> {
        Some(match self.elems.pop()? {
            Element { value: ElementValue::Substitute(i), .. } => self.subs[i].clone(),
            elem => elem,
        })
    }
}
