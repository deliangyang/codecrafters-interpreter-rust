use crate::objects::Object;

impl<'a> Container<'a> {
    pub fn new(obj: Object) -> Self {
        Container {
            object: obj,
            ref_object: None,
        }
    }

    pub fn set_ref(&'a mut self) {
        self.ref_object = Some(&self.object);
    }

    pub fn get_ref(&self) -> Option<&'a Object> {
        self.ref_object
    }
}

pub struct Container<'a> {
    object: Object,
    ref_object: Option<&'a Object>,
}
