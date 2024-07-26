use rust_fsm::state_machine;

state_machine! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub union_mpc(AwaitContribution)
    InitContributor => {
        NoContributor => AwaitContributor,
        ContributorSet => AwaitContribution,
    },
    AwaitContributor => {
        Join => InitContributor,
    },
    AwaitContribution => {
        Contribute => Verify,
        Join => AwaitContribution,
        SetPriority => AwaitContribution,
    },
    Verify => {
        Valid => InitContributor,
        Invalid => InitContributor,
    },
}
