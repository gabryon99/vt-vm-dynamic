pub struct Program {
    pub data: Vec<u8>,
    pub initial_acc: i32,
    pub initial_lc: i32,
}

impl Program {
    pub fn new(data: Vec<u8>, initial_acc: i32, initial_lc: i32) -> Self {
        Self {
            data,
            initial_acc,
            initial_lc,
        }
    }
}
