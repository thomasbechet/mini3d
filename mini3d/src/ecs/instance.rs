pub(crate) struct ECSInstance {
    pub(crate) containers: ContainerTable,
    entities: EntityTable,
    queries: QueryTable,
    instances: SystemInstanceTable,
    pub(crate) scheduler: Scheduler,
    global_cycle: u32,
}
