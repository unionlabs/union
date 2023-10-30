use crate::cdsl::isa::TargetIsa;
use crate::cdsl::settings::{SettingGroup, SettingGroupBuilder};

use crate::shared::Definitions as SharedDefinitions;

fn define_settings(_shared: &SettingGroup) -> SettingGroup {
    let mut settings = SettingGroupBuilder::new("s390x");

    // The baseline architecture for cranelift is z14 (arch12),
    // so we list only facilities of later processors here.

    // z15 (arch13) facilities
    let has_mie2 = settings.add_bool(
        "has_mie2",
        "Has Miscellaneous-Instruction-Extensions Facility 2 support.",
        "",
        false,
    );
    let has_vxrs_ext2 = settings.add_bool(
        "has_vxrs_ext2",
        "Has Vector-Enhancements Facility 2 support.",
        "",
        false,
    );

    // Architecture level presets
    settings.add_preset(
        "arch13",
        "Thirteenth Edition of the z/Architecture.",
        preset!(has_mie2 && has_vxrs_ext2),
    );

    // Processor presets
    settings.add_preset(
        "z15",
        "IBM z15 processor.",
        preset!(has_mie2 && has_vxrs_ext2),
    );

    settings.build()
}

pub(crate) fn define(shared_defs: &mut SharedDefinitions) -> TargetIsa {
    let settings = define_settings(&shared_defs.settings);

    TargetIsa::new("s390x", settings)
}
