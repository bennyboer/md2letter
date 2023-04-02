pub(crate) type Id = usize;

#[derive(Debug)]
pub(crate) struct IdGenerator {
    next_id: Id,
}

impl IdGenerator {
    pub(crate) fn new() -> Self {
        Self { next_id: 0 }
    }

    pub(crate) fn next(&mut self) -> Id {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}
