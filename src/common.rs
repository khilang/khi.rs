//! todo: Implement convenience methods for arguments and expressions that can be used to evaluate them to numbers, text, etc.


trait Text {
    fn ref_str(&self) -> &str;
}


impl dyn Text {
    pub fn as_str(&self) -> &str {
        self.ref_str()
    }
    pub fn as_u64(&mut self) {

    }
    pub fn as_f64(&mut self) {

    }
    pub fn as_bool() {

    }
}


trait Sequence {

}


trait Dictionary {

}


trait Compound {

}


trait Command {

}
