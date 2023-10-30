s! {
    pub struct ctl_info {
        pub ctl_id: u32,
        pub ctl_name: [::c_char; MAX_KCTL_NAME],
    }
}

pub const MAX_KCTL_NAME: usize = 96;
