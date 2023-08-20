use crate::utils::slotmap::SlotId;

pub(crate) enum PipelineStep {
    Exclusive {
        instance: SystemInstanceId,
        next: SlotId,
    },
    Parallel {
        instance: SystemInstanceId,
    },
}

pub(crate) struct SystemPipeline {
    steps: Vec<PipelineStep>,
}
