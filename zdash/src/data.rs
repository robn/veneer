use veneer::VdevState;

#[derive(Clone, Debug, Default)]
pub(crate) struct PoolData {
    pub name: String,
    pub state: VdevState,
    pub size: u64,
    pub alloc: u64,
    pub read: u64,
    pub write: u64,
    pub _wat: String,
}

