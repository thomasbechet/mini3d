pub(crate) enum TACOperator {
    Call,
    Add,
}

pub(crate) enum TACArgument {}

pub(crate) struct TAC {
    pub(crate) op: TACOperator,
    pub(crate) arg1: TACArgument,
    pub(crate) arg2: TACArgument,
}
